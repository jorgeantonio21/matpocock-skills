# Crates

Read this file when you are about to write an impl by hand that a crate would derive. The list is short on purpose. None of these crates is required, and no crate is banned. Use one only when it removes hand-written code in the change you are making. Use it only when the crate is already a dependency, or when the pull request can justify it. Do not add a crate for a single use. Write the code instead. A new dependency is a per-PR decision. This file does not instruct a migration: a project that uses a crate from "Choose by capability" keeps it.

## The set

- **`thiserror` 2 and `anyhow` 1.** Use `thiserror` for an error type in a library. Use `anyhow` in a binary and in tests. The rules are in the Errors section of [SKILL.md](SKILL.md). The `thiserror` expansion is the hand-written `Display`, `source`, and `From`, so it is hot-path safe.

- **`derive_more` 2.1.** Use `#[derive(Display, From, Into, FromStr)]` on an unconstrained newtype instead of four delegating impls. Enable one cargo feature per derive. Derive `From` and `FromStr` inbound only when every inner value is valid. The `FromStr` derive parses the inner type and wraps it, with no call to `new`, so on a validated newtype it accepts what `new` rejects. There, write `const fn new`, write `FromStr` by hand through `new`, and derive only `Display` and `Into`. The `Into` derive expands to `impl From<JobId> for u64`, which is the outbound `From` the Shape section asks for; no `Into` impl is written. Do not derive `Deref` on a newtype. The expansion is the hand-written code, so it is hot-path safe.

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // std
  #[derive(Display, From, Into)] // derive_more
  pub struct JobId(u64);
  ```

- **`strum` 0.28.** Use `#[derive(VariantArray, EnumCount, EnumString, Display)]` on a unit enum instead of a hand-written `ALL` array, a `COUNT` const, or a match-table `FromStr`. Use `VariantArray`, which is a `'static` slice. Do not use `EnumIter`, which allocates an iterator struct. `ParseError` carries no payload, so it is hot-path safe.

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] // std
  #[derive(VariantArray, EnumCount, EnumString, Display)] // strum
  #[strum(serialize_all = "lowercase")]
  pub enum Priority { Low, Normal, High }
  // Priority::VARIANTS, Priority::COUNT, "high".parse::<Priority>()
  ```

- **`itertools` 0.15.** Use `tuple_windows` for adjacent pairs, `chunk_by` for runs in sorted input, `kmerge` for a k-way merge, and `exactly_one` for an expected single result. Use a std method first where one exists: `slice::chunk_by`, `Iterator::is_sorted`, `inspect`. Write a typed `collect()`. Do not use `collect_vec()`. The named adaptors are zero-cost. `sorted_*`, `unique`, `counts`, and `into_group_map` allocate, so use them in tests and cold paths only.

- **`tokio-util` 0.7.** Use `CancellationToken` for shutdown and `TaskTracker` for a set of workers at the async edge, as [RUNTIME.md](RUNTIME.md) describes. The token takes a mutex on every check, so bridge it to an atomic for a hot loop. Use `codec::Framed` with a `Decoder` for length-prefixed framing at a wire edge. No proc macro.

  ```rust
  let token = CancellationToken::new();
  let tracker = TaskTracker::new();
  for worker in workers { tracker.spawn(run(worker, token.child_token())); }
  signal::ctrl_c().await?;
  token.cancel();
  tracker.close();
  tracker.wait().await;
  ```

- **`trait-variant` 0.1 and `dynosaur` 0.3.** Use `#[trait_variant::make(Send)]` on a trait with a native `async fn` when a spawn needs the returned future to be `Send`. Use `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` on the trait when a caller needs a trait object such as `Box<DynStore<'_>>`. Static dispatch through the native `async fn` returns the future unboxed. The trait object boxes each returned future, as `#[async_trait]` does. The pair saves the box on the static path only. It pays off in a crate that calls the trait statically on a path that matters. See [RUNTIME.md](RUNTIME.md) for when a trait needs async at all.

  ```rust
  #[trait_variant::make(Send)]
  #[dynosaur::dynosaur(DynStore = dyn(box) Store)]
  pub trait Store {
      async fn put(&self, key: Key, value: Vec<u8>) -> Result<(), StoreError>;
  }
  ```

- **`bon` 3.10.** Use it only when a config struct has many optional fields and needs a builder. Put `#[bon::bon]` on the impl block and `#[builder]` on `new`. A missing required member is then a compile error. A fallible `new` becomes a fallible `build()`, so validation lives in `new` and the fields stay private. Compile time grows with the member count, so use it for config-sized structs only. The runtime cost is zero.

  ```rust
  #[bon::bon]
  impl Server {
      #[builder]
      pub fn new(port: Port, #[builder(default = Duration::from_secs(5))] timeout: Duration) -> Self {
          Self { port, timeout }
      }
  }
  ```

- **`rstest` 0.26.** Use `#[rstest]` with one `#[case::name(..)]` per input row instead of a table looped inside one `#[test]`. Each case is then its own named test. Use `#[fixture] fn scheduler() -> Scheduler`, which rstest resolves by argument name, instead of a `fn setup()` called at the top of every test. Dev-dependency.

  ```rust
  #[rstest]
  #[case::empty("", None)]
  #[case::high("high", Some(Priority::High))]
  fn test_parses_priority(#[case] raw: &str, #[case] expected: Option<Priority>) {
      assert_eq!(raw.parse::<Priority>().ok(), expected);
  }
  ```

- **`pretty_assertions` 1.4.** Write `use pretty_assertions::assert_eq;` at the top of `mod tests`. A failed `assert_eq!` then prints a diff instead of two `Debug` dumps. Dev-dependency.

## One rule where two overlap

Use `derive_more::Display` for a type with a format string. Use `strum::Display` for a unit enum.

## Choose by capability

The crates below overlap with the set above. None is banned. Keep the convention the project has, and do not migrate a crate to follow this file. Pick for a new crate by the capability the project needs.

- **Errors.** `thiserror` and `anyhow` are the default the Errors section assumes. `snafu` gives one convention for a library and an application, with context selectors in place of `.context()`. `eyre` and `color-eyre` are `anyhow` with a pluggable report handler, for a binary that wants a custom error report. `miette` renders diagnostics with source spans, for a compiler, a linter, or a CLI that points at a line of input. `displaydoc` takes the `Display` message from the doc comment. Use one convention per crate, and the project's.
- **Builders.** `bon` and `typed-builder` report a missing required member at compile time. `derive_builder` reports it at run time, from `build()`. For a new builder, take a compile-time checked one. Keep the one the project uses.
- **Validation.** `validator` and `garde` derive field checks behind a `.validate()` call and report every failed field at once. That report is the product at an HTTP form boundary. A caller can skip the call, and serde does not run it. The check that establishes an invariant still lives in the type's constructor. Use the derive at the edge for the report, and the constructor for the guarantee.
- **Async traits.** `#[async_trait]` boxes the returned future on every call, static or dynamic, and gives an object-safe trait with one attribute. Native `async fn` in a trait with `trait-variant` and `dynosaur` boxes only on the dynamic path. Behind a `Box<dyn Trait>` seam the two cost the same, and `#[async_trait]` is one attribute where the pair is two. Keep the project's choice.

## Keep to std

- `lazy_static`, `once_cell`: use `std::sync::LazyLock` and `std::sync::OnceLock`. Both are stable since Rust 1.80. On an older supported toolchain, `once_cell` stays.
- `static_assertions`: write `const _: () = assert!(..);` for a predicate that const evaluation can decide, such as a size, an alignment, or a relation between two consts. It cannot state a trait bound. For a trait bound, write `const _: fn() = || { fn assert<T: Send>() {} assert::<Worker>() };`, or keep `assert_impl_all!` in a crate that already has the dependency.

## The cost of a proc macro

A proc macro runs at compile time. Its expansion costs nothing at run time. The cost that can land on a hot path is in the code it emits. Look for a `Box<dyn ...>`, a `format!`, a `clone`, or an allocation per call. Run `cargo expand` once on a new derive and read the output for those. Do not add a crate for a single use. Do not add a crate whose proc macro would put such code on a hot path without a measured reason.

Every version above was checked against crates.io on 2026-09-04.
