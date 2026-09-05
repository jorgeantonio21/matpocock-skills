# Crates

Read this file when you are about to write an impl by hand that a crate would derive. The list is short on purpose. None of these crates is required. Use one only when it removes hand-written code in the change you are making. Use it only when the crate is already a dependency, or when the pull request can justify it. Do not add a crate for a single use. Write the code instead. A new dependency is a per-PR decision. This file does not instruct a migration. Respect the project's supported Rust version and existing error, builder, validation, and async-trait conventions. Macro expansion happens at compile time; inspect generated runtime code for allocation, dispatch, and synchronization.

## The set

- **`thiserror` 2 and `anyhow` 1.** Prefer `thiserror` for typed library errors and `anyhow` for application reports when no project convention exists. Existing alternatives and small handwritten std implementations are valid. The rules are in the Errors section of [SKILL.md](SKILL.md). The `thiserror` expansion is the hand-written `Display`, `source`, and `From`, so inspect the generated impl for the operations used on a hot path.

- **`derive_more` 2.1.** Use `#[derive(Display, From, Into, FromStr)]` on a newtype instead of four delegating impls. Enable one cargo feature per derive. Derive inbound `From` and delegating `FromStr` only when every inner value is valid. Otherwise route parsing through checked conversion and derive only the valid outbound or display behavior. The `Into` derive expands to `impl From<JobId> for u64`, which is the outbound `From` the Shape section asks for; no `Into` impl is written. Do not derive `Deref` on a newtype. The expansion is the hand-written code, so inspect the generated impl for the operations used on a hot path.

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] // std
  #[derive(Display, From, Into)] // derive_more
  pub struct JobId(u64);
  ```

- **`strum` 0.28.** Use `#[derive(VariantArray, EnumCount, EnumString, Display)]` on a unit enum instead of a hand-written `ALL` array, a `COUNT` const, or a match-table `FromStr`. Use `VariantArray`, which is a `'static` slice. Use `EnumIter` when an iterator is useful. Its generated iterator state is not itself a heap allocation. `ParseError` carries no payload, so inspect the generated impl for the operations used on a hot path.

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq)] // std
  #[derive(VariantArray, EnumCount, EnumString, Display)] // strum
  #[strum(serialize_all = "lowercase")]
  pub enum Priority {
      Low,
      Normal,
      High,
  }
  // Priority::VARIANTS, Priority::COUNT, "high".parse::<Priority>()
  ```

- **`itertools` 0.15.** Use `tuple_windows` for adjacent pairs, `chunk_by` for runs in sorted input, `kmerge` for a k-way merge, and `exactly_one` for an expected single result. Use a std method first where one exists: `slice::chunk_by`, `Iterator::is_sorted`, `inspect`. Write a typed `collect()`. Do not use `collect_vec()`. Inspect the cost of the chosen adaptor. `sorted_*`, `unique`, `counts`, and `into_group_map` allocate, so use them in tests and cold paths only.

- **`tokio-util` 0.7.** Use `CancellationToken` for shutdown and `TaskTracker` for a set of workers at the async edge, as [RUNTIME.md](RUNTIME.md) describes. Check the pinned token implementation for internal locking before using it in a hot loop; bridge to an atomic where required. Use `codec::Framed` with a `Decoder` for length-prefixed framing at a wire edge. No proc macro.

  ```rust
  let token = CancellationToken::new();
  let tracker = TaskTracker::new();
  for worker in workers {
      tracker.spawn(run(worker, token.child_token()));
  }

  signal::ctrl_c().await?;
  token.cancel();
  tracker.close();
  tracker.wait().await;
  ```

- **`trait-variant` 0.1 and `dynosaur` 0.3.** Use `#[trait_variant::make(Send)]` on a trait with a native `async fn` when a spawn needs the returned future to be `Send`. Use `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` on the trait when a caller needs a trait object such as `Box<DynStore<'_>>`. Static dispatch avoids returned-future boxing; dynamic calls still box the returned future. `async-trait` supplies boxed-future methods and can be an appropriate alternative. No adapter is required merely to follow this skill. See [RUNTIME.md](RUNTIME.md) for when a trait needs async at all.

  ```rust
  #[trait_variant::make(Send)]
  #[dynosaur::dynosaur(DynStore = dyn(box) Store)]
  pub trait Store {
      async fn put(&self, key: Key, value: Vec<u8>) -> Result<(), StoreError>;
  }
  ```

- **`bon` 3.10.** Use it only when a config struct has many optional fields and needs a builder. Put `#[bon::bon]` on the impl block and `#[builder]` on `new`. A missing required member is then a compile error. A fallible `new` becomes a fallible `build()`, so validation lives in `new` and the fields stay private. Compile time grows with the member count, so use it for config-sized structs only. Check the generated runtime code, including constructor validation.

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

## When an alternative fits

- `validator`, `garde`: useful for field checks and aggregate diagnostics on raw input. Always run validation at the conversion into a validated type. A skippable `.validate()` call does not establish an invariant, and an ordinary Serde derive does not call it. Use checked deserialization, including `#[serde(try_from = "RawType")]` where appropriate.
- `derive_builder`, `typed-builder`: choose from the project's construction contract. `derive_builder` can report missing fields at runtime; `typed-builder` can enforce required fields at compile time. Every builder must still validate relationships between fields.
- `eyre`, `color-eyre`, `snafu`, `miette`, `displaydoc`: reporting, context, and diagnostic capabilities can justify these choices. Keep an existing error convention; do not migrate to `thiserror` or `anyhow` just to follow this skill.
- `async-trait`: useful for boxed-future trait methods, including dynamic dispatch. Native async methods suit static dispatch; `dynosaur` keeps static calls unboxed but boxes dynamic returned futures. Choose the tradeoff from the callers and the hot-path budget.

## Standard facilities and dependency limits

- `lazy_static`, `once_cell`: use `std::sync::LazyLock` and `std::sync::OnceLock` when the project's supported Rust version and required APIs permit. `OnceLock` stabilized in 1.70 and `LazyLock` in 1.80; individual methods may require a newer version.
- `static_assertions`: write `const _: () = assert!(..);` for const-evaluable predicates. This does not cover every trait assertion. Use an ordinary compile-time trait-bound assertion for a positive trait guarantee.
- Do not add a crate for a single use, or a proc-macro crate whose generated runtime code would land on a hot path without a measured reason. The performance concern is the generated runtime code, not compile-time macro expansion.

```rust
const HEADER_BYTES: usize = 8;
const _: () = assert!(HEADER_BYTES <= 16);

const _: fn() = || {
    fn assert_send<T: Send>() {}

    assert_send::<Vec<u8>>();
};
```

This proves the stored type's bound. Assert a method's returned future separately when that is the required guarantee (see [RUNTIME.md](RUNTIME.md)).

The versions in the preferred set were checked against crates.io on 2026-09-04. See [Serde's checked conversion](https://serde.rs/container-attrs.html#try_from), [dynosaur's dispatch behavior](https://docs.rs/dynosaur/0.3.1/dynosaur/), and the method-level stabilization notes for [OnceLock](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) and [LazyLock](https://doc.rust-lang.org/std/sync/struct.LazyLock.html).
