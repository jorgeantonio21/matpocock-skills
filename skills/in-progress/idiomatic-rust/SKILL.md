---
name: idiomatic-rust
description: Concise, idiomatic, readable Rust, each rule as an instruction with its reason. Use when writing new Rust, refactoring existing Rust, reviewing a diff that touches .rs or Cargo.toml files, or when another skill needs the Rust idiom baseline.
---

# Idiomatic Rust

This file holds the Rust rules that a linter cannot enforce. Each rule is an instruction. A reason follows the instruction where the reason is not obvious. Follow every rule when you write or change Rust. When you review Rust, compare each changed function against every rule and name the rule on each finding. A documented standard in the repository overrides this file. The check command in [LINTS.md](LINTS.md) enforces the mechanical rules, so this file does not repeat them.

## Shape

Decide the types before you write a function body.

- **The compiler is the guardrail.** Give each kind of value its own type: `Price`, `Quantity`, `AccountId`, `PortfolioId`. Do not use a bare `u64`, `f64`, `String`, or `bool` for a domain value. A function that takes two parameters of the same primitive type is a defect: the caller can swap the arguments and the compiler accepts the call. When two parameters have the same kind and different roles, put them in a struct with named fields. Put a unit in the type where a unit exists: `CentiCents(u64)`, `std::time::Duration`.

  ```rust
  pub struct Price(u64);
  pub struct Quantity(u64);
  pub struct Transfer { pub from: AccountId, pub to: AccountId, pub amount: Quantity }

  fn fill(price: Price, quantity: Quantity) -> Fill { /* .. */ }
  // fill(quantity, price) does not compile.
  ```

- **Make invalid states impossible.** Model "one of several" as an enum. Model "may be absent" as `Option`. Model "can fail" as `Result`. Do not pass a `bool` argument. Use a two-variant enum such as `Side::Buy` and `Side::Sell`. Do not use a sentinel value such as `-1` or an empty string. When a field is valid only while another field has a given value, merge both fields into one enum with data. Model a state machine as an enum, not as a set of flags. When a method is valid only in one phase of a value's life, put the phase in a type parameter: `Connection<Handshaking>` offers `complete()`, and `Connection<Ready>` offers `send()`. Use `NonZeroU64` for a count that is never zero.

- **Parse at the boundary.** Convert raw input (bytes, JSON, a query string, a config file) into domain types at the edge of the system, once, in one place. Make the constructor of a validated newtype the only way to create a value, and keep the field private. Every value inside the core is then valid, and the core never checks it again. Do not pass a raw `u64` or `String` into the core.

- **Rejection is an outcome. Failure is an error.** A domain method returns `Result<XOutcome, XError>`. `XOutcome` is an enum with a `Rejected(Reason)` variant for every expected "no": an unknown order, an insufficient balance, a closed market. `Err` means an invariant is broken or a dependency failed. Do not put an expected rejection in the error enum. Do not log a rejection with `warn!`. A rejection is a response to the caller.

  ```rust
  pub enum CancelOutcome { Cancelled { size: Quantity }, Rejected(RejectReason) }

  pub fn cancel(&mut self, id: OrderId) -> Result<CancelOutcome, EngineError> {
      let Some(order) = self.orders.get_mut(id) else {
          return Ok(CancelOutcome::Rejected(RejectReason::UnknownOrder));
      };
  ```

- **Newtype anatomy.** Keep the inner field private. Write `const fn new`. When the value has a bound, return `Option<Self>` from `new` with `(v <= MAX).then_some(Self(v))`, and add a `const fn new_const` that panics, for literals. Implement `From` in both directions. Derive `Display` and `FromStr` so they delegate to the inner type (see [CRATES.md](CRATES.md)). Put sentinel values in associated consts: `AccountId::MAX`, `PriceOfAtom::ZERO`. Add `#[repr(transparent)]` when the layout matters, with a comment that says why. Write one `#[derive(...)]` line per ecosystem with a trailing comment that names the ecosystem. The comment shows a reader which surfaces the type crosses, and it stops rustfmt from merging the lines.

  ```rust
  /// A system-generated account identifier.
  #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)] // std
  #[derive(Serialize, Deserialize)] // serde
  #[derive(FromBytes, IntoBytes, KnownLayout, Immutable)] // zerocopy
  #[repr(transparent)] // wire layout is the inner u64
  pub struct AccountId(u64);
  ```

- **Derive, or explain.** Derive `Debug` on every type. Derive `Copy`, `Clone`, `PartialEq`, `Eq`, `Hash`, and `Default` where they are valid. Implement `From`. Do not implement `Into`. Put trait bounds on impl blocks, not on the type definition. Write an impl by hand only when the derive would be wrong, and put the reason in a comment above it: `// do NOT derive Hash: the default hashes byte by byte and is slow`.

- **Private by default.** Keep fields private. Use `pub(crate)` for an item that other modules in the crate use. Use `pub` only for an item that other crates use. Give a type `new()` and derive `Default` when a no-argument value makes sense. Do not add a builder to a domain type. Add a builder only to a config struct with many optional fields, and use a compile-time checked builder (see [CRATES.md](CRATES.md)).

- **Small traits.** Give a trait one capability, and build a ladder: `HasPosition`, then `HasOpenOrders: HasPosition`. A caller then asks for the weakest bound it needs. Name a bound bundle with a marker trait and a blanket impl: `pub trait BodyLayout: IntoBytes + KnownLayout + Copy {}` plus `impl<T: IntoBytes + KnownLayout + Copy> BodyLayout for T {}`. Add a method to a foreign type with an extension trait named `<Type>Ext`. Use generics. Use `dyn Trait` only for a heterogeneous collection. Implement `Deref` only on a smart pointer or an owning collection. Do not use `Deref` to reach the methods of a field. Write forwarding methods instead.

## Errors

Decide the error types with the domain types.

- **Library errors use `thiserror`.** In a library crate, derive the error type with `#[derive(thiserror::Error, Debug)]`. Define one error enum per fallible operation or module, not one enum for the whole crate. A caller can then match on exactly the failures of the operation it called. Give each variant the data a caller needs to act on it: the id, the path, the offset, the expected and actual values. Do not format a message into a `String` field. Write the message with `#[error("...")]` and name the fields in it. Define a single-variant error as a struct, not as a one-variant enum. Derive `Copy` on an error that lives on a hot path, and keep `String` and `Box` out of it.

- **Compose errors with `#[from]`, `#[source]`, and `transparent`.** Use `#[from]` on a variant when the inner error converts into this variant and nothing else. The `?` operator then converts it. One inner type can carry `#[from]` on only one variant of an enum. Use `#[source]` when the variant adds its own fields, or when the same inner type feeds two variants. `#[source]` keeps the cause chain and generates no `From`. Use `#[error(transparent)]` on a variant that only wraps another error. The wrapped message and cause chain then show through.

  ```rust
  #[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
  #[error("account {0:?} not found")]
  pub struct AccountNotFound(pub AccountId);

  #[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
  pub enum EngineError {
      #[error(transparent)]
      AccountNotFound(#[from] AccountNotFound),
      #[error("mark price for {0:?} is stale")]
      StaleMarkPrice(Market),
  }
  ```

- **Translate errors at a layer boundary.** Write an exhaustive `From<InnerError> for OuterError` impl at each boundary. Name every variant. Do not write a `_` arm. A new inner variant then fails the build until the translation handles it.

- **Application errors use `anyhow`.** In a binary crate and in tests, return `anyhow::Result<T>`. Add context at each layer where the lower message is not enough on its own: `.context("load gateway config")?` or `.with_context(|| format!("read {path}"))?`. Do not return `anyhow::Error` from a library function. Do not use `Box<dyn Error>`, `String`, or `()` as an error type in any signature. Do not mix the two conventions in one crate's public interface.

- **Never swallow an error.** Propagate with `?` everywhere a caller can act on the error. Do not write `let _ = fallible();` without a comment that says why the result does not matter. Do not convert an error to a log line and continue, unless the operation is best-effort by design and a comment says so.

## Ownership

Decide the signatures.

- **Take what you use.** Take `&str`, `&[T]`, `&Path`, and `&T`. Do not take `&String`, `&Vec<T>`, `&PathBuf`, or `&Box<T>`. Take an owned value only when the function stores it or consumes it. Take `self` by value in a method on a `Copy` newtype. Take `impl IntoIterator<Item = T>` when the function only iterates.

- **Clone only for a second owner.** Do not add `.clone()` to make a borrow error go away. When the borrow checker rejects the code, do one of these: shrink the borrow with an inner block, move the value out with `mem::take` or `Option::take`, split the struct so the fields borrow independently, or borrow in the signature. Write `Arc::clone(&x)` when you increment a reference count. Do not write `x.clone()` on an `Arc`.

- **Elide lifetimes.** Do not write a lifetime the compiler can infer. Name a lifetime only when the output borrows from one of several inputs, or when a struct stores a reference. Own data at a public boundary: use `Vec<u8>`, not `&'a [u8]`, in a public struct. Do not use `'static`, `Arc`, or `Rc` to escape a lifetime error.

- **Share by message.** Send data between tasks through a channel (see [RUNTIME.md](RUNTIME.md)). Do not reach for `Arc<Mutex<T>>` first. When a lock is necessary, hold it for a few lines. Do not hold it across `.await`. Do not return a guard from a method.

- **Rebind, then move.** Rebind a captured variable in a block around a `move` closure, and keep the outer name inside the closure. Do not write `let n2_cloned = n2.clone()`. Freeze prepared data with a rebinding. After the rebinding, the compiler rejects a later mutation.

  ```rust
  let on_tick = {
      let counter = Arc::clone(&counter);
      move || counter.fetch_add(1, Ordering::Relaxed)
  };
  let levels = { let mut levels = fetch_levels(); levels.sort_unstable(); levels };
  ```

## Flow

Write the function body.

- **Transform, do not match.** Use `?`, `if let`, `while let`, `let ... else`, `map`, `map_err`, `ok_or_else`, `and_then`, `is_some_and`, `then_some`, and `matches!` on `Option` and `Result`. Write a `match` only when more than one arm binds a value. Write `let Some(x) = opt else { return ... };` for an early return. Write `let Ok(x) = res else { return Ok(Outcome::Rejected(reason)) };` when the domain has already answered the error.

- **Guards first, happy path flat.** Put every guard clause at the top of the function, with the reason in a one-line comment above it. Keep the happy path at the lowest indentation. Build a value that has several exit points in a labeled block, and name the label for the thing you escape. Do not extract a helper function only to get early returns.

  ```rust
  let cancelled_on_book = 'book: {
      let Some(portfolio) = self.portfolios.get_mut(id) else { break 'book false };
      let Some(exposure) = portfolio.markets.get_mut(market) else { break 'book false };
      exposure.open_orders.cancel(request_id).is_some()
  };
  ```

- **Match exhaustively.** Name every variant of a local enum in a `match`. Do not write a `_` arm. A new variant then fails the build. Group arms with the same body with `|`. Write the invariant in the message of `unreachable!` and `debug_assert!`: `debug_assert!(!exposure.is_empty(), "sparse map holds no empty exposure")`.

- **Chain to build, loop to consume.** Build a new collection with an iterator chain and `collect()`. Use `for x in &v` when the loop body has side effects. Use `filter_map` and `find_map` where one closure can filter and map. Collect a fallible map with `collect::<Result<Vec<_>, E>>()?`. Do not `unwrap` inside the closure. Use `sum`, `any`, and `find`. Do not use `for_each` with a block body. Do not write `for i in 0..v.len()` with `v[i]`. Use std methods first. Use itertools for `tuple_windows`, `chunk_by`, `kmerge`, and `exactly_one` (see [CRATES.md](CRATES.md)).

- **A panic is a decision.** Use `?` everywhere a caller can act on the error. Use `unwrap`, `expect`, and indexing only in tests, in `main`, and at a site that cannot fail. At such a site, write the invariant in the `expect` message or in a comment within two lines. Do not use `panic!` to handle an error. When you call `drop(x)` before the end of the scope, write a comment that says why.

- **Name every number.** Give a literal other than `0`, `1`, and `2` a name. Use a `const`, or an associated const on the type that owns the value: `PriceOfAtom::ONE_CENT`, `AccountId::MAX`.

- **Check arithmetic on external values.** Use `checked_add`, `checked_mul`, and `saturating_sub` on values that come from the wire or from a client. Map an overflow to an error or a rejection. Use a plain operator only when a comment states the bound that makes it safe.

## Surface

Name the items and shape the public interface.

- **Imports at the top.** Import every non-std path at the top of the file. Do not write `use` inside a function. Do not write a qualified path such as `crate::time::TimeStamp` at a call site or in a signature. When two names collide, alias one: `use parking_lot::Mutex as SyncMutex;`. Macro paths and attribute paths are the exception.

- **Names follow the std conventions.** Name a conversion by cost and ownership: `as_` is free and borrowed, `to_` is expensive, `into_` consumes `self`. Name a getter after the field, without `get_`. Write an acronym as one word: `Uuid`. Name a function for the decision it makes: `cancel_old_then_reject`. Use the suffixes with these meanings: `Outcome` for a domain result, `Kind` for a closed discriminant, `Reason` for why a request was rejected, `Config` for a configuration struct, `Handle` for a cloneable owner of a worker, `Ext` for an extension trait, `Has*` for a capability trait, `*For<T>` for a relation trait. Name a generic parameter with the initial of its concept (`E` for exposure). Name a label for the thing it escapes (`'book`). Name a closure parameter with the domain noun. Shadow a variable through a transformation: `let order = parse(order)?;`. Do not write `raw_order` and `parsed_order`. Use one name for one concept in a diff, and use the name the crate already uses.

- **`lib.rs` is a module list.** Write `lib.rs` as an alphabetized list of `pub mod`, `pub(crate) mod`, and `mod` lines. Put `#[cfg(test)]` and feature gates on the `mod` line. Use `pub use` only to export an item from a private module. Do not write a `prelude` module. Do not write `use foo::*`. The exception is `use super::*` in a test module.

- **Attributes say why.** Write `#[must_use = "reason"]`. Do not write a bare `#[must_use]`. Write `#[expect(lint, reason = "...")]` on the one item that needs it. Do not write `#[allow]` at crate scope. Do not write `#![deny(warnings)]` in source. Write `// SAFETY:` above every `unsafe` block, name the obligation, and say why it holds.

- **Bundle the arguments.** Keep a function at five positional parameters or fewer. Put more parameters in a request struct with named fields. Do not write `#[allow(clippy::too_many_arguments)]`. Give a complex closure type a trait. Do not describe its parameters with `/* name */` comments.

## Words

Write the comments, the docs, the tests, and the logs.

- **Write in Simplified Technical English.** Write comments, doc comments, error messages, and log messages in ASD-STE100 style. Write one instruction or one fact per sentence. Keep a sentence at 20 words or fewer. Use the active voice and the present tense. Use one word for one concept, and use the name the codebase uses for it: the type name, the function name, or the entry in the repository's glossary (`CONTEXT.md` or `GLOSSARY.md`) when one exists. Do not use a figure of speech, a synonym for variety, or a pronoun without a clear noun.

- **Comments say why.** Write a comment only when the code cannot show the reason, and write the reason in one line. Do not restate what the code does. Do not describe a past implementation. Do not write "we" or "this function". Keep a comment or a `TODO` that you did not write. Remove it only when the code it explains is gone or its work is done.

- **Docs state the rule and the consequence.** Start a doc comment with a one-line summary. Then state the invariant and what breaks when the invariant is violated: "Chosen once and fixed forever. A change to this key changes every derived id and breaks replay." Do not restate the parameter types. Do not put design discussion in a doc comment. Add `# Errors`, `# Panics`, and `# Safety` sections where they apply. Link every item you mention with an intra-doc link: `` [`AccountId`] ``. Write a module doc (`//!`) as an operating procedure: say how to add a variant, when a number may change, and what depends on the module, in a numbered list. Write a variant doc that says when the variant occurs.

- **Tests read as sentences and show their arithmetic.** Put unit tests in a `#[cfg(test)] mod tests` block in the same file. Name a test for the behavior it checks: `micro_price_leans_toward_thinner_side`. Do not prefix it with `test_`. Return the crate's `Result` alias from the test so the body can use `?`. Derive the expected value in a comment above the assertion. Write `assert!(cond, "micro ({micro:?}) is above mid ({mid:?})")` with the values in the message. Define one-letter constructors for domain values in the test module: `p(..)` for a price, `q(..)` for a quantity. Use a named constant for time. Do not call `now()` in a test. Prefix a property test with `prop_`. Write one named case per input (see [CRATES.md](CRATES.md)). Add a test in the same crate for every new `pub fn`.

- **Logs are structured.** Pass values as key-value arguments. Do not format the message before the macro call. Do not log a rejection with `warn!`. Emit `info!` per cluster event. Do not emit `info!` per order or per message.

## Check

Before you hand Rust back, run the check command from [LINTS.md](LINTS.md) on each crate the diff touches. Then run `cargo +nightly fmt` on those crates. Fix each finding in a file the diff touches, or put `#[expect(lint, reason = "...")]` on the one item. A crate that never ran the check has a backlog in its other files. Report the count of that backlog and leave it.

## Pointers

- **The diff has async code, tasks, or threads.** Read [RUNTIME.md](RUNTIME.md).
- **You are about to write an impl by hand that a crate would derive.** Read [CRATES.md](CRATES.md).
- **You need the check command or the lint policy.** Read [LINTS.md](LINTS.md).
