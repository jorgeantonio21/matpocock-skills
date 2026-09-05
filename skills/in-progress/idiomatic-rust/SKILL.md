---
name: idiomatic-rust
description: Concise, idiomatic, readable Rust, each rule as an instruction with its reason. Use when writing new Rust, refactoring existing Rust, reviewing a diff that touches .rs or Cargo.toml files, deciding what a Rust type guarantees, or when another skill needs the Rust idiom baseline.
---

# Idiomatic Rust

This file holds the Rust rules that a linter cannot enforce. Each rule is an instruction with its reason. To build, read the section for the work in hand, in the order the sections appear. To review, compare each changed function against every rule and name the rule on each finding. A documented standard in the repository overrides this file. The rules assume a library, a service, or an engine. A rule that names a narrower case, such as the hot path, applies only in that case. The rules are of three kinds. A rule that names a consequence, such as an invalid value in the core, is a correctness requirement. A finding against it is a defect. A rule that picks one of several sound choices, such as a crate or a return shape, is a default. The project's convention overrides it. A rule about a name, a comment, or a test name is a readability convention, and a finding against it is advisory. Name the kind on each finding. The check command in [LINTS.md](LINTS.md) enforces the mechanical rules, so this file does not repeat them.

## Shape

Decide the types before you write a function body.

- **State the guarantee before you pick the type.** A wrapper earns its place when it protects an invariant, separates two roles a caller can confuse, or carries behavior. Otherwise keep the primitive: a count with no bound stays `usize`, and free text stays `String`. Name the kind of guarantee. An intrinsic invariant is true of the value alone, so the type carries it. An aggregate invariant is true of the elements together, so every mutation re-establishes it. A contextual admission is true at one moment against one policy, so the type records the check and does not prove a later one. Read [INVARIANTS.md](INVARIANTS.md) for the patterns, what each prevents, and what is enough instead.

- **The compiler is the guardrail.** Give a value with a role of its own a type of its own: `Width`, `Height`, `UserId`, `DocumentId`. Two parameters of one primitive type that a caller can swap are a defect the compiler accepts. Write `fn resize(width: Width, height: Height)`, so `resize(height, width)` does not compile. When two values share a type and only a role tells them apart, use a struct with named fields: `Move { from: FolderId, to: FolderId }`. Two free-text strings, such as a title and a body, stay `String` inside such a struct. Put a unit in the type where a unit exists: `Millis(u64)`, `std::time::Duration`.

- **Name the absence.** Use `Option<T>` only when `None` means "not there" and nothing more. Examples: a map lookup, the first element of a slice, an optional config field. When the absence has a domain meaning, define an enum that names it: `Uninitialized` or `Initialized(T)`, `Unpinned` or `Pinned(Index)`. Write `enum Cached<T> { Fresh(T), Stale { last: T, age: Duration }, Empty }` where `Option<T>` would hide three cases behind one `None`. A reader then sees why the value can be missing, and a `match` names every case. Do not use `Option<T>` where a comment would be needed to say what `None` means. Convert from `Option` with a `From<Option<T>>` impl at the boundary where a database or a wire format only knows null.

- **Make invalid states impossible.** Model "one of several" as an enum. Model "can fail" as `Result`. Do not pass a `bool` argument. Use a two-variant enum such as `Visibility::Public` and `Visibility::Private`. Do not use a sentinel value such as `-1` or an empty string. Merge two fields into one enum with data when one is valid only while the other has a given value. Model a state machine as an enum, not as a set of flags. When a call in the wrong phase of a value's life is a bug the tests keep missing, put the phase in a type parameter. `Connection<Handshaking>` offers `complete()`, and `Connection<Ready>` offers `send()`. A private field and a checked method are enough when the wrong-phase call is not a real risk. Do not add a phantom type, a branded lifetime, or a version token without a named bug it closes. Model a closed set of strings as an enum. Derive `EnumString` and `Display` from `strum` for the conversion at the boundary (see [CRATES.md](CRATES.md)). A `String` stays a string only for free text: a name, a message, a path from a user.

- **Parse at the boundary.** Convert raw input into domain types at the edge of the system, once, in one place. Raw input is bytes, JSON, a query string, or a config file. Make the constructor of a validated newtype the only way to create a value, and keep the field private. Every value inside the core is then valid, and the core never checks it again. Do not pass a raw `u64` or `String` into the core. Give a type that is read or written as raw bytes `#[repr(C)]` or `#[repr(transparent)]`. Name every padding byte with a field. Keep pointers, `String`, `Vec`, and `Box` out of it. Decode through a checked conversion that rejects invalid bytes, so an invalid frame never becomes a value. A type that crosses a JSON or a protobuf boundary uses serde or prost and keeps its `String` and `Vec` fields.

- **Close every route.** A private field stops direct construction and nothing else. List every way a value comes to exist. The routes are `new`, `From`, `TryFrom`, `FromStr`, `Deserialize`, each byte decoder, each database read, `Default`, each setter, and each arithmetic operation. Route each one through the checked constructor. A derived `Deserialize` and a delegating `FromStr` derive build the value without `new`, so a validated type gets `#[serde(try_from = "u32")]` and a `FromStr` that calls `new`. A decoder returns `Result`, and a database row goes through the same `TryFrom`. Only an unconstrained wrapper, where every inner value is valid, derives these directly. An operation returns the refined type only when it preserves the invariant. It reports an independent failure, such as overflow, in its return type.

- **Newtype anatomy.** Keep the inner field private. Write `const fn new`. When the value has a bound, return `Option<Self>` from `new` with an `if` expression: `if v <= MAX { Some(Self(v)) } else { None }`. `bool::then_some` cannot be called in a `const fn` on stable Rust, so the `if` is not a style choice. At a config or a request boundary, return `Result<Self, E>` instead, with the rejected value in `E`. The report then names the value. Add a `const fn new_const` that panics only when the code needs a literal of the type, such as a protocol constant. Implement `From<Newtype> for Inner` always. Implement `From<Inner> for Newtype`, and derive `FromStr` and `Deserialize` directly, only when every inner value is valid. Otherwise `new` is the only way in (see "Close every route"). Derive `Display` so it delegates to the inner type (see [CRATES.md](CRATES.md)). Put sentinel values in associated consts: `JobId::MAX`, `Retries::ZERO`. Add `#[repr(transparent)]` when the layout matters, with a comment that says why. Write one `#[derive(...)]` line per ecosystem with a trailing comment that names the ecosystem. The comment shows a reader which surfaces the type crosses, and it stops rustfmt from merging the lines.

  ```rust
  /// A system-generated job identifier. Every `u64` is valid, so the derives build it directly.
  #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
  #[derive(Serialize, Deserialize)] // serde
  #[derive(FromBytes, IntoBytes, KnownLayout, Immutable)] // zerocopy
  #[repr(transparent)] // wire layout is the inner u64
  pub struct JobId(u64);
  ```

- **Derive, or explain.** Derive `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, and `Default` where they are valid. Put trait bounds on impl blocks, not on the type definition. Write an impl by hand only when the derive would be wrong. Put the reason in a comment above it: `// do NOT derive Hash: the default hashes byte by byte and is slow`.

- **Private by default.** Keep fields private. Use `pub(crate)` for an item that other modules in the crate use. Use `pub` only for an item that other crates use. Give a type `new()` and derive `Default` when a no-argument value makes sense. Mark a `pub` enum or struct that another crate matches on, and that will grow, `#[non_exhaustive]`. A new variant is then not a breaking change. Do not add a builder to a domain type. Add a builder only to a config struct with many optional fields, and use a compile-time checked builder (see [CRATES.md](CRATES.md)).

- **Small traits.** Give a trait one capability, and build a ladder: `HasClock`, then `HasStore: HasClock`. A caller then asks for the weakest bound it needs. Name a bound bundle with a marker trait and a blanket impl: `pub trait Message: Send + Sync + 'static {}` plus `impl<T: Send + Sync + 'static> Message for T {}`. Add a method to a foreign type with an extension trait named `<Type>Ext`. Use generics. Use `dyn Trait` for a heterogeneous collection, or for a pluggable backend (see [RUNTIME.md](RUNTIME.md)). Implement `Deref` only on a smart pointer or an owning collection. Do not use `Deref` to reach the methods of a field. Write forwarding methods instead.

## Errors

Decide the error types with the domain types.

- **Rejection is an outcome. Failure is an error.** A domain method returns `Result<XOutcome, XError>`. `XOutcome` is an enum with a `Rejected(Reason)` variant for every expected "no": an unknown id, a full queue, a closed session. `Err` means an invariant is broken or a dependency failed. Do not put an expected rejection in the error enum. Do not log a rejection with `warn!`. A rejection is a response to the caller. Mark every outcome enum `#[must_use]` with a reason, so a caller cannot drop a `Rejected` silently. Skip the attribute on a private outcome with one caller: the one `match` already handles every case. Put in the outcome the facts the transition established: the removed children, the new expiry, the remaining count. A caller that looks the state up again after the call can get a different answer (see [INVARIANTS.md](INVARIANTS.md)). In a thin adapter or a script, where the caller only reports the rejection, return a plain `Result<T, E>`.

  ```rust
  #[must_use = "a rejection must reach the caller"]
  pub enum CancelOutcome { Cancelled { attempts: Retries }, Rejected(RejectReason) }

  impl Scheduler {
      pub fn cancel(&mut self, id: JobId) -> Result<CancelOutcome, SchedulerError> {
          let Some(job) = self.jobs.get_mut(&id) else {
              return Ok(CancelOutcome::Rejected(RejectReason::UnknownJob));
          };
  ```

- **Library errors use `thiserror`.** In a library crate, derive the error type with `#[derive(thiserror::Error, Debug)]`. Define one error enum per fallible surface. In a large crate, that is an operation or a module. In a small crate, it is the whole crate. A caller can then match on exactly the failures of the operation it called. Give each variant the data a caller needs to act on it. Examples: the id, the path, the offset, the expected and actual values. Do not format a message into a `String` field. Write the message with `#[error("...")]` and name the fields in it. Write the message in lowercase with no trailing period, as std does. A chained report then reads as one sentence per cause. Define a single-variant error as a struct, not as a one-variant enum. Derive `Copy` on an error that lives on a hot path, and keep `String` and `Box` out of it. An error at the edge or in a tool can carry a `String` path or a boxed cause.

- **Compose errors with `#[from]`, `#[source]`, and `transparent`.** Use `#[from]` on a variant when the inner error converts into this variant and nothing else. The `?` operator then converts it. One inner type can carry `#[from]` on only one variant of an enum. Use `#[source]` when the variant adds its own fields, or when the same inner type feeds two variants. `#[source]` keeps the cause chain and generates no `From`. Use `#[error(transparent)]` on a variant that only wraps another error. The wrapped message and cause chain then show through. Below, `JobNotFound` is a `Copy` struct error with its own `#[error]` message.

  ```rust
  #[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
  pub enum SchedulerError {
      #[error(transparent)]
      JobNotFound(#[from] JobNotFound),
      #[error("worker {0:?} is offline")]
      WorkerOffline(WorkerId),
  }
  ```

- **Translate errors at a layer boundary.** Write an exhaustive `From<InnerError> for OuterError` impl at each boundary. Name every variant. Do not write a `_` arm. A new inner variant then fails the build until the translation handles it.

- **A module-local `Result` alias is optional.** When a module defines one error enum and its fallible functions all return it, an alias shortens the signatures. Write `std::result::Result` (or `core::result::Result`) on the right side, so the alias does not name itself. Give the alias the visibility of its callers; `pub` only when the functions are `pub`. Keep the explicit `Result<T, E>` in three cases. The function is generic over the error. The module has several error types. An import of the alias would collide with std's. This is a readability convention. It is not a requirement on every error enum, it does not ask for one crate-wide error type, and a test keeps `anyhow::Result<()>`.

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

- **Application errors use `anyhow`.** In a binary crate and in tests, return `anyhow::Result<T>`. In a library crate, `anyhow` is a dev-dependency. Add context at each layer where the lower message is not enough on its own: `.context("load the service config")?` or `.with_context(|| format!("read {path}"))?`. Do not return `anyhow::Error` from a library function. Do not use `Box<dyn Error>`, `String`, or `()` as an error type in any signature. Do not mix the two conventions in one crate's public interface.

- **Propagate every error.** Use `?` everywhere a caller can act on the error. Do not write `let _ = fallible();` without a comment that says why the result does not matter. Do not convert an error to a log line and continue. The exception is an operation that is best-effort by design, with a comment that says so.

## Ownership

Decide the signatures.

- **Take what you use.** Take `&str`, `&[T]`, `&Path`, and `&T`. Take an owned value only when the function stores it or consumes it. Take `self` by value in a method on a `Copy` newtype.

- **Return by need.** Return `&str` or `&[T]` for a view of a field. Return `impl Iterator<Item = T>` when the caller consumes a sequence once, and `Vec<T>` when the caller keeps the collection. Do not box the iterator by default. Box it when the body returns one of several iterator types, or when a trait object stores it. Return `Cow<'_, str>` when a function returns its input unchanged in most calls and builds a new value in the rest.

- **Clone only for a second owner.** Do not add `.clone()` to make a borrow error go away. When the borrow checker rejects the code, use one of four moves. Shrink the borrow with an inner block. Move the value out with `mem::take` or `Option::take`. Split the struct so the fields borrow independently. Borrow in the signature. Write `Arc::clone(&x)` when you increment a reference count. Do not write `x.clone()` on an `Arc`.

- **Elide lifetimes.** Do not write a lifetime the compiler can infer. Name a lifetime only when the output borrows from one of several inputs, or when a struct stores a reference. Own data at a public boundary: use `Vec<u8>`, not `&'a [u8]`, in a public struct. Do not use `'static`, `Arc`, or `Rc` to escape a lifetime error.

- **Share by message.** Give state one owner and send it messages through a channel. Share a flag or a counter through an atomic, and a snapshot through `ArcSwap` (see [RUNTIME.md](RUNTIME.md)). A lock is the choice when one operation must read and write several fields together and no owner fits. Then hold it for the few lines that touch the shared state, with the preparation and the I/O outside. Do not return a guard from a method. On a hot path there is no lock and no structure that locks inside. [RUNTIME.md](RUNTIME.md) names the alternatives.

- **Time and randomness are inputs.** A state transition takes `now: Timestamp` as a parameter, and an `&mut impl Rng` when it needs randomness. Do not call `Instant::now()`, `SystemTime::now()`, or `thread_rng()` inside a state transition. The same inputs then replay the same state, and a test passes a constant. Read the clock and seed the generator at the edge, once per event. Use `Instant` for a duration. Use `SystemTime` only where a wall-clock time is recorded.

- **Rebind, then move.** Rebind a captured variable in a block around a `move` closure, and keep the outer name inside the closure. Do not write `let n2_cloned = n2.clone()`. Freeze prepared data with a rebinding. After the rebinding, the compiler rejects a later mutation.

  ```rust
  let on_tick = {
      let counter = Arc::clone(&counter);
      move || counter.fetch_add(1, Ordering::Relaxed)
  };
  let jobs = { let mut jobs = fetch_jobs(); jobs.sort_unstable(); jobs };
  ```

## Flow

Write the function body.

- **Transform, do not match.** Use `?`, `if let`, `while let`, `let ... else`, `map`, `map_err`, `ok_or_else`, `and_then`, `is_some_and`, `then_some`, and `matches!` on `Option` and `Result`. Write a `match` only when more than one arm binds a value. Chain bindings and conditions with `&&`: `if let Some(job) = slot && job.is_ready() && let Some(worker) = free_worker()` (edition 2024, Rust 1.88). One chain replaces a stack of nested `if let` blocks. Write `let Ok(x) = res else { return Ok(Outcome::Rejected(reason)) };` when the domain has already answered the error.

- **Guards first, happy path flat.** Put every guard clause at the top of the function. Write the reason in a one-line comment above each guard. Write one `let ... else` per binding that exits. Write a let chain for a guard that is a condition, not an exit. Keep the happy path at the lowest indentation. Build a value that has several exit points in a labeled block. Name the label for the thing you escape.

  ```rust
  let removed = 'queue: {
      let Some(worker) = self.workers.get_mut(&worker_id) else { break 'queue false };
      let Some(queue) = worker.queues.get_mut(&priority) else { break 'queue false };
      queue.remove(&job_id).is_some()
  };
  ```

- **Match exhaustively.** Name every variant of a local enum in a `match`. Do not write a `_` arm. A new variant then fails the build. Group arms with the same body with `|`. Put a policy that several consumers apply to one enum on the enum, as one method with an exhaustive `match`. A new variant then fails the build until the policy decides it, in one place. Write the invariant in the message of `unreachable!` and `debug_assert!`: `debug_assert!(!queue.is_empty(), "the map holds no empty queue")`.

- **Chain to build, loop to consume.** Build a new collection with an iterator chain and `collect()`. Use `for x in &v` when the loop body has side effects. Use `filter_map` and `find_map` where one closure can filter and map. Collect a fallible map with `collect::<Result<Vec<_>, E>>()?`. Do not `unwrap` inside the closure. Use std methods first. Use itertools for `tuple_windows`, `chunk_by`, `kmerge`, and `exactly_one` (see [CRATES.md](CRATES.md)).

- **A panic is a decision.** Use `unwrap`, `expect`, and indexing only in tests, in `main`, and at a site that cannot fail. At such a site, write `expect` with the invariant as the message. For an index, put the invariant in a comment within two lines. `unwrap` does not pass the check, so write the `expect`. Do not write `unreachable!`, a new error variant, or a `?` that cannot fail instead. Do not use `panic!` to handle an error. When you call `drop(x)` before the end of the scope, write a comment that says why.

- **Name every number.** Give a literal other than `0`, `1`, and `2` a name. Use a `const`, or an associated const on the type that owns the value: `Backoff::INITIAL`, `JobId::MAX`.

- **Check arithmetic on external values.** Use `checked_add`, `checked_mul`, and `saturating_sub` on values that come from the wire or from a client. Map an overflow to an error or a rejection. Use a plain operator only when a comment states the bound that makes it safe.

## Surface

Name the items and shape the public interface.

- **Imports at the top.** Import every non-std path at the top of the file. Do not write `use` inside a function. Do not write a qualified path such as `crate::clock::Timestamp` at a call site or in a signature. When two names collide, alias one: `use parking_lot::Mutex as SyncMutex;`. Macro paths and attribute paths are the exception.

- **Names follow the std conventions.** Name a conversion by cost and ownership: `as_` is free and borrowed, `to_` is expensive, `into_` consumes `self`. Name a getter after the field, without `get_`. Name a function for the decision it makes: `evict_stale_then_insert`. Use these suffixes with these meanings. `Outcome`: a domain result. `Kind`: a closed discriminant. `Reason`: why a request was rejected. `Config`: a configuration struct. `Handle`: a cloneable owner of a worker. `Ext`: an extension trait. `Has*`: a capability trait. Name a generic parameter with the initial of its concept (`W` for worker). Name a label for the thing it escapes (`'queue`). Name a closure parameter with the domain noun. Shadow a variable through a transformation: `let job = parse(job)?;`. Do not write `raw_job` and `parsed_job`. Use one name for one concept in a diff, and use the name the crate already uses.

- **`lib.rs` is a module list.** Write `lib.rs` as an alphabetized list of `pub mod`, `pub(crate) mod`, and `mod` lines. Put `#[cfg(test)]` and feature gates on the `mod` line. Use `pub use` only to export an item from a private module.

- **Attributes say why.** Write `#[must_use = "reason"]`. Do not write a bare `#[must_use]`. Write `#[expect(lint, reason = "...")]` on the one item that needs it. Do not write `#[allow]` at crate scope. Do not write `#![deny(warnings)]` in source. In the `// SAFETY:` comment above an `unsafe` block, name the obligation and say why it holds.

- **Bundle the arguments.** Keep a function at five positional parameters or fewer. Put more parameters in a request struct with named fields. Do not write `#[allow(clippy::too_many_arguments)]`. Give a complex closure type a trait.

## Words

Write the comments, the docs, the tests, and the logs.

- **Write in Simplified Technical English.** Write comments, doc comments, error messages, and log messages in ASD-STE100 style. Write one instruction or one fact per sentence. Keep an instruction at 20 words or fewer, and a description at 25 words or fewer. Use the active voice and the present tense. Use one word for one concept. Use the name the codebase uses: the type name, the function name, or the glossary entry. A glossary is `CONTEXT.md` or `GLOSSARY.md`, when one exists. Do not use a figure of speech, a synonym for variety, or a pronoun without a clear noun.

- **Comments say why.** Write a comment only when the code cannot show the reason, and write the reason in one line. Do not restate what the code does. Do not describe a past implementation. Do not write "we" or "this function". Keep a comment or a `TODO` that you did not write. Remove it only when the code it explains is gone or its work is done.

- **Docs state the rule and the consequence.** Start a doc comment with a one-line summary. Then state the invariant and what breaks when the invariant is violated: "Chosen once and fixed forever. A change to this seed changes every derived id and breaks replay." Do not restate the parameter types. Do not put design discussion in a doc comment. Link every item you mention with an intra-doc link: `` [`JobId`] ``. Write a module doc (`//!`) as an operating procedure, in a numbered list. Say how to add a variant, what may change and what must not, and what depends on the module. Write a variant doc that says when the variant occurs.

- **Tests read as sentences and show their arithmetic.** Put unit tests in a `#[cfg(test)] mod tests` block in the same file. Start every test name with `test_`, then the behavior it checks: `test_backoff_doubles_after_each_failure`. The prefix marks a test in a grep, a panic message, and a profile, where the `#[test]` attribute is not visible. Drop the prefix only in a crate that enables `clippy::redundant_test_prefix`. Return `anyhow::Result<()>` from the test so the body can use `?` on any error (see Errors). Derive the expected value in a comment above the assertion. Write `assert!(next > prev, "next ({next:?}) is above prev ({prev:?})")` with the values in the message. Use a named constant for time. Do not call `now()` in a test. Name a property test `test_prop_<behavior>`. Write one named case per input (see [CRATES.md](CRATES.md)). Add a test in the same crate for every new `pub fn`. When a stored type gains a stricter check, keep the old bytes in a test. Check that they load with their old meaning, and that a corrupt input is an error, not a default.

- **Logs are structured.** Pass values as key-value arguments. Do not format the message before the macro call. Do not log a rejection with `warn!`. Emit `info!` per lifecycle event. Do not emit `info!` per request or per message.

## Check

Before you hand Rust back, run the check command from [LINTS.md](LINTS.md) on each crate the diff touches. Then run `cargo +nightly fmt` on those crates. Fix each finding in a file the diff touches, or put `#[expect(lint, reason = "...")]` on the one item. A crate that never ran the check has a backlog in its other files. Report the count of that backlog and leave it.

## Pointers

- **You are deciding what a type guarantees, or reviewing a type that claims one.** Read [INVARIANTS.md](INVARIANTS.md).
- **The diff has async code, tasks, or threads.** Read [RUNTIME.md](RUNTIME.md).
- **You are about to write an impl by hand that a crate would derive.** Read [CRATES.md](CRATES.md).
- **You need the check command or the lint policy.** Read [LINTS.md](LINTS.md).
