# Crates

Reached when about to hand-write an impl a crate would derive: each entry names the hand-written form, the derive or type that retires it, the hot-path cost, whether `zll_core` already pays for the crate, and the `SKILL.md` section it serves. Reach for these when writing new code; adding a dependency to a repo is a per-PR call, and nothing here instructs a migration.

## Adopt

- **`derive_more` 2.1.** A newtype carrying hand-written `Display`, `From`, `Into`, `FromStr` that each forward to the inner field. → `#[derive(Display, From, Into, FromStr)]`, one cargo feature per derive; `From` inbound only when every inner value is valid, otherwise `Into` outbound plus a `const fn new`. The `Deref` derive is for smart pointers and owning collections, never a newtype. Expansion is the hand-written code, hot-path safe. New to `zll_core`. Shape.

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

- **`strum` 0.28.** A unit enum with a hand-written `ALL` array, a `COUNT` const, and a match-table `FromStr`. → `#[derive(VariantArray, EnumCount, EnumString, Display)]` with `#[strum(serialize_all = "lowercase")]`; `VariantArray` (a `'static` slice) over `EnumIter` (allocates an iterator struct); `EnumDiscriminants` for enums with payloads. `ParseError` carries no payload, hot-path safe. Already in `zll_core`. Shape.

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

- **`bon` 3.10.** A config struct under `derive_builder` plus a `validator` pass, with a runtime `Result` from `build()` and a `.validate()` the caller can forget. → `#[bon::bon] impl` with `#[builder]` on `new`: a missing required member is a compile error, and a fallible `new` becomes a fallible `build()` for free, so validation lives in `new` and the fields stay private. Typestate builder with one generic per required member, so compile time grows with members: config-sized structs only, zero runtime cost. New to `zll_core`, alternative to `derive_builder`. Surface.

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

- **`thiserror` 2, `anyhow` 1.** Kept as the one library/application error convention; the rules live under Shape in `SKILL.md`. Expansion is the hand-written `Display`, `source`, and `From`, hot-path safe. Already in `zll_core`. Shape, Words.

- **`tokio-util` 0.7 (`CancellationToken`, `TaskTracker`) with `tokio::task::JoinSet`.** Async shutdown through an `Arc<AtomicBool>` polled in loops or a broadcast-of-unit channel, and a `Vec<JoinHandle>` joined in a loop. → `CancellationToken` selected in every task, `TaskTracker` for workers whose result is `()`, `JoinSet` when results are read; `child_token()` scopes cancellation per subsystem. An `AtomicBool` read by a sync OS thread stays correct; the rule is scoped to tasks. No proc macro. Already in `zll_core`. Runtime.

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

- **`rstest` 0.26.** A test table looped inside one `#[test]`, reporting one failure for the whole table, and a `fn setup()` called at the top of every test. → `#[rstest]` with one `#[case::name(..)]` per row and a `#[fixture] fn engine() -> Engine` resolved by argument name; each case is its own named test under nextest. Dev-dependency, proc-macro cost on test targets only. New to `zll_core`. Words (tests).

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

- **`num_enum` 0.7.** A `#[repr(u8)]` wire enum with a hand-written `TryFrom<u8>` match table and `as u8` casts. → `#[derive(IntoPrimitive, TryFromPrimitive)]`; `#[num_enum(catch_all)]` on an `Unknown(u8)` variant for a forward-compatible protocol. Decode boundaries only: a zerocopy struct keeps the raw `u8` field and converts at the edge. Expansion is the same match, `no_std`, the error carries only the offending value, hot-path safe. New to `zll_core`. Shape.

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

- **`serde_with` 3.** `deserialize_with = "helper"` functions and `#[serde(with = "..")]` modules accumulating in a config crate, and `skip_serializing_if` repeated per field. → `#[serde_as]` with `DurationSeconds<u64>`, `DisplayFromStr` (reuses the type's `FromStr`/`Display`), `Bytes` for arrays past 32 elements; `#[skip_serializing_none]` once at struct level. Config and API crates only; the attribute macro rewrites the struct and the adapter set is large, so wire crates keep plain serde. New to `zll_core`. Surface.

- **`delegate` 0.13.** A wrapper whose methods each forward to one field. → `delegate! { to self.field { pub fn len(&self) -> usize; .. } }`: signatures stay written and greppable, bodies go. Four or more forwarded methods; below that, write the bodies. Expansion is the direct call with `#[inline]`, hot-path safe. New to `zll_core`. Surface.

- **`insta` 1.48.** A long literal `assert_eq!` on wire-frame bytes or an order-book dump. → `assert_snapshot!(hexdump(&frame.encode()))` stored as a `.snap` beside the test and reviewed with `cargo insta review`; redactions for timestamps and ids. Encodings, state after a scenario, CLI and config rendering; scalars keep `assert_eq!`. Dev-dependency. New to `zll_core`. Words (tests).

- **`itertools` 0.15.** An index-arithmetic loop over adjacent pairs, run-grouping with state variables, a k-way merge loop, `next().unwrap()` on an expected-single result. → `tuple_windows`, `chunk_by` (sorted input), `kmerge`, `exactly_one`, `format_with`; std first where it exists (`slice::chunk_by`, `Iterator::is_sorted`, `inspect`); a typed `collect()` over `collect_vec()`. No proc macro; the named set is zero-cost, while `sorted_*`, `unique`, `counts`, `into_group_map` allocate and belong in tests and cold paths. Already in `zll_core`. Flow.

- **`pretty_assertions` 1.4.** An `assert_eq!` on a struct whose failure prints two `Debug` walls. → `use pretty_assertions::assert_eq;` at the top of `mod tests` for a coloured diff; its `assert_matches!` for pattern asserts. Dev-dependency, retires nothing. New to `zll_core`. Words (tests).

## Adopt for one condition

All new to `zll_core` except `pin-project-lite`, already compiled as a tokio dependency.

- **Credentials loaded from config** → `secrecy` 0.10 `SecretString`: `Debug` prints `[REDACTED]`, every read is a greppable `expose_secret()`, memory is zeroised on drop. One `Box` per secret at load time.
- **Engine-side entities needing stable handles, where no tested arena exists** → `slotmap` 1.1 `SlotMap<Key, T>` with `new_key_type!`: generational keys, O(1) insert, get, remove, no ABA reuse. Declarative macro, hot-path safe.
- **A map that is serialised, snapshotted, or replayed** → `indexmap` 2.14: insertion-order iteration, identical output every run; `dashmap` stays for concurrency and `BTreeMap` for sorting.
- **Typestate and marker generics** → `derive-where` 1.6: `Clone`, `Copy`, `PartialEq`, `Debug` on `Handle<T>(u32, PhantomData<T>)` with no `T:` bound.
- **A zero-copy `str` or slice newtype view** → `ref-cast` 1.0 `#[derive(RefCast)]` on a `#[repr(transparent)]` newtype: `Symbol::ref_cast(s)` with no copy and no `unsafe` in your code.
- **A field that is a set of named bits** → `bitflags` 2.13 `bitflags!`: declarative, `#[repr(transparent)]` allowed inside, `from_bits_retain` keeps unknown bits for forward compatibility.
- **A manual `Future` or `Stream` impl** → `pin-project-lite` 0.2 `pin_project!` with `#[pin]` fields and `self.project()`, declarative; `pin-project` only for a feature it lacks.
- **A fuzz target on a wire decoder or the WAL reader** → `arbitrary` 1.4 `#[derive(Arbitrary)]`; a different trait from proptest's `Arbitrary`, the two coexist behind separate derives.

## Overlap rules

- `derive_more::Display` for anything with a format string; `strum::Display` for a unit enum.
- `serde_repr` for a serde-facing repr enum; `num_enum` for raw-byte decode; an enum crossing both boundaries takes `num_enum` plus `#[serde(into = "u8", try_from = "u8")]`. Never `serde_repr`, `strum::FromRepr`, and `num_enum` together on one enum.

## Skip

- `typed-builder`, `derive_builder`: missing fields surface at runtime or as deprecation warnings, no fallible finish. `bon`.
- `derive-new`, `smart-default`: positional `new` and per-field defaults that std `#[default]` and a hand-written `impl Default` already spell out. Std; `derive_more::Constructor` for the positional `new`.
- `educe`: per-field `Debug` skipping and custom `Hash`/`Ord` methods, more power than a generics-aware derive needs. `derive-where`.
- `ambassador`, `enum_dispatch`: a cross-module macro import ritual, and proc-macro global state that breaks under some incremental builds. Write the match, or make the caller generic.
- `static_assertions`: last shipped 2019. `const _: () = assert!(..)` (std since 1.57); for `Send + Sync`, `const _: () = { const fn assert_send_sync<T: Send + Sync>() {} assert_send_sync::<Engine>(); };`.
- `nonempty`, `nunny`, `mitsein`: head-plus-`Vec` layout with no contiguous slice view, or one maintainer with a download count four orders of magnitude below the rest. A private `struct Batch(Vec<T>)` with a `first()` returning `&T` and a constructor rejecting empty input; `nunny` once a non-empty slice type appears in three or more signatures.
- `bounded-integer`, `konst`, `parse-display`, `enum-iterator`: a niche the hand-written type does without, compile-time string parsing nothing here does, a regex-backed `FromStr`, and a `strum` duplicate. Hand-written `const fn new -> Option`, `derive_more::Display`, `strum::VariantArray`.
- `enumset`: a second flags crate from one maintainer. `bitflags`.
- `tap`, `extend`: a prelude import per file for `.pipe`, and an extension trait whose name is hidden from grep. Std `inspect` and `inspect_err`; a hand-written `<Type>Ext` trait.
- `validator`, `garde`, `serde_valid`: a post-construction `.validate()` a caller can skip and serde never runs. `bon` `new` plus typed fields; `#[serde(try_from = "RawConfig")]` for a check spanning fields.
- `color-eyre`, `snafu`, `miette`, `displaydoc`, `error_set`: a second error convention. `thiserror` plus `anyhow`.
- `test-case`, `assert_matches`, `googletest`, `expect-test`, `similar-asserts`: a second test vocabulary. `rstest`, `pretty_assertions` (its `assert_matches!`), `insta`.
- `divan`: a second benchmark harness. `criterion`, already present.
- `ordered-float`: prices and quantities are `rust_decimal`. `f64::total_cmp`; `NotNan` only if an `f64` ever keys a `BTreeMap`.
- `nutype` 0.7: rejects every non-doc attribute on the struct, so `#[repr(transparent)]`, `#[derive(zerocopy::FromBytes)]`, and `#[serde(transparent)]` cannot combine with it, and it cannot wrap a wire type. Hand-written `const fn new -> Option<Self>`; once 0.8 ships attribute passthrough, a config-boundary newtype (port, symbol, bounded size) may take `#[nutype(validate(..))]`, whose `try_new` also runs inside `Deserialize`.

## The workspace today

Facts an agent may cite in a review; whether any of them changes is a per-PR call.

- `eyre` appears in one file and one manifest, `settlement_circuit`, where it arrives with `openvm`; `anyhow` is the application error type everywhere else.
- `flume` is declared in the workspace and imported nowhere; `crossbeam-channel` is present.
- `derive_builder` is in six crates and `validator` in twelve; together they do what `bon` does alone, with a runtime `Result` on `build()` and a `.validate()` call a caller can skip.
- `strum` is imported in 97 files with `VariantArray`, `EnumCount`, `FromRepr`, `EnumIter`, `AsRefStr`, `IntoStaticStr` in use; two `ALL` arrays remain hand-written, at `matching_engine/src/version_gates.rs:56` and `types/src/requests.rs:740`.
- Nine hand-written `Display` and `FromStr` impls live in `primitives/src/id.rs` and `price_and_positions.rs`.
- `itertools` is imported in two files.
- `AtomicBool` shutdown flags at `wal/src/io_thread.rs:23` (a sync OS thread) and `market_data_gateway/src/server.rs:761` (async cancellation bridged to a sync loop, with a comment saying so) are correct.
- `Vec<JoinHandle>` occurs once, at `market_data_router/src/private/emitter.rs:211`, behind a `Mutex`, holding OS-thread handles.
- `mockall` is present; the house rule is mock boundaries, not logic.
- `uuid` and `ulid` coexist; the split is external versus internal id.

Every version above was checked against crates.io on 2026-09-04.
