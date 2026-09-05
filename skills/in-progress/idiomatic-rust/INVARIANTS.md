# Invariants

Read this file when you decide what a type guarantees, or when you review a type that claims a guarantee. Each pattern states the guarantee, the invalid behavior it prevents, when it applies, and what is enough in a simpler case. Follow it together with [SKILL.md](SKILL.md). Every snippet is an excerpt of a module in [`examples/`](examples/), which compiles, passes its tests, and passes the check in [LINTS.md](LINTS.md) on Rust 1.97.1. `evals/check.sh` fails when a snippet and its module differ.

## Three kinds of guarantee

Name the kind before you pick the machinery. Each kind is established in a different place, and each breaks in a different way.

| Kind | Example | What establishes it | What breaks it |
| --- | --- | --- | --- |
| Intrinsic value invariant | A nonzero concurrency limit | The representation, or a checked constructor on every route in | An unchecked route in, or an operation that does not preserve it |
| Aggregate invariant | An ordered set of non-overlapping ranges | Validation of the whole aggregate, and mutation only through the type | A decoder that fills the collection directly, or an exposed `&mut` |
| Contextual admission | A request accepted under a policy | The authority that holds the policy at the time of the check | A later change to the policy or the state the check read |

An intrinsic invariant is true of the value alone, so the type can carry it forever. An aggregate invariant is true of the elements together, so every mutation must re-establish it. A contextual admission is true at one moment against one policy. The type records the admission. It does not prove that the value passes a later policy.

## When a wrapper is not the answer

Introduce a newtype when it protects an invariant, separates two roles a caller can confuse, or carries behavior. Keep the primitive when the wrapper adds none of the three. A `usize` count with no bound, a `String` message, and a `Duration` stay as they are. A wrapper that only renames a primitive costs a conversion at every boundary and protects nothing.

Module privacy and a narrow constructor are enough for most guarantees. A private field with one checked `new` closes every route a caller has. Reach for a phantom type, a branded lifetime, a version token, or a non-`Copy` handle only when a named bug survives the private field. Say which bug in the doc comment.

A stricter representation must keep every valid behavior of the domain, including a supported exception. Do not round, drop, or reinterpret a valid input to make it fit a stronger type. When a rule has an exception, put the exception in the type or in the constructor, and test it.

## The patterns

### Intrinsic value invariant

- **Guarantee**: the limit is never zero, from construction to the last use.
- **Prevents**: a zero limit that blocks every job, and a division by zero downstream.
- **Applies when**: the invariant is a property of the value alone. Nonzero, a bounded range, a valid UTF-8 name, an aligned size.
- **Enough instead**: a `NonZeroU32` field with no wrapper, when no operation or route needs a name of its own.

The representation carries the invariant. Every route in goes through `new`: `TryFrom`, `FromStr`, and `Deserialize` through `#[serde(try_from = "u32")]`. A delegating `FromStr` derive or a plain `Deserialize` derive would build the value without `new`. An operation returns `Self` only when it preserves the invariant, so `halve` stops at one.

```rust
/// The number of jobs a pool runs at once. Never zero: a zero limit blocks every job forever.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[serde(try_from = "u32", into = "u32")]
pub struct Concurrency(NonZeroU32);
```

Inside `impl Concurrency`, `Self` is `Concurrency`, so `new` returns `Option<Concurrency>`: `Some(Concurrency(nonzero))` for a nonzero input, `None` for zero.

```rust
impl Concurrency {
    /// The smallest limit a pool accepts.
    pub const ONE: Self = Self(NonZeroU32::MIN);

    /// Returns the limit, or `None` when `value` is zero.
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(nonzero) => Some(Self(nonzero)),
            None => None,
        }
    }

    /// The limit as a plain number, for a log line or a semaphore.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    /// Half the limit, rounded down and never below one. The invariant survives the operation.
    #[must_use]
    pub const fn halve(self) -> Self {
        match NonZeroU32::new(self.0.get() / 2) {
            Some(half) => Self(half),
            None => Self::ONE,
        }
    }
}
```

```rust
impl FromStr for Concurrency {
    type Err = ParseConcurrencyError;

    fn from_str(text: &str) -> Result<Self, ParseConcurrencyError> {
        let value: u32 = text.parse()?;
        Ok(Self::try_from(value)?)
    }
}
```

### Aggregate invariant

- **Guarantee**: the windows are sorted, and no two overlap.
- **Prevents**: a lookup that finds the wrong window, and a binary search on unsorted data.
- **Applies when**: the invariant is a relationship between elements. Sorted, unique, non-overlapping, balanced, a total that matches a sum.
- **Enough instead**: a `BTreeSet` or a `BTreeMap`, when the standard collection already holds the relationship.

The whole aggregate is checked at once. `new` and `insert` are the only routes in, and the `Vec` is private, so no caller can push. A rejected insert leaves the list unchanged. The JSON route goes through `#[serde(try_from = "Vec<Range<u32>>")]`, so a decoded list takes the same check. Readers get a slice, never a `&mut Vec`. From `impl Windows`:

```rust
    /// Inserts one window in its sorted place.
    ///
    /// # Errors
    ///
    /// An empty window or an overlap is rejected, and the list is unchanged.
    pub fn insert(&mut self, window: Range<u32>) -> Result<(), WindowsError> {
        if window.is_empty() {
            return Err(WindowsError::Empty(window));
        }
        let at = self
            .0
            .partition_point(|existing| existing.start < window.start);
        // The neighbour before starts earlier, so only its end can reach into the new window.
        if let Some(before) = at.checked_sub(1).and_then(|i| self.0.get(i))
            && before.end > window.start
        {
            return Err(WindowsError::Overlap {
                new: window,
                existing: before.clone(),
            });
        }
        // The neighbour after starts at or later, so only the new window's end can reach it.
        if let Some(after) = self.0.get(at)
            && window.end > after.start
        {
            return Err(WindowsError::Overlap {
                new: window,
                existing: after.clone(),
            });
        }
        self.0.insert(at, window);
        Ok(())
    }
```

### Contextual admission

- **Guarantee**: an `Admitted` value passed the policy that was in force when it was checked.
- **Prevents**: a request that reaches the queue without a check, and a check scattered across callers.
- **Applies when**: acceptance depends on state outside the value. A policy, a configuration, a quota, a permission.
- **Enough instead**: a check at the one call site, when the value never travels past it.

The authority builds the admission. `Limits::admit` is the only constructor, and `Admitted` records the policy that checked it. The record says nothing about a later policy. When the policy changes, the consumer decides what happens to earlier admissions and says so in code. `KeepEarlier` serves work the old policy admitted. `Readmit` checks it again and returns what left the queue. Neither happens by accident of the type. From `impl Limits`:

```rust
    /// Admits `request` under this policy, or says why not.
    ///
    /// # Errors
    ///
    /// A prompt longer than `max_tokens` is rejected.
    pub const fn admit(self, request: Request) -> Result<Admitted, RejectReason> {
        if request.tokens > self.max_tokens {
            return Err(RejectReason::TooManyTokens {
                tokens: request.tokens,
                max: self.max_tokens,
            });
        }
        Ok(Admitted {
            request,
            admitted_under: self.id,
        })
    }
```

From `impl Queue`:

```rust
    /// Replaces the policy. What happens to earlier admissions is `on_change`, chosen by the caller.
    /// Returns the requests that left the queue.
    pub fn set_limits(&mut self, limits: Limits, on_change: OnPolicyChange) -> Vec<Request> {
        self.limits = limits;
        match on_change {
            OnPolicyChange::KeepEarlier => Vec::new(),
            OnPolicyChange::Readmit => {
                let (kept, dropped): (Vec<_>, Vec<_>) = self
                    .queued
                    .iter()
                    .partition(|admitted| limits.admit(admitted.request()).is_ok());
                self.queued = kept;
                dropped.into_iter().map(Admitted::request).collect()
            }
        }
    }
```

### Raw-to-validated boundary

- **Guarantee**: a `Header` has a supported version and a payload that fits a frame, whichever route decoded it.
- **Prevents**: a structurally valid input with an invalid meaning that reaches the core.
- **Applies when**: input arrives as bytes, JSON, a query string, or a database row, and a decoder can build the type.
- **Enough instead**: one `TryFrom<Raw>` with no raw type, when only one route exists and serde is not on it.

Parsing structure and validating meaning are two steps. serde checks that the fields exist and have the right types, and builds a `RawHeader`. `Header::try_from` checks what the fields mean. `Header` derives `Deserialize` through `#[serde(try_from = "RawHeader")]`, so the JSON route cannot skip the second step. The byte route builds a `RawHeader` and takes the same step. From `impl Header`:

```rust
    /// Decodes the fixed prefix of a frame.
    ///
    /// # Errors
    ///
    /// The same errors as [`Header::try_from`]: the bytes are structurally a header, and the
    /// meaning is checked in the one place.
    pub fn from_bytes(bytes: [u8; Self::SIZE]) -> Result<Self, HeaderError> {
        let [version, len @ ..] = bytes;
        Self::try_from(RawHeader {
            version,
            payload_len: u32::from_be_bytes(len),
        })
    }
```

```rust
impl TryFrom<RawHeader> for Header {
    type Error = HeaderError;

    fn try_from(raw: RawHeader) -> Result<Self, HeaderError> {
        if raw.version != Self::VERSION {
            return Err(HeaderError::UnsupportedVersion(raw.version));
        }
        if raw.payload_len > Self::MAX_PAYLOAD {
            return Err(HeaderError::PayloadTooLong {
                len: raw.payload_len,
                max: Self::MAX_PAYLOAD,
            });
        }
        Ok(Self {
            payload_len: raw.payload_len,
        })
    }
}
```

### Invariant-preserving operation

- **Guarantee**: the result of the operation holds the invariant, and an independent failure is reported, not hidden.
- **Prevents**: a panic in debug and a silent wrap in release on `i32::MIN`, and a zero that the type forbids.
- **Applies when**: an operation on a refined type can leave the refined set, or can fail for a reason the invariant does not cover.
- **Enough instead**: a plain operator with a comment that states the bound that makes it safe.

Nonzero survives negation, so `checked_neg` returns `Option<Self>` and not `Option<i32>`. Nonzero does not remove overflow, so the `Option` stays. `magnitude` returns a `NonZeroU32` because the distance of a nonzero value is nonzero, and `i32::MIN` fits. `sign` is total because zero is excluded. The return types say what is true, and no more. From `impl Offset`:

```rust
    /// The distance, which is nonzero because the offset is. `i32::MIN` fits: its magnitude is `2^31`.
    #[must_use]
    pub const fn magnitude(self) -> NonZeroU32 {
        self.0.unsigned_abs()
    }

    /// The opposite offset, or `None` for `i32::MIN`, whose negation does not fit in an `i32`.
    #[must_use]
    pub const fn checked_neg(self) -> Option<Self> {
        match self.0.checked_neg() {
            Some(negated) => Some(Self(negated)),
            None => None,
        }
    }
```

### Authoritative transition result

- **Guarantee**: the outcome of a mutation carries the facts the mutation established.
- **Prevents**: a caller that reconstructs the result with a second lookup and gets a different answer.
- **Applies when**: a consumer needs to know what changed. An event, a log line, a reply, a cascade.
- **Enough instead**: a `bool` or a count, when no consumer needs the removed items.

`remove_group` holds the state while it runs, so it alone knows which members left with the group. The outcome carries them. A lookup after the call finds nothing, or a new group under the same id. The event is built from the outcome. `#[must_use]` with a reason stops a caller from dropping it. From `impl Registry`:

```rust
    /// Removes `group` and every member in it. The outcome says which members those were.
    pub fn remove_group(&mut self, group: GroupId) -> RemoveOutcome {
        let Some(members) = self.groups.remove(&group) else {
            return RemoveOutcome::Rejected(RejectReason::UnknownGroup(group));
        };
        RemoveOutcome::Removed {
            members,
            groups_left: self.groups.len(),
        }
    }
```

```rust
/// Removes `group` and builds the event from the outcome, the one record of what was removed.
pub fn remove_and_announce(registry: &mut Registry, group: GroupId) -> Option<GroupRemoved> {
    match registry.remove_group(group) {
        RemoveOutcome::Removed {
            members,
            groups_left: _,
        } => Some(GroupRemoved { group, members }),
        RemoveOutcome::Rejected(RejectReason::UnknownGroup(_)) => None,
    }
}
```

### Shared interpretation

- **Guarantee**: every consumer applies the same policy to a status, because the policy has one definition.
- **Prevents**: two `matches!` expressions that drift apart, and a new variant that one consumer classifies and another ignores.
- **Applies when**: several modules interpret the same enum. Metrics, a scheduler, an archive, a reply.
- **Enough instead**: a local `matches!`, when one consumer exists and the enum is private to it.

The policy lives on the type that owns the status. The `match` names every variant, so a new variant fails the build until the policy decides it. A `_` arm would give the new variant a silent default.

```rust
impl JobStatus {
    /// Whether the job is finished for good. Every consumer that counts finished jobs asks here.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self {
            Self::Succeeded | Self::Cancelled | Self::Failed { retryable: false } => true,
            // A retryable failure is not the end: the scheduler queues the job again.
            Self::Queued | Self::Running | Self::Failed { retryable: true } => false,
        }
    }
}
```

### Persistent representation change

- **Guarantee**: a file written under the old rules loads with the meaning it had, and the new rules reject what they must.
- **Prevents**: a silent reinterpretation of old data, a migration that rounds a valid value, and a corrupt file that loads as a default.
- **Applies when**: a stored type gains a stricter check. A config file, a snapshot, a database column, a wire message.
- **Enough instead**: a version bump with no migration, when no file of the old version exists and none can arrive.

Version 1 stored `workers: 0` to mean "one per core". Version 2 writes `"auto"` and rejects zero. `load` reads both. A version 1 zero becomes `Workers::Auto`, so the meaning is kept and not rounded to one. The tests hold the historical bytes, the migrated meaning, the stricter rejection, and a corrupt input. From `impl TryFrom<Stored> for Config`:

```rust
    fn try_from(stored: Stored) -> Result<Self, LoadError> {
        let workers = match (stored.version, stored.workers) {
            // Version 1 wrote zero for "one per core". The meaning is kept.
            (1, StoredWorkers::Count(0)) => Workers::Auto,
            (2, StoredWorkers::Count(0)) => return Err(LoadError::ZeroWorkers),
            (1 | 2, StoredWorkers::Count(count)) => {
                let count = NonZeroU32::new(count).ok_or(LoadError::ZeroWorkers)?;
                Workers::Fixed(count)
            }
            (2, StoredWorkers::Word(word)) if word == "auto" => Workers::Auto,
            (1 | 2, StoredWorkers::Word(word)) => return Err(LoadError::UnknownWord(word)),
            (version, StoredWorkers::Count(_) | StoredWorkers::Word(_)) => {
                return Err(LoadError::UnsupportedVersion(version));
            }
        };
        Ok(Self { workers })
    }
```

From the tests:

```rust
    /// A file written by a version 1 build. The bytes must keep loading as long as version 1 is supported.
    const GOLDEN_V1_AUTO: &str = r#"{"version":1,"workers":0}"#;
    const GOLDEN_V1_FIXED: &str = r#"{"version":1,"workers":4}"#;

    #[test]
    fn test_historical_zero_keeps_its_meaning() -> anyhow::Result<()> {
        assert_eq!(load(GOLDEN_V1_AUTO)?.workers, Workers::Auto);
        assert_eq!(load(GOLDEN_V1_FIXED)?.workers.fixed(), Some(4));
        Ok(())
    }
```

## Review questions

Ask these of every type in a diff that claims a guarantee. Name the question on the finding.

1. **Which kind is it?** Intrinsic, aggregate, or contextual. A contextual admission presented as an intrinsic invariant is the most common overclaim.
2. **Is every route closed?** List `new`, `From`, `TryFrom`, `FromStr`, `Deserialize`, each byte decoder, each database read, `Default`, each setter, and each arithmetic operation. A private field closes none of the derived ones.
3. **Does every operation preserve it?** An operation that can leave the refined set returns `Option` or `Result`. An independent failure such as overflow stays visible.
4. **Does the outcome carry the facts?** A consumer builds its event from the return value, not from a second lookup.
5. **Does the stricter type keep every valid behavior?** Old data, a supported exception, and work admitted under an earlier policy load and run as before. A test holds the historical bytes.
6. **Is the machinery the smallest that closes the bug?** A private field and one constructor first. A phantom type, a branded lifetime, or a version token only for a bug that survives them, named in the doc comment.
