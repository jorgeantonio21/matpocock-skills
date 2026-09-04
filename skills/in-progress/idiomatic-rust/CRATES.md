# Crates

Read this file when you are about to write an impl by hand that a crate would derive. Each entry says which derive or type to use, what not to write by hand, the hot-path cost, and the `SKILL.md` section it serves. Use these crates when you write new code. A new dependency in a repository is a per-PR decision. This file does not instruct a migration.

## Adopt

- **`derive_more` 2.1.** Use `#[derive(Display, From, Into, FromStr)]` on a newtype. Do not write the delegating impls by hand. Enable one cargo feature per derive. Derive `From` inbound only when every inner value is valid. Otherwise derive `Into` outbound and write a `const fn new`. Do not derive `Deref` on a newtype. `Deref` is for smart pointers and owning collections. The expansion is the hand-written code, so it is hot-path safe. Section: Shape.

  ```rust
  pub struct OrderId(u64);
  impl fmt::Display for OrderId {
      fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { self.0.fmt(f) }
  }
  impl From<u64> for OrderId { fn from(v: u64) -> Self { Self(v) } }
  impl From<OrderId> for u64 { fn from(v: OrderId) -> Self { v.0 } }
  ```

  ```rust
  #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Display, From, Into)]
  pub struct OrderId(u64);
  ```

- **`strum` 0.28.** Use `#[derive(VariantArray, EnumCount, EnumString, Display)]` with `#[strum(serialize_all = "lowercase")]` on a unit enum. Do not write an `ALL` array, a `COUNT` const, or a match-table `FromStr` by hand. Use `VariantArray`, which is a `'static` slice. Do not use `EnumIter`, which allocates an iterator struct. Use `EnumDiscriminants` for an enum with payloads. `ParseError` carries no payload, so it is hot-path safe. Section: Shape.

  ```rust
  pub enum Side { Buy, Sell }
  impl Side { pub const ALL: [Side; 2] = [Side::Buy, Side::Sell]; }
  impl FromStr for Side {
      type Err = ParseSideError;
      fn from_str(s: &str) -> Result<Self, ParseSideError> {
          match s { "buy" => Ok(Side::Buy), "sell" => Ok(Side::Sell), _ => Err(ParseSideError) } } }
  ```

  ```rust
  #[derive(Clone, Copy, Debug, PartialEq, Eq, VariantArray, EnumCount, EnumString, Display)]
  #[strum(serialize_all = "lowercase")]
  pub enum Side { Buy, Sell }
  // Side::VARIANTS, Side::COUNT, "buy".parse::<Side>()
  ```

- **`bon` 3.10.** Use `#[bon::bon]` on the impl block and `#[builder]` on `new` for a config struct. Do not use `derive_builder` with a `validator` pass, which gives a runtime `Result` from `build()` and a `.validate()` call the caller can forget. With `bon`, a missing required member is a compile error. A fallible `new` becomes a fallible `build()`, so validation lives in `new` and the fields stay private. The builder is a typestate with one generic per required member, so compile time grows with the member count. Use it for config-sized structs only. The runtime cost is zero.. Section: Surface.

  ```rust
  #[derive(Builder, Validate)]
  pub struct Gateway {
      #[validate(range(min = 1024))] pub port: u16,
      #[builder(default = "Duration::from_secs(5)")] pub timeout: Duration,
  }
  // GatewayBuilder::default().port(9000).build()?.validate()?
  ```

  ```rust
  #[bon::bon]
  impl Gateway {
      #[builder]
      pub fn new(port: Port, #[builder(default = Duration::from_secs(5))] timeout: Duration) -> Self {
          Self { port, timeout } }
  }
  ```

- **`thiserror` 2, `anyhow` 1.** Keep them as the one error convention: `thiserror` in a library, `anyhow` in a binary and in tests. The rules are in the Errors section of `SKILL.md`. The expansion is the hand-written `Display`, `source`, and `From`, so it is hot-path safe. Section: Errors.

- **`tokio-util` 0.7 (`CancellationToken`, `TaskTracker`) with `tokio::task::JoinSet`.** Select on a `CancellationToken` in every task. Use `TaskTracker` for workers whose result is `()`. Use `JoinSet` when you read the results. Use `child_token()` to scope cancellation per subsystem. Do not poll an `Arc<AtomicBool>` in an async loop. Do not signal shutdown with a broadcast channel of `()`. Do not join a `Vec<JoinHandle>` in a loop. An `AtomicBool` that a sync OS thread reads is correct. No proc macro. Section: Runtime.

  ```rust
  let shutdown = Arc::new(AtomicBool::new(false));
  let mut handles = Vec::new();
  for gw in gateways { let s = shutdown.clone(); handles.push(tokio::spawn(run(gw, s))); }
  signal::ctrl_c().await?; shutdown.store(true, Ordering::SeqCst);
  for h in handles { h.await??; }
  ```

  ```rust
  let token = CancellationToken::new();
  let tracker = TaskTracker::new();
  for gw in gateways { tracker.spawn(run(gw, token.child_token())); }
  signal::ctrl_c().await?; token.cancel();
  tracker.close(); tracker.wait().await;
  ```

- **`rstest` 0.26.** Use `#[rstest]` with one `#[case::name(..)]` per input row. Use `#[fixture] fn engine() -> Engine`, which rstest resolves by argument name. Each case is then its own named test under nextest. Do not loop a test table inside one `#[test]`, which reports one failure for the whole table. Do not call a `fn setup()` at the top of every test. Dev-dependency, so the proc-macro cost lands on test targets only. Section: Words (tests).

  ```rust
  #[test]
  fn rejects_bad_prices() {
      for (raw, ok) in [(0, false), (1, true), (u64::MAX, false)] {
          assert_eq!(setup().accept(Price::new(raw)).is_ok(), ok, "raw={raw}");
      }
  }
  ```

  ```rust
  #[rstest]
  #[case::zero(0, false)]
  #[case::one(1, true)]
  fn rejects_bad_prices(engine: Engine, #[case] raw: u64, #[case] ok: bool) {
      assert_eq!(engine.accept(Price::new(raw)).is_ok(), ok);
  }
  ```

- **`num_enum` 0.7.** Use `#[derive(IntoPrimitive, TryFromPrimitive)]` on a `#[repr(u8)]` wire enum. Do not write a `TryFrom<u8>` match table or an `as u8` cast by hand. Add `#[num_enum(catch_all)]` on an `Unknown(u8)` variant for a forward-compatible protocol. Use it at decode boundaries only. A zerocopy struct keeps the raw `u8` field and converts at the edge. The expansion is the same match, the crate is `no_std`, and the error carries only the offending value, so it is hot-path safe. Section: Shape.

  ```rust
  #[repr(u8)] pub enum MsgType { NewOrder = 1, Cancel = 2 }
  impl TryFrom<u8> for MsgType {
      type Error = UnknownMsgType;
      fn try_from(v: u8) -> Result<Self, UnknownMsgType> {
          match v { 1 => Ok(Self::NewOrder), 2 => Ok(Self::Cancel), other => Err(UnknownMsgType(other)) } } }
  impl From<MsgType> for u8 { fn from(m: MsgType) -> u8 { m as u8 } }
  ```

  ```rust
  #[repr(u8)]
  #[derive(Clone, Copy, Debug, PartialEq, Eq, IntoPrimitive, TryFromPrimitive)]
  pub enum MsgType { NewOrder = 1, Cancel = 2 }
  ```

- **`serde_with` 3.** Use `#[serde_as]` with `DurationSeconds<u64>`, `DisplayFromStr` (which reuses the type's `FromStr` and `Display`), and `Bytes` for arrays past 32 elements, in config and API crates. Use `#[skip_serializing_none]` once at struct level. Do not write `deserialize_with = "helper"` functions, `#[serde(with = "..")]` modules, or `skip_serializing_if` on every field. Do not use it in a wire crate. The attribute macro rewrites the struct, and the adapter set is large. Section: Surface.

- **`delegate` 0.13.** Use `delegate! { to self.field { pub fn len(&self) -> usize; .. } }` when a wrapper forwards four or more methods to one field. The signatures stay written and greppable. Below four methods, write the bodies by hand. The expansion is the direct call with `#[inline]`, so it is hot-path safe. Section: Surface.

- **`insta` 1.48.** Use `assert_snapshot!(hexdump(&frame.encode()))` for a wire encoding, the state after a scenario, or a CLI or config rendering. The snapshot is a `.snap` file beside the test. Review it with `cargo insta review`. Use redactions for timestamps and ids. Do not write a long literal `assert_eq!` on bytes or on a dump. Keep `assert_eq!` for scalars. Dev-dependency. Section: Words (tests).

- **`itertools` 0.15.** Use `tuple_windows` for adjacent pairs, `chunk_by` for runs in sorted input, `kmerge` for a k-way merge, `exactly_one` for an expected single result, and `format_with` for lazy formatting. Do not write an index-arithmetic loop, a run-grouping loop with state variables, or `next().unwrap()` for these. Use a std method first where one exists: `slice::chunk_by`, `Iterator::is_sorted`, `inspect`. Write a typed `collect()`. Do not use `collect_vec()`. The named set is zero-cost. `sorted_*`, `unique`, `counts`, and `into_group_map` allocate, so use them in tests and cold paths only. Section: Flow.

- **`pretty_assertions` 1.4.** Write `use pretty_assertions::assert_eq;` at the top of `mod tests`. A failed `assert_eq!` then prints a diff instead of two `Debug` dumps. Use its `assert_matches!` for a pattern assert. Dev-dependency. Section: Words (tests).

## Adopt for one condition

- When a credential is loaded from config, use `secrecy` 0.10 `SecretString`. Its `Debug` prints `[REDACTED]`, every read is a greppable `expose_secret()`, and the memory is zeroed on drop. The cost is one `Box` per secret at load time.
- When engine-side entities need stable handles and no tested arena exists, use `slotmap` 1.1 `SlotMap<Key, T>` with `new_key_type!`. The keys are generational, so a removed key is never reused. Insert, get, and remove are O(1). The macro is declarative, so it is hot-path safe.
- When a map is serialized, snapshotted, or replayed, use `indexmap` 2.14. It iterates in insertion order, so the output is identical on every run. Keep `dashmap` for concurrency and `BTreeMap` for sorting.
- When a struct has a typestate or marker type parameter, use `derive-where` 1.6. It derives `Clone`, `Copy`, `PartialEq`, and `Debug` on `Handle<T>(u32, PhantomData<T>)` without a bound on `T`.
- When you need a zero-copy view of a `str` or slice as a newtype, use `ref-cast` 1.0. Put `#[derive(RefCast)]` on a `#[repr(transparent)]` newtype. `Symbol::ref_cast(s)` then makes the view with no copy and no `unsafe` in your code.
- When a field is a set of named bits, use `bitflags` 2.13 `bitflags!`. The macro is declarative, it accepts `#[repr(transparent)]` inside, and `from_bits_retain` keeps unknown bits for forward compatibility.
- When you write a `Future` or `Stream` impl by hand, use `pin-project-lite` 0.2 `pin_project!` with `#[pin]` fields and `self.project()`. It is declarative and it is already in the dependency tree of tokio. Use `pin-project` only for a feature that `pin-project-lite` lacks.
- When you write a fuzz target for a wire decoder or a log reader, use `arbitrary` 1.4 `#[derive(Arbitrary)]`. Its `Arbitrary` trait is a different trait from proptest's, and the two coexist behind separate derives.

## Overlap rules

- Use `derive_more::Display` for a type with a format string. Use `strum::Display` for a unit enum.
- Use `serde_repr` for a repr enum that serde reads and writes. Use `num_enum` for a repr enum that raw bytes decode. When one enum crosses both boundaries, use `num_enum` plus `#[serde(into = "u8", try_from = "u8")]`. Do not put `serde_repr`, `strum::FromRepr`, and `num_enum` together on one enum.

## Skip

- Do not use `typed-builder` or `derive_builder`. A missing field surfaces at runtime or as a deprecation warning, and there is no fallible finish. Use `bon`.
- Do not use `derive-new` or `smart-default`. Std `#[default]` and a hand-written `impl Default` already spell out the defaults. Use `derive_more::Constructor` for a positional `new`.
- Do not use `educe`. It offers per-field `Debug` skipping and custom `Hash` and `Ord` methods, which is more power than a generics-aware derive needs. Use `derive-where`.
- Do not use `ambassador` or `enum_dispatch`. One needs a cross-module macro import in every user, and the other relies on proc-macro global state that breaks under some incremental builds. Write the match, or make the caller generic.
- Do not use `static_assertions`. It last shipped in 2019. Write `const _: () = assert!(..);`, which std supports since 1.57. For `Send + Sync`, write `const _: () = { const fn assert_send_sync<T: Send + Sync>() {} assert_send_sync::<Engine>(); };`.
- Do not use `nonempty`, `nunny`, or `mitsein` by default. `nonempty` stores a head plus a `Vec`, so there is no contiguous slice view. The other two have one maintainer each and a small user base. Write a private `struct Batch(Vec<T>)` with a `first()` that returns `&T` and a constructor that rejects empty input. Use `nunny` only when a non-empty slice type appears in three or more signatures.
- Do not use `bounded-integer`, `konst`, `parse-display`, or `enum-iterator`. A hand-written `const fn new -> Option<Self>` covers the bound. Nothing parses strings at compile time. A regex-backed `FromStr` is the wrong shape for input parsing. `strum::VariantArray` already iterates variants.
- Do not use `enumset`. It is a second flags crate from one maintainer. Use `bitflags`.
- Do not use `tap` or `extend`. `.pipe` costs a prelude import per file, and `extend` hides the extension trait's name from grep. Use std `inspect` and `inspect_err`, and write a `<Type>Ext` trait by hand.
- Do not use `validator`, `garde`, or `serde_valid`. Each gives a post-construction `.validate()` that a caller can skip and that serde never runs. Use `bon` `new` with typed fields. Use `#[serde(try_from = "RawConfig")]` for a check that spans fields.
- Do not use `color-eyre`, `snafu`, `miette`, `displaydoc`, or `error_set`. Each is a second error convention. Use `thiserror` and `anyhow`.
- Do not use `test-case`, `assert_matches`, `googletest`, `expect-test`, or `similar-asserts`. Each is a second test vocabulary. Use `rstest`, `pretty_assertions` (which has `assert_matches!`), and `insta`.
- Do not use `divan`. It is a second benchmark harness. Use `criterion`.
- Do not use `ordered-float` for a price or a quantity. Put money in a decimal type. Use `f64::total_cmp` to sort floats. Use `NotNan` only when an `f64` keys a `BTreeMap`.
- Do not use `nutype` 0.7 on a wire type. It rejects every non-doc attribute on the struct, so `#[repr(transparent)]`, `#[derive(zerocopy::FromBytes)]`, and `#[serde(transparent)]` cannot combine with it. Write `const fn new -> Option<Self>` by hand. When 0.8 ships attribute passthrough, a config-boundary newtype (a port, a symbol, a bounded size) may take `#[nutype(validate(..))]`, whose `try_new` also runs inside `Deserialize`.

Every version above was checked against crates.io on 2026-09-04.
