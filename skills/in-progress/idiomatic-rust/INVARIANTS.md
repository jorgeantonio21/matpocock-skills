# Invariant patterns

Read the pattern relevant to the change. Each example is independent; the examples do not assemble into an application. The evaluation harness compiles and runs these Rust blocks, including the Serde example.

## Intrinsic value invariant and checked entry paths

**Guarantee:** a `Limit` contains a value from 1 through 10. **Prevents:** parsing, derives, defaults, database loading, mutation, or arithmetic bypassing the constructor. **Applies:** a bound belongs to the value itself. **Alternatives:** use `NonZeroUsize` when only nonzero matters, or a primitive with local validation when no lasting guarantee is promised.

The module owns the guarantee. Privacy restricts outside callers, but code inside the module can still construct directly. Audit every route, including future setters and operator implementations. Derive `Default` only if it produces a valid value. Deserializing a private tuple field ordinarily constructs it without calling `new`; a delegating `FromStr` has the same problem.

```rust
mod limits {
    use serde::Deserialize;
    use std::str::FromStr;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
    #[serde(try_from = "u8")]
    pub struct Limit(u8);

    impl Limit {
        pub const fn new(value: u8) -> Option<Self> {
            if value >= 1 && value <= 10 { Some(Self(value)) } else { None }
        }
        pub const fn get(self) -> u8 { self.0 }
    }

    impl TryFrom<u8> for Limit {
        type Error = String;
        fn try_from(value: u8) -> Result<Self, Self::Error> {
            Self::new(value).ok_or_else(|| format!("limit {value} is outside 1..=10"))
        }
    }

    impl FromStr for Limit {
        type Err = String;
        fn from_str(raw: &str) -> Result<Self, Self::Err> {
            let value = raw.parse::<u8>().map_err(|error| error.to_string())?;
            Self::try_from(value)
        }
    }
}

use limits::Limit;
const TEN: Option<Limit> = Limit::new(10);
assert_eq!(TEN.map(Limit::get), Some(10));
assert!(Limit::new(11).is_none());
assert!(Limit::try_from(11).is_err());
assert!("11".parse::<Limit>().is_err());
assert!(serde_json::from_str::<Limit>("11").is_err());
assert_eq!(serde_json::from_str::<Limit>("10").unwrap().get(), 10);
```

A database adapter should similarly decode a raw numeric value and call `TryFrom`. Add that path to tests when a database adapter exists. A public mutable reference to the inner value would invalidate the guarantee; expose checked replacement if mutation is needed. Diagnostics may allocate at an input boundary; choose a typed, nonallocating error if this is a measured hot path.

## Aggregate invariant

**Guarantee:** ranges are nonempty, ordered, and non-overlapping. **Prevents:** individually valid elements forming an invalid collection, or a mutation exposing partial invalid state. **Applies:** correctness relates multiple values. **Alternatives:** validate an immutable snapshot at each boundary, or use an interval structure whose operations enforce the same relationships.

```rust
mod ranges {
    use std::ops::Range;

    #[derive(Debug)]
    pub struct Ranges(Vec<Range<u32>>);

    impl Ranges {
        pub fn new(raw: Vec<Range<u32>>) -> Result<Self, &'static str> {
            if raw.iter().any(|range| range.start >= range.end)
                || raw.windows(2).any(|pair| pair[0].end > pair[1].start)
            {
                return Err("ranges must be nonempty, ordered, and non-overlapping");
            }
            Ok(Self(raw))
        }
        pub fn as_slice(&self) -> &[Range<u32>] { &self.0 }
        pub fn replace(&mut self, raw: Vec<Range<u32>>) -> Result<(), &'static str> {
            let next = Self::new(raw)?;
            *self = next;
            Ok(())
        }
    }
}

let mut ranges = ranges::Ranges::new(vec![1..3, 3..5]).unwrap();
assert!(ranges.replace(vec![1..4, 3..5]).is_err());
assert_eq!(ranges.as_slice(), &[1..3, 3..5]);
assert!(ranges::Ranges::new(vec![5..7, 1..3]).is_err());
```

This contract rejects unsorted input. Sorting can be a legitimate alternative only if order has no domain meaning and normalization is part of the documented API. Do not silently drop overlaps to construct a stronger type.

## Contextual admission

**Guarantee:** the authority accepted work under the policy at admission. **Prevents:** callers manufacturing acceptance, or execution accidentally reapplying a newer admission policy to grandfathered work. **Applies:** requests depend on configuration, authorization, quotas, or other state. **Alternatives:** recheck at execution when policy explicitly requires revocation; reserve resources when admission promises capacity; use version tokens or identity binding only when crossing authorities requires them.

```rust
mod admission {
    use std::collections::VecDeque;

    #[derive(Default)]
    pub struct Scheduler {
        max_new_tokens: u32,
        accepted: VecDeque<u32>,
    }
    impl Scheduler {
        pub fn set_max_new_tokens(&mut self, max: u32) { self.max_new_tokens = max; }
        pub fn admit(&mut self, tokens: u32) -> Result<(), &'static str> {
            if tokens == 0 || tokens > self.max_new_tokens {
                return Err("request exceeds admission policy");
            }
            self.accepted.push_back(tokens);
            Ok(())
        }
        pub fn next_accepted(&mut self) -> Option<u32> { self.accepted.pop_front() }
    }
}

let mut scheduler = admission::Scheduler::default();
scheduler.set_max_new_tokens(10);
scheduler.admit(8).unwrap();
scheduler.set_max_new_tokens(4);
assert!(scheduler.admit(8).is_err());
assert_eq!(scheduler.next_accepted(), Some(8));
```

The private queue and narrow admission method suffice here; a branded request is unnecessary. This guarantees historical acceptance, not current resource availability or continuing authorization. If policy changes must revoke work, implement that transition explicitly and test it. If grandfathering is promised, a stronger internal representation must continue to represent that valid work.

## Raw-to-validated boundary

**Guarantee:** structurally decoded input acquires semantic validity only after checked conversion. **Prevents:** treating a parse success as authorization or a valid domain request. **Applies:** JSON, frames, config, or database records. **Alternatives:** deserialize directly through checked conversion, as the `Limit` example does, or fuse decoding and validation behind one public function.

```rust
#[derive(Debug)]
struct RawRequest { batch: u8, tokens: u16 }

#[derive(Debug, PartialEq, Eq)]
struct Request { batch: u8, tokens: u16 }

fn decode(bytes: &[u8]) -> Result<RawRequest, &'static str> {
    let [batch, lo, hi] = bytes else { return Err("expected three bytes"); };
    Ok(RawRequest { batch: *batch, tokens: u16::from_le_bytes([*lo, *hi]) })
}

impl TryFrom<RawRequest> for Request {
    type Error = &'static str;
    fn try_from(raw: RawRequest) -> Result<Self, Self::Error> {
        if raw.batch == 0 || raw.tokens == 0 || u16::from(raw.batch) > raw.tokens {
            return Err("every batch item needs at least one token");
        }
        Ok(Self { batch: raw.batch, tokens: raw.tokens })
    }
}

assert!(decode(&[3, 2, 0]).is_ok());
assert!(Request::try_from(decode(&[3, 2, 0]).unwrap()).is_err());
assert_eq!(Request::try_from(decode(&[2, 3, 0]).unwrap()).unwrap().tokens, 3);
```

These types are private to this small example. In a public library, put the validated type and its conversions in the module that owns the guarantee. A validation library on `RawRequest` can help report several problems, provided this conversion cannot skip it.

## Invariant-preserving operation

**Guarantee:** successful negation preserves nonzero and representability. **Prevents:** overflow at the minimum signed value, even though zero was excluded. **Applies:** signed nonzero values or any refined numeric type. **Alternatives:** use a wider result type, or reject the minimum at construction only if the domain truly excludes it. Do not silently narrow the accepted domain to make an operator total.

```rust
use std::num::NonZeroI32;

fn checked_negate(value: NonZeroI32) -> Option<NonZeroI32> {
    value.get().checked_neg().and_then(NonZeroI32::new)
}

assert_eq!(checked_negate(NonZeroI32::new(7).unwrap()).unwrap().get(), -7);
assert_eq!(checked_negate(NonZeroI32::new(i32::MIN).unwrap()), None);
```

The return type reflects two facts: success is nonzero, and not every input can be negated. Check addition, multiplication, conversion, and mutation against their own guarantees too. Saturating arithmetic is only correct when saturation is the intended semantic operation.

## Authoritative transition result

**Guarantee:** removal reports the children actually removed and the remaining state at that transition. **Prevents:** reconstructing effects from a second mutable-state lookup, which may observe a later state or find the removed data already gone. **Applies:** caches, resource registries, job trees, or stateful services. **Alternatives:** compute everything in one transaction, return a stable snapshot, or emit an event from the committing authority.

```rust
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq)]
struct Removal { children: Vec<u64>, remaining_groups: usize }

fn remove(groups: &mut HashMap<String, Vec<u64>>, key: &str) -> Option<Removal> {
    let children = groups.remove(key)?;
    Some(Removal { children, remaining_groups: groups.len() })
}

let mut groups = HashMap::from([("a".to_owned(), vec![1, 2])]);
let removed = remove(&mut groups, "a").unwrap();
groups.insert("b".to_owned(), vec![3]);
assert_eq!(removed, Removal { children: vec![1, 2], remaining_groups: 0 });
```

The result describes one transition, not the current state forever. With concurrent storage, removal and the reported state need one appropriate transaction or synchronization scope. Merely placing two independent database reads in this function would not provide that guarantee.

## Shared interpretation

**Guarantee:** consumers use one definition of a status policy. **Prevents:** retry or visibility rules drifting across call sites. **Applies:** several consumers share the same meaning. **Alternatives:** a pure policy function, or an injected policy when consumers intentionally use different rules.

```rust
#[derive(Clone, Copy)]
enum Status { Queued, Running, Succeeded, Failed }

impl Status {
    fn is_terminal(self) -> bool {
        match self {
            Self::Queued | Self::Running => false,
            Self::Succeeded | Self::Failed => true,
        }
    }
}

assert!(!Status::Queued.is_terminal());
assert!(!Status::Running.is_terminal());
assert!(Status::Succeeded.is_terminal());
assert!(Status::Failed.is_terminal());
```

An exhaustive match makes a new variant require a decision. It cannot decide the correct policy for that variant; add a behavior test when extending it.

## Persistent representation change

**Guarantee:** historical bytes retain their supported meaning while new writes follow stricter validation. **Prevents:** an apparently stronger representation silently changing semantics or making valid stored data unreadable. **Applies:** versioned configuration, snapshots, event logs, and database migrations. **Alternatives:** an explicit offline migration, retaining the old representation, or a versioned compatibility branch with a documented retirement policy.

```rust
use std::num::NonZeroU8;

#[derive(Debug, PartialEq, Eq)]
enum Capacity { LegacyUnlimited, Bounded(NonZeroU8) }

fn load(bytes: &[u8]) -> Result<Capacity, &'static str> {
    match bytes {
        [1, 0] => Ok(Capacity::LegacyUnlimited),
        [1 | 2, value] => NonZeroU8::new(*value)
            .map(Capacity::Bounded)
            .ok_or("zero is invalid in version 2"),
        _ => Err("unsupported version or corrupt record"),
    }
}

// Version 1 encoded unlimited as zero; version 2 allows only bounded writes.
assert_eq!(load(&[1, 0]), Ok(Capacity::LegacyUnlimited));
assert_eq!(load(&[1, 4]), load(&[2, 4]));
assert!(load(&[2, 0]).is_err());
assert!(load(&[3, 4]).is_err());
assert!(load(&[1]).is_err());
assert!(load(&[1, 4, 0]).is_err());
```

Test literal historical bytes, not just round-trips produced by the new serializer. Test semantic migration, unknown versions, truncated and corrupt input, and the new writer's constraints. `LegacyUnlimited` is a supported exception, not corrupt data to clamp to one. The loader illustrates reading; a production migration also needs tests for writing, rollback, and partial migration where those operations are supported.
