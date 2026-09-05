---
name: idiomatic-rust
description: Contextual Rust idioms and invariant design. Use when writing Rust, refactoring existing Rust, reviewing a diff that touches .rs or Cargo.toml files, or when another skill needs the Rust idiom baseline.
---

# Idiomatic Rust

Use the sections relevant to the change. These patterns apply to libraries, CLI tools, inference servers, and stateful services without prescribing one architecture. A documented repository standard overrides this file's conventions, but cannot make an invalid construction or unsafe operation correct.

Distinguish three kinds of guidance in implementation and review:

- **Correctness requirements:** preserve the promised invariants and supported behavior across every entry path and operation. Prove a violation with an input or execution, not a style preference.
- **Contextual defaults:** choose ownership, dispatch, errors, and dependencies from the callers, supported Rust version, and measured requirements. Explain departures where the reason is not clear from the code.
- **Optional readability conventions:** names, local aliases, derive grouping, comment shape, and module layout. Respect house-rule exemptions. Do not report a convention as a bug or demand a migration to satisfy it.

## Shape

- **Types earn their place.** Introduce a newtype when it protects an invariant, distinguishes confusable roles, or carries useful behavior. `Width` and `Height` can prevent swapped arguments; `Duration` supplies units and operations. Two primitive parameters alone are not a defect: `(title: &str, body: &str)` and simple counters can stay simple. Named fields can distinguish two roles of the same type.
- **Name meaningful states.** Use enums for mutually exclusive states or several kinds of absence. Keep `Option<T>` for ordinary optional values, and a boolean where its meaning is clear. Module privacy and narrow methods often suffice. Add typestate, branded lifetimes, version tokens, or restrictions on `Copy` only when a concrete misuse needs preventing.
- **Audit the guarantee.** Before claiming that a type is always valid, check constructors, `From`/`TryFrom`, `FromStr`, deserializers, database reads, `Default`, mutation, and arithmetic. Private fields protect callers outside the module, not other construction within it. Validate whole aggregates when the rule relates several values. A setter or `&mut` accessor must not bypass that rule.
- **Parse structure, then validate meaning.** Raw bytes or decoded data need checked conversion before acquiring a semantic guarantee. An intrinsic bound remains true until an operation changes the value. Admission under policy belongs to the authority that knows that policy; historical authorization does not prove validity against later mutable state. Preserve admitted work when compatibility requires it. Read [INVARIANTS.md](INVARIANTS.md) for these boundaries, operations, transition results, and persistence changes.
- **Checked newtypes.** Keep constrained fields private and route every inbound conversion through validation. Use `const fn` when callers need constants and the supported toolchain permits it. For a bound, use `if value <= MAX { Some(Self(value)) } else { None }`, which works in a `const fn`; `then_some` is not const-compatible on the evaluation toolchain. Use `Result<Self, E>` when callers need a diagnostic, including the rejected value where useful. Infallible inbound `From`, delegating `FromStr`, and unchecked `Deserialize` derives belong only on unconstrained wrappers. A validated wrapper's `FromStr` parses and calls `TryFrom`; Serde can use `#[serde(try_from = "RawType")]`. Outbound access or `From<Wrapper> for Inner` can expose the value without permitting mutation.
- **Layout is a separate contract.** Use `repr(C)` or `repr(transparent)` only when an ABI or raw-byte representation requires it. Neither validates bytes nor specifies byte order or a portable serialization format. Ordinary JSON/protobuf models can keep `String` and `Vec`. Audit padding, alignment, byte validity, and ownership separately for raw-byte access.
- **Derive valid behavior.** Derive standard traits when their implementations preserve the contract, including `Default`. Keep `Copy` for simple values unless copying violates a concrete ownership rule. Use `pub(crate)` for crate callers and `pub` for external callers. Consider `#[non_exhaustive]` before publishing an extensible type. A builder is useful when construction is complex; its final build must validate relationships too.
- **Small traits, chosen dispatch.** Ask for the capabilities a caller needs. Generics suit static specialization; `dyn Trait` suits heterogeneous collections and pluggable backends. Neither is mandatory for all systems. A forwarding method is usually clearer than `Deref` for a field that is not a smart pointer or collection.

## Errors

- **Model the caller's decision.** An outcome enum can separate expected admission or rejection from infrastructure failure when callers act on those distinctions. A plain `Result<T, E>` is often clearer for parsing, libraries, thin adapters, and scripts. Use `#[must_use = "reason"]` when silently dropping a standalone outcome loses a required action; a private outcome exhaustively handled by its one caller may not need it.
- **Keep actionable error data.** Use typed errors when callers match failures. Include useful ids, offsets, paths, or bounds and preserve cause chains. `thiserror` is a convenient default, not a required ecosystem. Handwritten `Error` implementations and existing alternatives are valid. An application or test can use `anyhow`, another report type, or `Box<dyn Error>` when it only reports failures. Avoid allocating on measured hot failure paths without a reason.
- **Compose without losing meaning.** With `thiserror`, `#[from]` generates conversion and a source; `#[source]` retains a cause when the outer error adds context; `transparent` forwards a wrapped report. Use equivalent capabilities in the project's error library. Translate explicitly where a layer promises a narrower error surface. Preserve the distinction between rejecting input and silently discarding valid work.
- **A local Result alias is optional.** When a module consistently returns its own error, an alias can shorten signatures. Qualify the definition to avoid recursion, and match its visibility to callers. Keep explicit results for generic errors, multiple error types, or import collisions. This convention does not require one global crate error or force tests to share a production error type.

  ```rust
  #[derive(Debug)]
  pub enum DecodeError {
      InvalidHeader,
      TruncatedInput,
  }

  pub type Result<T> = std::result::Result<T, DecodeError>;
  ```

  Use `core::result::Result` in a suitable `no_std` module. The example's `pub` is not mandatory.

- **Handle failures deliberately.** Propagate with `?` when callers can act. A best-effort operation may log and continue or explicitly discard its result, with the reason stated locally. Add context where the lower-level report cannot explain the failed operation.

## Ownership

- **Take and return what callers need.** Borrow `str`, slices, paths, and values unless storing or consuming them. A borrowed public view is valid; ownership is useful when independence from the source lifetime matters. Return an iterator for one-pass consumption or a collection when callers need storage. Use `Cow` when avoiding a common copy helps.
- **Clone for an owner.** First consider a shorter borrow, disjoint fields, `Option::take`, or `mem::take`. A clone is legitimate when a second owner needs the value. Prefer `Arc::clone(&value)` when making reference-counted sharing visible helps. Do not add `Arc` or `'static` just to obscure a lifetime problem.
- **Share according to the workload.** Start with exclusive ownership or partitioned state. Channels, snapshots, atomics, and locks solve different requirements. Read [RUNTIME.md](RUNTIME.md) for the strong hot-path lock preference and its narrow correctness exception. A channel or concurrent collection is not necessarily lock-free.
- **Make nondeterminism testable.** Pass time and randomness into replayable transitions. A small CLI need not grow a clock trait to read the current time. Use monotonic time for elapsed durations and wall-clock time for recorded timestamps.

## Flow

- **Make decisions visible.** Use `?`, guards, `let ... else`, combinators, or `match` according to which makes the cases clearest. Let chains require a compatible edition and Rust version. Use exhaustive matches where a new local enum variant needs a new policy decision. Put shared interpretation on the owning status type so consumers cannot drift.
- **Choose arithmetic semantics.** Use checked operations for possible overflow and reflect failure in the return type. Nonzero signed values still include the minimum value, whose negation overflows. Saturation and wrapping are domain choices, not interchangeable fixes. A stricter representation must preserve valid behavior and supported exceptions; do not silently round, drop, or reinterpret input to fit it.
- **Use the transition's result.** Have an authoritative mutation return the removed resources and resulting state needed by callers. A second lookup into mutable state can observe a later state or lose information. See [INVARIANTS.md](INVARIANTS.md) for an independent example and alternatives.
- **Keep iteration clear.** Iterator chains suit transformations and fallible collection; loops suit side effects or complex state. Use standard methods when they express the operation directly. Name constants when the name explains a unit or policy, rather than wrapping every literal.
- **A panic needs a contract.** Return errors for invalid external input. At a provably infallible site, an `expect` message should state the invariant. Tests can panic on failed expectations. Follow the project's lint exceptions without replacing a justified assertion with a fictional error path.

## Surface and words

- **Optimize for readers.** Use conventional conversion names (`as_`, `to_`, `into_`) and one term per concept. Group arguments when named fields clarify roles. Keep imports and module boundaries easy to navigate; a tiny library can live in `lib.rs`. Derive grouping, test prefixes, and short comments are readability preferences, not independent correctness guarantees.
- **Explain obligations.** Comments should explain reasons the code cannot show. Public docs should state guarantees, failures, and compatibility constraints. A `// SAFETY:` comment must establish the actual obligation of an unsafe operation. Keep repository conventions and supported lint syntax, including house-rule exemptions such as `clippy::redundant_test_prefix`.
- **Test behavior.** Exercise invalid construction routes, arithmetic limits, state changes, and historical data. Integration tests can protect public guarantees without matching an implementation. A test may return `()`, a local alias, or another suitable error type. Keep straightforward valid code unchanged when extra wrappers, aliases, dependencies, or typestate provide no benefit.
- **Use the output channel the product needs.** Structured logs suit services. CLI stdout is legitimate product output; apply its documented lint exception in both scoring passes. Choose log severity and per-request volume from operational requirements.

## Check

Run the relevant tests and the commands in [LINTS.md](LINTS.md) on touched crates, respecting the project's supported toolchain, feature combinations, and documented exceptions. Format with the project's formatter; use nightly only when its configuration requires it. Fix findings introduced by the change and report unrelated backlog. A failed or incomplete compiler invocation is not a clean check. In review, separate demonstrated correctness problems, contextual tradeoffs, and optional conventions.

## Pointers

- **Validation, admission, arithmetic, transition results, or persisted representations change:** read [INVARIANTS.md](INVARIANTS.md).
- **Async code, tasks, threads, or shared state change:** read [RUNTIME.md](RUNTIME.md).
- **Choosing a dependency or derive:** read [CRATES.md](CRATES.md).
- **Running checks or deciding lint exceptions:** read [LINTS.md](LINTS.md).
