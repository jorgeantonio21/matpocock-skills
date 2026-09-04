# Crates

Read this file when you are about to write an impl by hand that a crate would derive. The list is short on purpose. None of these crates is required. Use one only when it removes hand-written code in the change you are making, and only when the crate is already a dependency or the pull request can justify it. Do not add a crate for a single use. Write the code instead. A new dependency is a per-PR decision. This file does not instruct a migration.

## The set

- **`thiserror` 2 and `anyhow` 1.** Use `thiserror` for an error type in a library. Use `anyhow` in a binary and in tests. The rules are in the Errors section of [SKILL.md](SKILL.md). The `thiserror` expansion is the hand-written `Display`, `source`, and `From`, so it is hot-path safe.

- **`derive_more` 2.1.** Use `#[derive(Display, From, Into, FromStr)]` on a newtype instead of four delegating impls. Enable one cargo feature per derive. Derive `From` inbound only when every inner value is valid. Otherwise write a `const fn new` and derive only `Into`. The `Into` derive expands to `impl From<JobId> for u64`, which is the outbound `From` the Shape section asks for; no `Into` impl is written. Do not derive `Deref` on a newtype. The expansion is the hand-written code, so it is hot-path safe.

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

- **`bon` 3.10.** Use it only when a config struct has many optional fields and needs a builder. Put `#[bon::bon]` on the impl block and `#[builder]` on `new`. A missing required member is then a compile error, and a fallible `new` becomes a fallible `build()`, so validation lives in `new` and the fields stay private. Compile time grows with the member count, so use it for config-sized structs only. The runtime cost is zero.

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
  fn parses_priority(#[case] raw: &str, #[case] expected: Option<Priority>) {
      assert_eq!(raw.parse::<Priority>().ok(), expected);
  }
  ```

- **`pretty_assertions` 1.4.** Write `use pretty_assertions::assert_eq;` at the top of `mod tests`. A failed `assert_eq!` then prints a diff instead of two `Debug` dumps. Dev-dependency.

## One rule where two overlap

Use `derive_more::Display` for a type with a format string. Use `strum::Display` for a unit enum.

## Do not reach for

- `validator`, `garde`: a post-construction `.validate()` that a caller can skip and that serde never runs. Put the check in the type's constructor.
- `derive_builder`, `typed-builder`: a missing field surfaces at runtime or as a deprecation warning. Use `bon` when a builder is warranted at all.
- `eyre`, `color-eyre`, `snafu`, `miette`, `displaydoc`: a second error convention. Use `thiserror` and `anyhow`.
- `async-trait`: native `async fn` in traits is stable. See [RUNTIME.md](RUNTIME.md) for when a trait needs async at all.
- `lazy_static`, `once_cell`: use `std::sync::LazyLock` and `std::sync::OnceLock`.
- `static_assertions`: write `const _: () = assert!(..);`.
- A crate for a single use, or a crate whose proc macro would land on a hot path without a measured reason.

Every version above was checked against crates.io on 2026-09-04.
