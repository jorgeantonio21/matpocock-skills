# Crates

Read this file when you are about to write an impl by hand that a crate would derive. None of these crates is required, and no crate is banned. A crate is used when it removes hand-written code in the change you are making, and it is already a dependency or the pull request can justify it; a new dependency is a per-PR decision, and a single use does not justify one. This file instructs no migration: a project that uses a crate from "Choose by capability" keeps it.

## The set

- **`thiserror` 2 and `anyhow` 1.** `thiserror` for an error type in a library, `anyhow` in a binary and in tests; the rules are in the Errors section of [SKILL.md](SKILL.md). The `thiserror` expansion is the hand-written `Display`, `source`, and `From`, so it is hot-path safe.

- **`derive_more` 2.1.** `#[derive(Display, From, Into, FromStr)]` on an unconstrained newtype replaces four delegating impls, with one cargo feature per derive. `From` and `FromStr` inbound only when every inner value is valid: the `FromStr` derive parses the inner type and wraps it with no call to `new`, so on a validated newtype it accepts what `new` rejects. There, write `const fn new`, write `FromStr` by hand through `new`, and derive only `Display` and `Into`. The `Into` derive expands to `impl From<JobId> for u64`, the outbound `From` the Shape section asks for. `Deref` is not derived on a newtype. The expansion is the hand-written code, so it is hot-path safe.

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // std
  #[derive(Display, From, Into)] // derive_more
  pub struct JobId(u64);
  ```

- **`strum` 0.28.** `#[derive(VariantArray, EnumCount, EnumString, Display)]` on a unit enum replaces a hand-written `ALL` array, a `COUNT` const, and a match-table `FromStr`. `VariantArray` is a `'static` slice; `EnumIter` builds a small iterator struct on the stack, with no heap allocation, for a caller that wants an iterator. `ParseError` carries no payload, so it is hot-path safe.

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] // std
  #[derive(VariantArray, EnumCount, EnumString, Display)] // strum
  #[strum(serialize_all = "lowercase")]
  pub enum Priority { Low, Normal, High }
  // Priority::VARIANTS, Priority::COUNT, "high".parse::<Priority>()
  ```

- **`itertools` 0.15.** `tuple_windows` for adjacent pairs, `chunk_by` for runs in sorted input, `kmerge` for a k-way merge, `exactly_one` for an expected single result. A std method comes first where one exists: `slice::chunk_by`, `Iterator::is_sorted`, `inspect`. A typed `collect()`, not `collect_vec()`. The named adaptors are zero-cost; `sorted_*`, `unique`, `counts`, and `into_group_map` allocate, so they belong in tests and cold paths.

- **`tokio-util` 0.7.** `CancellationToken` for shutdown and `TaskTracker` for a set of workers at the async edge, as [RUNTIME.md](RUNTIME.md) describes; the token takes a mutex on every check, so a hot loop reads an atomic bridged from it. `codec::Framed` with a `Decoder` for length-prefixed framing at a wire edge. No proc macro.

  ```rust
  let token = CancellationToken::new();
  let tracker = TaskTracker::new();
  for worker in workers { tracker.spawn(run(worker, token.child_token())); }
  signal::ctrl_c().await?;
  token.cancel();
  tracker.close();
  tracker.wait().await;
  ```

- **`trait-variant` 0.1 and `dynosaur` 0.3.** `#[trait_variant::make(Send)]` on a trait with a native `async fn` when a spawn needs the returned future to be `Send`; `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` when a caller needs a trait object such as `Box<DynStore<'_>>`. The pair saves the box on the static path only (see "Choose by capability"), so it pays off in a crate that calls the trait statically on a path that matters. [RUNTIME.md](RUNTIME.md) says when a trait needs async at all.

  ```rust
  #[trait_variant::make(Send)]
  #[dynosaur::dynosaur(DynStore = dyn(box) Store)]
  pub trait Store {
      async fn put(&self, key: Key, value: Vec<u8>) -> Result<(), StoreError>;
  }
  ```

- **`bon` 3.10.** For a config struct with many optional fields that needs a builder: `#[bon::bon]` on the impl block and `#[builder]` on `new`, so a missing required member is a compile error. A fallible `new` becomes a fallible `build()`, so validation lives in `new` and the fields stay private. Compile time grows with the member count, so it is for config-sized structs; the runtime cost is zero.

  ```rust
  #[bon::bon]
  impl Server {
      #[builder]
      pub fn new(port: Port, #[builder(default = Duration::from_secs(5))] timeout: Duration) -> Self {
          Self { port, timeout }
      }
  }
  ```

- **`rstest` 0.26.** `#[rstest]` with one `#[case::name(..)]` per input row, so each case is its own named test, instead of a table looped inside one `#[test]`. `#[fixture] fn scheduler() -> Scheduler`, resolved by argument name, instead of a `fn setup()` called at the top of every test. Dev-dependency.

  ```rust
  #[rstest]
  #[case::empty("", None)]
  #[case::high("high", Some(Priority::High))]
  fn test_parses_priority(#[case] raw: &str, #[case] expected: Option<Priority>) {
      assert_eq!(raw.parse::<Priority>().ok(), expected);
  }
  ```

- **`pretty_assertions` 1.4.** `use pretty_assertions::assert_eq;` at the top of `mod tests`, so a failed `assert_eq!` prints a diff instead of two `Debug` dumps. Dev-dependency.

## One rule where two overlap

`derive_more::Display` for a type with a format string. `strum::Display` for a unit enum.

## Choose by capability

The crates below overlap with the set above. None is banned, the project's convention is kept, and no crate is migrated to follow this file. A new crate picks by the capability it needs.

- **Errors.** `thiserror` and `anyhow` are the default the Errors section assumes. `snafu` gives one convention for a library and an application, with context selectors in place of `.context()`. `eyre` and `color-eyre` are `anyhow` with a pluggable report handler, for a binary that wants a custom error report. `miette` renders diagnostics with source spans, for a compiler, a linter, or a CLI that points at a line of input. `displaydoc` takes the `Display` message from the doc comment. One convention per crate, and the project's.
- **Builders.** `bon` and `typed-builder` report a missing required member at compile time; `derive_builder` reports it at run time, from `build()`. A new builder is a compile-time checked one; the project's stays.
- **Validation.** `validator` and `garde` derive field checks behind a `.validate()` call and report every failed field at once, which is the product at an HTTP form boundary. A caller can skip the call and serde does not run it, so the check that establishes an invariant still lives in the type's constructor: the derive at the edge for the report, the constructor for the guarantee.
- **Async traits.** `#[async_trait]` boxes the returned future on every call, static or dynamic, and gives an object-safe trait with one attribute. Native `async fn` with `trait-variant` and `dynosaur` returns the future unboxed on the static path and boxes it, as `#[async_trait]` does, only through the trait object. Behind a `Box<dyn Trait>` seam the two cost the same, and `#[async_trait]` is one attribute where the pair is two. The project's choice stays.

## Keep to std

- `lazy_static`, `once_cell`: `std::sync::LazyLock` and `std::sync::OnceLock`, stable since Rust 1.80. On an older supported toolchain, `once_cell` stays.
- `static_assertions`: `const _: () = assert!(..);` for a predicate that const evaluation can decide, such as a size, an alignment, or a relation between two consts. It cannot state a trait bound; for one, write `const _: fn() = || { fn assert<T: Send>() {} assert::<Worker>() };`, or keep `assert_impl_all!` in a crate that already has the dependency.

## The cost of a proc macro

A proc macro runs at compile time and its expansion costs nothing at run time; the cost that can land on a hot path is in the code it emits. Run `cargo expand` once on a new derive and read the output for a `Box<dyn ...>`, a `format!`, a `clone`, or an allocation per call. A crate whose proc macro would put such code on a hot path needs a measured reason.

Every version above was checked against crates.io on 2026-09-04.
