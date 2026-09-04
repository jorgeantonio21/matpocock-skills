---
name: idiomatic-rust
description: Concise, idiomatic, readable Rust, each rule with its rewrite. Use when writing new Rust, refactoring existing Rust, reviewing a diff that touches .rs or Cargo.toml files, or when another skill needs the Rust idiom baseline.
---

# Idiomatic Rust

The rules a linter cannot enforce, each reading *what it looks like* → *the move*. Building, read the section for the work in hand before writing: Shape decides the types and errors, Ownership the signatures, Flow the body, Surface the names and imports, Words the comments, docs, and tests. Reviewing, match every entry against the diff and cite it on each finding. A documented repo standard overrides any entry. Whatever the check command in [LINTS.md](LINTS.md) enforces is deliberately absent here.

## Shape

The types and the errors, decided first.

- **Rejection is a value, failure is an `Err`.** One error enum mixing "the request was declined" with "the system is broken". → `Result<XOutcome, Error>` where `XOutcome::Rejected(Reason)` carries every expected no, and `Err` means an invariant is gone. A rejection is a response to the caller, never a `warn!`.

  ```rust
  pub enum CancelOutcome { Cancelled { size: Quantity }, Rejected(RejectReason) }

  pub fn cancel(&mut self, id: OrderId) -> Result<CancelOutcome, EngineError> {
      let Some(order) = self.orders.get_mut(id) else {
          return Ok(CancelOutcome::Rejected(RejectReason::UnknownOrder));
      };
  ```

- **The type carries the meaning.** A `bool` parameter, a sentinel (`-1`, an empty string), a field "only valid when" another is set, a `u64` that is really an id. → One-of is an enum, absent is `Option`, fallible is `Result`, a domain primitive is a newtype. `Widget::new(Size::Small, Shape::Round)`, never `Widget::new(true, false)`. A state machine is an enum with data, never a set of flags.

- **Errors by layer.** `Box<dyn Error>`, `Result<T, String>`, or `anyhow` in a library signature; a `thiserror` enum in `main`; both conventions in one crate. → `thiserror` in libraries and deterministic cores, `anyhow` with `.context()` at the application and test edge. Variants carry structured fields (the path, the offset, the id), never a pre-formatted `String`, so the error is debuggable without the log line around it. A single-variant error is a `Copy` struct; enums absorb it with `#[error(transparent)]` and `#[from]`. `#[source]` when the message is re-worded, `#[from]` when the conversion is the whole story. Layer boundaries translate with an exhaustive `From`, so a new variant fails the build.

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

- **Newtype anatomy.** `pub struct AccountId(pub u64)`; a type alias standing in for a type; `impl Display` written out by hand. → Private field, `const fn new` (fallible: `(v <= MAX).then_some(Self(v))`, plus a `const` panicking twin for literals), `From` both ways, `Display` and `FromStr` delegated to the inner type (derived, see [CRATES.md](CRATES.md)), sentinels as associated consts (`AccountId::MAX`), `#[repr(transparent)]` with a comment saying why the layout matters. Derives stacked one line per ecosystem with a trailing comment, so a reader sees which surfaces the type crosses; the comment is also what stops rustfmt merging the lines.

  ```rust
  /// A system-generated account identifier.
  #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)] // std
  #[derive(Serialize, Deserialize)] // serde
  #[derive(FromBytes, IntoBytes, KnownLayout, Immutable)] // zerocopy
  #[repr(transparent)] // wire layout is the inner u64
  pub struct AccountId(u64);
  ```

- **Derive eagerly, hand-write with a reason.** A public type without `Debug`; `impl Into<T>`; a hand-written `Default` a derive would produce. → `Debug` on every type; `Copy`, `Clone`, `Eq`, `Hash`, `Default` where valid; `From`, never `Into`. Trait bounds live on impl blocks, never on the type definition. A hand-written impl only where the derive would be wrong, with the reason in a comment above it (`// do NOT derive Hash: the default hashes byte by byte and is slow`).

- **Private by default.** `pub` fields on a struct with an invariant; a `pub fn` nothing outside the crate calls; a builder on a domain type. → Fields private, `pub(crate)` across modules, `new()` beside `Default`. A builder only on a config-shaped struct with many optional inputs, and then a compile-time-checked one ([CRATES.md](CRATES.md)).

- **Traits are small and layered.** A ten-method trait; `Box<dyn Trait>` where every caller knows the type; `Deref` to reach a field's methods. → One capability per trait (`HasPosition`, then `HasOpenOrders: HasPosition`); a marker trait with a blanket impl to name a bound bundle; an extension trait (`AccountExt`) to add a method to a foreign type. Generics until the collection is heterogeneous. `Deref` only on smart pointers and owning collections; forwarding methods everywhere else.

## Ownership

Signatures and borrows.

- **Take what you use.** `&String`, `&Vec<T>`, `&Box<T>`, `&PathBuf` in a signature; an owned `String` the body only reads; `&self` on a `Copy` newtype. → `&str`, `&[T]`, `&T`, `&Path`; owned only when the function stores or consumes it; `self` by value on `Copy` types; `impl IntoIterator<Item = T>` when the body only iterates.

- **A clone is for a second owner.** `.clone()` added to make a borrow error go away; a whole collection cloned to read it; `x.clone()` on an `Arc`. → Shrink the borrow's scope with an inner block, move the value out with `mem::take` or `Option::take`, split the struct so fields borrow independently, or borrow in the signature. `Arc::clone(&x)` spelled out where a count bump is the intent.

- **Elide.** `'a` on a function with one reference input; `'static` or `Arc` reached for to escape a lifetime; a lifetime threaded through a public struct. → Name a lifetime only when the output borrows from one of several inputs or a struct stores a reference. Own at a public boundary (`Vec<u8>`, not `&'a [u8]`) and let the compiler elide the rest.

- **Share by message.** `Arc<Mutex<T>>` as the first answer to sharing; a guard held across `.await`; a `MutexGuard` returned from a method. → A channel by role ([RUNTIME.md](RUNTIME.md)). When a lock is unavoidable, its scope is a few lines and the guard never leaves them.

- **Rebind, then move.** `let n2_cloned = n2.clone(); move || use(n2_cloned)`; a value prepared with `let mut` and mutable ever after. → Rebind inside a block around the closure so the body keeps the outer names; freeze prepared data with `let data = data;` or an inner block.

  ```rust
  let on_tick = {
      let counter = Arc::clone(&counter);
      move || counter.fetch_add(1, Ordering::Relaxed)
  };
  let levels = { let mut levels = fetch_levels(); levels.sort_unstable(); levels };
  ```

## Flow

Inside a function.

- **Transform, don't match.** `match opt { Some(x) => x, None => return }`; `if x.is_some() { x.unwrap() }`; a `match` on a `Result` whose `Err` arm only returns. → `?`, `if let`, `while let`, `let...else`, `map`, `map_err`, `ok_or_else`, `and_then`, `is_some_and`, `then_some`, `matches!`. A `match` earns its place by binding in more than one arm. `let Ok(x) = fallible() else { return Ok(Outcome::Rejected(reason)) };` discards an error the domain has already answered.

- **Guards first, happy path flat.** A pyramid of nested `if let`; a helper extracted only to get several early exits; a bare `return` with no reason. → Guard clauses at the top with the reason in a one-line comment, the happy path unindented. A value with several exits is a labeled block, the label named for the thing escaped.

  ```rust
  let cancelled_on_book = 'book: {
      let Some(portfolio) = self.portfolios.get_mut(id) else { break 'book false };
      let Some(exposure) = portfolio.markets.get_mut(market) else { break 'book false };
      exposure.open_orders.cancel(request_id).is_some()
  };
  ```

- **Exhaustive with `|`.** `_ =>` on a local enum; identical arms written twice. → Every variant named, arms with the same body grouped with `|`, so adding a variant fails the build. `unreachable!("reason")` and `debug_assert!(cond, "invariant")` carry the invariant in their message.

- **Chain to build, loop to consume.** `for i in 0..v.len()` with `v[i]`; `.filter().map()` that could fuse; `.collect()` mid-chain; `.for_each` with a block body; `fold` threading an accumulator tuple. → An iterator chain for a fresh collection; `for x in &v` when the body has side effects; `filter_map`, `find_map`; `collect::<Result<Vec<_>, E>>()?` for a fallible map; `sum`, `any`, `find` over `for_each`. std first; itertools for `tuple_windows`, `chunk_by`, `kmerge`, `exactly_one` ([CRATES.md](CRATES.md)).

- **A panic is a decision.** `.unwrap()` on I/O, parsing, or a map lookup; `let _ = fallible();`; `panic!` as error handling. → `?` everywhere a caller can act. `unwrap`, `expect`, and indexing live in tests, `main`, and provably infallible sites, with the invariant in the `expect` message or a comment within two lines. A discarded `Result` carries a comment saying why ignoring it is safe; an explicit `drop(guard)` says why it is early.

- **Numbers have names.** `86_400`, `1 << 3`, `0.0001` inline in an argument or a comparison. → A `const` named for its meaning, or an associated const on the type that owns the noun (`PriceOfAtom::ONE_CENT`, `AccountId::MAX`). Only `0`, `1`, and `2` stay literal.

- **Arithmetic is checked.** `a + b` on values from the wire or from a client; `x / y` with `y` from input. → `checked_add`, `checked_mul`, `saturating_sub`, with the overflow mapped to an error or a rejection; plain operators only where a comment states the bound that makes them safe.

## Surface

Names, imports, and the public API.

- **Imports at the top.** `use` inside a function; `crate::time::TimeStamp` spelled out at a call site or in a signature. → Every non-std path imported at the top of the file. On a collision, alias: `use parking_lot::Mutex as SyncMutex;`. Macro and attribute paths are the exception.

- **Names by convention.** `get_first()`; `to_` on a cheap borrow; `UUID`; `data`, `utils`, `manager`; `SnapShot` beside `Snapshot`; `replace_balance(amount)`. → `as_` free and borrowed, `to_` expensive, `into_` consuming; getters without `get_`; `iter`, `iter_mut`, `into_iter`; acronyms as words (`Uuid`). Functions name the decision (`cancel_old_then_reject`, `drain_then_cancel`). Suffixes carry meaning: `Outcome`, `Kind`, `Reason`, `Config`, `Handle`, `Ext`, `Has*`, `*For<T>`. Generic letters are initials (`E` for exposure, `H` for header); labels name the thing escaped (`'book`, `'sweep`); closure parameters are the domain noun. Shadow through a transformation (`let order = parse(order)?`), never `raw_order` and `parsed_order`. One name per concept within a diff, matching the crate's existing name for it.

- **`lib.rs` is a module list.** `pub use` re-exporting everything; a `prelude`; `use foo::*`; logic inside `mod.rs`. → A visibility-graded `pub mod` / `pub(crate) mod` list, alphabetised; `pub use` only rescues an item from a private module; `#[cfg(test)]` and feature gates on the `mod` line. Glob imports only as `use super::*` in tests.

- **Attributes say why.** A bare `#[must_use]`; `#[allow(dead_code)]` at crate scope; `#![deny(warnings)]`; `unsafe {}` with nothing above it. → `#[must_use = "None means the tree was full"]`; `#[expect(lint, reason = "...")]` on the one item; `-D warnings` in CI, never in source; `// SAFETY:` above every `unsafe` block, discharging the contract by name.

- **Bundle the arguments.** `#[allow(clippy::too_many_arguments)]`; six positional parameters; a closure type explained with `/* name */` comments. → A request struct with named fields; a trait for the closure's shape. Five positional parameters is the ceiling.

## Words

Comments, docs, tests, logs.

- **Comments are one line and say why.** A paragraph restating what the code does; "We chose this approach because"; "Previously this used a HashMap"; a comment opening with "we" or "this function". → One line, the reason, present tense. The what is the code's job. A comment or `TODO` you did not write stays unless the code it explains is gone or its work is done.

- **Docs state the rule and the consequence.** `/// Returns the account id.`; parameter types restated; design reasoning that belongs in a PR. → A one-line summary, then the invariant and what breaks if it is violated: "Chosen once and fixed forever: changing it changes every derived id, breaking replay." `# Errors`, `# Panics`, `# Safety` where they apply; an intra-doc link for every item mentioned. A module doc carries the operating procedure: how to add a variant, when a number may change, what depends on it.

- **Tests read as sentences and show their arithmetic.** `test_partial_match`; a bare `assert_eq!(x, 1_010_000_000)`; a table looped inside one test; `now()` in a test. → In-file `mod tests`; names that state the behaviour (`micro_price_leans_toward_thinner_side`); the crate's `Result` alias as the return type so `?` works; the expected value derived in the comment above the assertion; `assert!(cond, "micro ({micro:?}) should exceed mid ({mid:?})")`; one-letter local constructors (`p(..)`, `q(..)`); a named constant for time; `prop_` for property tests; one named case per input ([CRATES.md](CRATES.md)). A new `pub fn` lands with a test in the same crate.

- **Logs are structured.** `info!("processed {}", format!(..))`; `error!` on a declined request; `info!` per message. → Key-value arguments the macro formats lazily; a rejected request is a response, never a `warn!`; `info!` fires per cluster event, not per order.

## Check

Before handing Rust back, run the check command from [LINTS.md](LINTS.md) on the crates the diff touches, then `cargo +nightly fmt` on them. Findings in the files the diff touches are yours: fix each one, or put `#[expect(lint, reason = "...")]` on the one item. A crate that never ran the check carries a backlog in its other files; report the count and leave it.

## Pointers

- **Async, tasks, or threads in the diff** → read [RUNTIME.md](RUNTIME.md).
- **About to hand-write an impl a crate would derive** → read [CRATES.md](CRATES.md).
- **The check command and the lint policy** → read [LINTS.md](LINTS.md).
