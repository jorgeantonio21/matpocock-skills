---
name: idiomatic-rust
description: Concise, idiomatic, readable Rust, each rule as an instruction with its reason. Use when writing new Rust, refactoring existing Rust, reviewing a diff that touches .rs or Cargo.toml files, deciding what a Rust type guarantees, or when another skill needs the Rust idiom baseline.
---

# Idiomatic Rust

The Rust rules a linter cannot enforce, each an instruction with its reason. To build, read the section for the work in hand, in order. To review, hold each changed function against every rule and name the rule on each finding. A documented standard in the repository overrides this file. The rules assume a library, a service, or an engine; a rule that names a narrower case, such as the hot path, applies only there. The check command in [LINTS.md](LINTS.md) enforces the mechanical rules, so this file does not repeat them.

Each rule is one of three kinds, and a finding names the kind:

- A **requirement** names a consequence, such as an invalid value in the core. A finding against it is a defect.
- A **default** picks one of several sound choices, such as a crate or a return shape. The project's convention overrides it.
- A **convention** is about a name, a comment, or a test name. A finding against it is advisory.

## Shape

Decide the types before you write a function body.

- **State the guarantee before you pick the type.** A wrapper earns its place when it protects an invariant, separates two roles a caller can confuse, or carries behavior. Otherwise keep the primitive: `usize` for an unbounded count, `String` for free text. Name the kind of guarantee. An intrinsic invariant is true of the value alone, so the type carries it. An aggregate invariant is true of the elements together, so every mutation re-establishes it. A contextual admission is true at one moment against one policy, so the type records the check and proves nothing later. [INVARIANTS.md](INVARIANTS.md) has the patterns, what each prevents, and what is enough instead.

- **The compiler is the guardrail.** A value with a role of its own gets a type of its own: `fn resize(width: Width, height: Height)` rejects `resize(height, width)`, where two `u32` parameters would compile. Two values of one type told apart by role go in a struct with named fields, `Move { from: FolderId, to: FolderId }`; two free-text strings such as a title and a body stay `String` inside it. Put a unit in the type where one exists: `Millis(u64)`, `Duration`.

- **Name the absence.** `Option<T>` means "not there" and nothing more: a map lookup, an optional config field. An absence with a domain meaning is an enum that names it, `enum Cached<T> { Fresh(T), Stale { last: T, age: Duration }, Empty }`, where `Option<T>` would hide three cases behind one `None` and a `match` names each. A `None` that needs a comment is an enum. Convert with `From<Option<T>>` at the boundary where a database or a wire format only knows null.

- **Make invalid states impossible.** "One of several" is an enum; "can fail" is `Result`. A `bool` parameter is a two-variant enum (`Visibility::Public` or `Private`), and a sentinel such as `-1` or `""` is a variant. Two fields where one is valid only while the other holds a given value are one enum with data. A state machine is an enum, not a set of flags. A closed set of strings is an enum, with `EnumString` and `Display` from `strum` for the boundary conversion (see [CRATES.md](CRATES.md)); `String` is for free text only: a name, a message, a path from a user. Put a phase in a type parameter (`Connection<Handshaking>` offers `complete()`, `Connection<Ready>` offers `send()`) only when a wrong-phase call is a bug the tests keep missing; otherwise a private field and a checked method are enough. A phantom type, a branded lifetime, or a version token needs a named bug it closes.

- **Parse at the boundary.** Raw input (bytes, JSON, a query string, a config file) becomes domain types at the edge, once, in one place. A validated newtype has a private field and one checked constructor, so the core never checks again and no raw `u64` or `String` enters it. A type read or written as raw bytes gets `#[repr(C)]` or `#[repr(transparent)]`, names every padding byte with a field, holds no pointer, `String`, `Vec`, or `Box`, and decodes through a checked conversion that rejects invalid bytes. A type that crosses JSON or protobuf uses serde or prost and keeps its `String` and `Vec` fields.

- **Close every route.** A private field stops direct construction and nothing else. List every route in (`new`, `From`, `TryFrom`, `FromStr`, `Deserialize`, each byte decoder, each database read, `Default`, each setter, each arithmetic operation) and send each through the checked constructor. A derived `Deserialize` and a delegating `FromStr` derive build the value without `new`, so a validated type gets `#[serde(try_from = "u32")]` and a `FromStr` that calls `new`; a decoder returns `Result`, and a database row takes the same `TryFrom`. Only an unconstrained wrapper, where every inner value is valid, derives these directly. An operation returns the refined type only when it preserves the invariant, and reports an independent failure such as overflow in its return type.

- **Newtype anatomy.** Private inner field. `const fn new` returns `Option<Self>` when there is a bound, through an `if` expression, because `bool::then_some` is not const-callable on stable. At a config or request boundary it returns `Result<Self, E>` with the rejected value in `E`, so the report names it. A panicking `const fn new_const` only when the code needs a literal of the type, such as a protocol constant. `From<Newtype> for Inner` always; `From<Inner>`, a derived `FromStr`, and a derived `Deserialize` only when every inner value is valid (see "Close every route"). `Display` delegates to the inner type (see [CRATES.md](CRATES.md)). Sentinels are associated consts: `JobId::MAX`, `Retries::ZERO`. `#[repr(transparent)]` when the layout matters, with a comment that says why. One `#[derive(...)]` line per ecosystem with a trailing comment that names it, so a reader sees which surfaces the type crosses and rustfmt keeps the lines apart.

  ```rust
  /// A system-generated job identifier. Every `u64` is valid, so the derives build it directly.
  #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
  #[derive(Serialize, Deserialize)] // serde
  #[derive(FromBytes, IntoBytes, KnownLayout, Immutable)] // zerocopy
  #[repr(transparent)] // wire layout is the inner u64
  pub struct JobId(u64);
  ```

- **Derive, or explain.** Derive `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, and `Default` where they are valid. Trait bounds go on impl blocks, not on the type definition. A hand-written impl exists only where the derive would be wrong, with the reason in a comment above it: `// do NOT derive Hash: the default hashes byte by byte and is slow`.

- **Private by default.** Fields are private; `pub(crate)` for an item other modules use, `pub` for an item other crates use. Give a type `new()` and derive `Default` when a no-argument value makes sense. A `pub` enum or struct that another crate matches on, and that will grow, is `#[non_exhaustive]`, so a new variant is not a breaking change. A builder belongs on a config struct with many optional fields, compile-time checked (see [CRATES.md](CRATES.md)), never on a domain type.

- **Small traits.** One capability per trait, in a ladder (`HasClock`, then `HasStore: HasClock`) so a caller asks for the weakest bound. A bound bundle is a marker trait with a blanket impl: `pub trait Message: Send + Sync + 'static {}` plus `impl<T: Send + Sync + 'static> Message for T {}`. A method on a foreign type is an extension trait `<Type>Ext`. Generics by default; `dyn Trait` for a heterogeneous collection or a pluggable backend (see [RUNTIME.md](RUNTIME.md)). `Deref` belongs on a smart pointer or an owning collection; a field's methods are forwarded, not reached through `Deref`.

## Errors

Decide the error types with the domain types.

- **Rejection is an outcome. Failure is an error.** A domain method returns `Result<XOutcome, XError>`: `XOutcome` has a `Rejected(Reason)` variant for every expected "no" (an unknown id, a full queue, a closed session), and `Err` means a broken invariant or a failed dependency. A rejection is a response to the caller, so it is neither an error variant nor a `warn!`. Every outcome enum is `#[must_use]` with a reason, so a `Rejected` cannot be dropped silently; a private outcome with one caller, whose one `match` handles every case, skips it. The outcome carries the facts the transition established (the removed children, the new expiry, the remaining count), because a second lookup can get a different answer (see [INVARIANTS.md](INVARIANTS.md)). A thin adapter or a script that only reports the rejection returns a plain `Result<T, E>`.

  ```rust
  #[must_use = "a rejection must reach the caller"]
  pub enum CancelOutcome { Cancelled { attempts: Retries }, Rejected(RejectReason) }

  impl Scheduler {
      pub fn cancel(&mut self, id: JobId) -> Result<CancelOutcome, SchedulerError> {
          let Some(job) = self.jobs.get_mut(&id) else {
              return Ok(CancelOutcome::Rejected(RejectReason::UnknownJob));
          };
  ```

- **Library errors use `thiserror`.** A library crate derives `#[derive(thiserror::Error, Debug)]`. One error enum per fallible surface (an operation or a module in a large crate, the whole crate in a small one) lets a caller match on exactly the failures of the operation it called. Each variant carries the data a caller acts on as fields (the id, the path, the offset, the expected and actual values) and its message in `#[error("...")]` naming them, never a preformatted `String`. A message is lowercase with no trailing period, as in std, so a chained report reads as one sentence per cause. A single-variant error is a struct. An error on a hot path is `Copy`, with no `String` or `Box` in it; an error at the edge or in a tool can carry a `String` path or a boxed cause.

- **Compose errors with `#[from]`, `#[source]`, and `transparent`.** `#[from]` when the inner error converts into this variant and nothing else, so `?` converts it; one inner type carries `#[from]` on one variant only. `#[source]` when the variant adds fields, or the same inner type feeds two variants: it keeps the cause chain and generates no `From`. `#[error(transparent)]` on a variant that only wraps another error, so the wrapped message and chain show through. Below, `JobNotFound` is a `Copy` struct error with its own `#[error]` message.

  ```rust
  #[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
  pub enum SchedulerError {
      #[error(transparent)]
      JobNotFound(#[from] JobNotFound),
      #[error("worker {0:?} is offline")]
      WorkerOffline(WorkerId),
  }
  ```

- **Translate errors at a layer boundary.** An exhaustive `From<InnerError> for OuterError` at each boundary names every variant, with no `_` arm, so a new inner variant fails the build until the translation handles it.

- **A module-local `Result` alias is optional.** When a module has one error enum and its fallible functions all return it, an alias shortens the signatures. The right side is `std::result::Result` (or `core::result::Result`), so the alias does not name itself; its visibility matches the callers. Keep the explicit `Result<T, E>` when the function is generic over the error, the module has several error types, or the alias would collide with an import of std's. This is a readability convention: it requires no alias per error enum and no crate-wide error type, and a test keeps `anyhow::Result<()>`.

  ```rust
  #[derive(Debug, Error)]
  pub enum DecodeError {
      #[error("invalid header")]
      InvalidHeader,
      #[error("truncated input")]
      TruncatedInput,
  }

  pub type Result<T> = std::result::Result<T, DecodeError>;

  pub fn header(bytes: &[u8]) -> Result<Header> {
  ```

- **Application errors use `anyhow`.** A binary crate and a test return `anyhow::Result<T>`; in a library, `anyhow` is a dev-dependency. Add context at each layer where the lower message is not enough on its own: `.context("load the service config")?` or `.with_context(|| format!("read {path}"))?`. A library function never returns `anyhow::Error`, no signature uses `Box<dyn Error>`, `String`, or `()` as an error type, and one crate's public interface uses one convention.

- **Propagate every error.** `?` everywhere a caller can act on the error. A `let _ = fallible();` carries a comment that says why the result does not matter. An error becomes a log line only in an operation that is best-effort by design, with a comment that says so.

## Ownership

Decide the signatures.

- **Take what you use.** Take `&str`, `&[T]`, `&Path`, and `&T`; take an owned value only when the function stores or consumes it. A method on a `Copy` newtype takes `self` by value.

- **Return by need.** `&str` or `&[T]` for a view of a field. `impl Iterator<Item = T>` when the caller consumes a sequence once, `Vec<T>` when it keeps the collection. Box an iterator only when the body returns one of several iterator types, or a trait object stores it. `Cow<'_, str>` when most calls return the input unchanged and the rest build a new value.

- **Clone only for a second owner.** A `.clone()` that silences the borrow checker is replaced by one of four moves: shrink the borrow with an inner block, move the value out with `mem::take` or `Option::take`, split the struct so the fields borrow independently, or borrow in the signature. A reference-count increment is `Arc::clone(&x)`, not `x.clone()`.

- **Elide lifetimes.** Name a lifetime only when the output borrows from one of several inputs, or a struct stores a reference. A public struct owns its data: `Vec<u8>`, not `&'a [u8]`. A lifetime error is fixed in the borrow, not escaped with `'static`, `Arc`, or `Rc`.

- **Share by message.** State has one owner, and other threads send it messages through a channel. A flag or a counter is an atomic, and a snapshot is an `ArcSwap`. A lock is the choice when one operation must read and write several fields together and no owner fits: hold it for the few lines that touch the shared state, with the preparation and the I/O outside, and never return the guard from a method. On a hot path there is no lock and no structure that locks inside; [RUNTIME.md](RUNTIME.md) names the alternatives.

- **Time and randomness are inputs.** A state transition takes `now: Timestamp` as a parameter, and `&mut impl Rng` when it needs randomness; `Instant::now()`, `SystemTime::now()`, and `thread_rng()` are called at the edge, once per event. The same inputs then replay the same state, and a test passes a constant. `Instant` measures a duration; `SystemTime` records a wall-clock time.

- **Rebind, then move.** Rebind a captured variable in a block around a `move` closure and keep the outer name inside, instead of `let n2_cloned = n2.clone()`. Freeze prepared data the same way: after the rebinding, the compiler rejects a later mutation.

  ```rust
  let on_tick = {
      let counter = Arc::clone(&counter);
      move || counter.fetch_add(1, Ordering::Relaxed)
  };
  let jobs = { let mut jobs = fetch_jobs(); jobs.sort_unstable(); jobs };
  ```

## Flow

Write the function body.

- **Transform, do not match.** `?`, `if let`, `while let`, `let ... else`, `map`, `map_err`, `ok_or_else`, `and_then`, `is_some_and`, `then_some`, and `matches!` handle `Option` and `Result`; a `match` appears only when more than one arm binds a value. A let chain, `if let Some(job) = slot && job.is_ready() && let Some(worker) = free_worker()` (edition 2024, Rust 1.88), replaces a stack of nested `if let` blocks. When the domain has already answered the error, write `let Ok(x) = res else { return Ok(Outcome::Rejected(reason)) };`.

- **Guards first, happy path flat.** Every guard sits at the top of the function with a one-line reason comment above it: one `let ... else` per binding that exits, a let chain for a guard that is a condition rather than an exit. The happy path stays at the lowest indentation. A value with several exit points is built in a labeled block, with the label named for the thing it escapes.

  ```rust
  let removed = 'queue: {
      let Some(worker) = self.workers.get_mut(&worker_id) else { break 'queue false };
      let Some(queue) = worker.queues.get_mut(&priority) else { break 'queue false };
      queue.remove(&job_id).is_some()
  };
  ```

- **Match exhaustively.** A `match` on a local enum names every variant, with no `_` arm, so a new variant fails the build; arms with the same body are grouped with `|`. A policy that several consumers apply to one enum lives on the enum, as one method with an exhaustive `match`, so a new variant is decided in one place. The message of `unreachable!` and `debug_assert!` states the invariant: `debug_assert!(!queue.is_empty(), "the map holds no empty queue")`.

- **Chain to build, loop to consume.** Build a collection with an iterator chain and `collect()`; use `for x in &v` when the body has side effects. `filter_map` and `find_map` where one closure filters and maps. A fallible map collects with `collect::<Result<Vec<_>, E>>()?`, never with an `unwrap` in the closure. std methods first; itertools for `tuple_windows`, `chunk_by`, `kmerge`, and `exactly_one` (see [CRATES.md](CRATES.md)).

- **A panic is a decision.** `unwrap`, `expect`, and indexing belong in tests, in `main`, and at a site that cannot fail. At such a site write `expect` with the invariant as the message, or, for an index, the invariant in a comment within two lines; `unwrap` fails the check. That site gets neither `unreachable!`, nor a new error variant, nor a `?` that cannot fail. `panic!` handles no error. A `drop(x)` before the end of the scope carries a comment that says why.

- **Name every number.** A literal other than `0`, `1`, and `2` is a `const`, or an associated const on the type that owns the value: `Backoff::INITIAL`, `JobId::MAX`.

- **Check arithmetic on external values.** A value from the wire or from a client goes through `checked_add`, `checked_mul`, or `saturating_sub`, and an overflow maps to an error or a rejection. A plain operator needs a comment that states the bound that makes it safe.

## Surface

Name the items and shape the public interface.

- **Imports at the top.** Every non-std path is imported at the top of the file: no `use` inside a function, and no qualified path such as `crate::clock::Timestamp` at a call site or in a signature. Two colliding names get an alias: `use parking_lot::Mutex as SyncMutex;`. Macro paths and attribute paths are the exception.

- **Names follow the std conventions.** A conversion is named by cost and ownership: `as_` is free and borrowed, `to_` is expensive, `into_` consumes `self`. A getter is named after the field, without `get_`. A function is named for the decision it makes: `evict_stale_then_insert`. Each suffix has one meaning: `Outcome` a domain result, `Kind` a closed discriminant, `Reason` why a request was rejected, `Config` a configuration struct, `Handle` a cloneable owner of a worker, `Ext` an extension trait, `Has*` a capability trait. A generic parameter is the initial of its concept (`W` for worker), a label is the thing it escapes (`'queue`), and a closure parameter is the domain noun. A variable is shadowed through a transformation, `let job = parse(job)?;`, not split into `raw_job` and `parsed_job`. One name per concept in a diff, and the name the crate already uses.

- **`lib.rs` is a module list.** `lib.rs` is an alphabetized list of `pub mod`, `pub(crate) mod`, and `mod` lines, with `#[cfg(test)]` and feature gates on the `mod` line. `pub use` only exports an item from a private module.

- **Attributes say why.** `#[must_use = "reason"]`, never bare. `#[expect(lint, reason = "...")]` on the one item that needs it; no `#[allow]` at crate scope, and no `#![deny(warnings)]` in source. The `// SAFETY:` comment above an `unsafe` block names the obligation and says why it holds.

- **Bundle the arguments.** A function takes five positional parameters or fewer; more go in a request struct with named fields, never behind `#[allow(clippy::too_many_arguments)]`. A complex closure type gets a trait.

## Words

Write the comments, the docs, the tests, and the logs.

- **Write in Simplified Technical English.** Comments, doc comments, error messages, and log messages follow ASD-STE100: one instruction or one fact per sentence, an instruction at 20 words or fewer, a description at 25 or fewer, active voice, present tense, one word for one concept. Use the name the codebase uses: the type name, the function name, or the glossary entry in `CONTEXT.md` or `GLOSSARY.md` when one exists. No figure of speech, no synonym for variety, no pronoun without a clear noun.

- **Comments say why.** A comment exists only where the code cannot show the reason, and states that reason in one line: not what the code does, not a past implementation, not "we" or "this function". A comment or a `TODO` you did not write stays until the code it explains is gone or its work is done.

- **Docs state the rule and the consequence.** A doc comment opens with a one-line summary, then states the invariant and what breaks when it is violated: "Chosen once and fixed forever. A change to this seed changes every derived id and breaks replay." It repeats no parameter types, holds no design discussion, and links every item it mentions: `` [`JobId`] ``. A module doc (`//!`) is an operating procedure in a numbered list: how to add a variant, what may change and what must not, and what depends on the module. A variant doc says when the variant occurs.

- **Tests read as sentences and show their arithmetic.** Unit tests live in a `#[cfg(test)] mod tests` block in the same file. A test name starts with `test_` and then the behavior it checks, `test_backoff_doubles_after_each_failure`; the prefix marks a test in a grep, a panic message, and a profile, where the `#[test]` attribute is not visible, and drops only in a crate that enables `clippy::redundant_test_prefix`. A test returns `anyhow::Result<()>` so the body can use `?`. The expected value is derived in a comment above the assertion, and the message carries the values: `assert!(next > prev, "next ({next:?}) is above prev ({prev:?})")`. Time is a named constant, never `now()`. A property test is `test_prop_<behavior>`, and a table of inputs is one named case each (see [CRATES.md](CRATES.md)). Every new `pub fn` gets a test in the same crate. When a stored type gains a stricter check, a test keeps the old bytes and checks that they load with their old meaning, and that a corrupt input is an error, not a default.

- **Logs are structured.** Values go as key-value arguments, not formatted into the message. `info!` marks a lifecycle event, not a request or a message. A rejection is a response to the caller, not a `warn!` (see Errors).

## Check

Before you hand Rust back, run the check command from [LINTS.md](LINTS.md) on each crate the diff touches, then `cargo +nightly fmt` on those crates. Fix each finding in a file the diff touches, or put `#[expect(lint, reason = "...")]` on the one item. A crate that never ran the check has a backlog in its other files: report the count and leave it.

## Pointers

- **You are deciding what a type guarantees, or reviewing a type that claims one.** Read [INVARIANTS.md](INVARIANTS.md).
- **The diff has async code, tasks, or threads.** Read [RUNTIME.md](RUNTIME.md).
- **You are about to write an impl by hand that a crate would derive.** Read [CRATES.md](CRATES.md).
- **You need the check command or the lint policy.** Read [LINTS.md](LINTS.md).
