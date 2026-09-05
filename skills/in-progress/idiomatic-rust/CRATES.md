# Crates

Read this when choosing a dependency or derive. Respect existing project conventions and supported Rust versions. No dependency migration is required just to follow this skill. Use standard facilities or a small implementation when they suffice; a dependency earns its place through meaningful repeated capability, maintenance, or diagnostics.

Do not add a crate for a single use. Do not add a proc-macro crate whose generated runtime code would land on a hot path without a measured reason. Macro expansion itself happens at compile time: inspect the generated runtime code for allocation, dispatch, and synchronization. This constraint is not a blanket ban on proc macros, nor a claim that every derive is free.

## Selection guidance

| Need | Capabilities and tradeoffs |
| --- | --- |
| Typed errors | `thiserror` generates `Display`, sources, and conversions. Handwritten std implementations can suffice for a small error surface. `snafu` adds context selectors; `displaydoc` derives display text. Choose according to caller matching and the existing error convention. |
| Application reports | `anyhow` and `eyre` provide context and erased errors. `color-eyre` and `miette` can improve terminal diagnostics. Preserve structured errors where callers need to act; a small CLI or test can use `Box<dyn Error>` without another dependency. |
| Newtype conversion | `derive_more` can remove repetitive delegation. Inbound `From` and delegating `FromStr` are valid only for unconstrained wrappers. Validated wrappers need checked conversion. Outbound delegation and `Display` do not create an invalid value. |
| Validation | `validator` and `garde` can express field checks and aggregate reports. A raw DTO may derive them; ensure conversion into the validated type always runs the checks. Post-construction validation that callers can skip does not establish an invariant. Serde's `try_from` can enforce this boundary. |
| Builders | `bon` and `typed-builder` can encode required fields at compile time. `derive_builder` can report missing fields at runtime. Handwritten constructors/builders may be smaller. In every case, validate cross-field relationships at build time; required fields alone do not establish semantic validity. |
| Enum conversion | `strum` is useful for repeated parsing, display, and variant enumeration. A small match may suffice. Generated iterator state is not itself evidence of heap allocation. Check accepted spellings and compatibility before deriving parsing. |
| Iteration | Prefer a std adaptor when it expresses the operation. `itertools` earns its place for repeated operations unavailable in std. Allocation depends on the chosen adaptor and usage; inspect it rather than calling all adaptors zero-cost. |
| Async runtime utilities | `tokio-util` provides cancellation, tracking, and codecs when Tokio is already appropriate. Define draining and frame validation explicitly. See [RUNTIME.md](RUNTIME.md) for internal synchronization and hot paths. |
| Async trait dispatch | Native methods/`impl Future` suit static calls; boxed-future methods, `async-trait`, or adapters support dynamic calls. `trait-variant` can supply Send variants. `dynosaur` avoids returned-future boxing on static dispatch but still boxes on dynamic dispatch. Choose the required dispatch and bounds before choosing a macro. |
| Tests | Table-driven std tests can suffice. `rstest` helps when separate named cases and reusable fixtures matter. `pretty_assertions` helps inspect large differences. Keep these as dev-dependencies when chosen. |

## Retained standard-library preferences

- `lazy_static`, `once_cell`: use `std::sync::LazyLock` and `std::sync::OnceLock` when the supported Rust version and needed APIs permit. `OnceLock` stabilized in Rust 1.70 and `LazyLock` in 1.80; individual methods may require newer versions. Older MSRVs or missing capabilities can justify retaining an existing dependency.
- `static_assertions`: write `const _: () = assert!(..);` for **const-evaluable predicates**. This is not a replacement for every trait assertion. Use an ordinary compile-time trait-bound assertion for a positive trait guarantee; choose a suitable method for negative or other unsupported assertions.

```rust
const HEADER_BYTES: usize = 8;
const _: () = assert!(HEADER_BYTES <= 16);
const _: fn() = || {
    fn assert_send<T: Send>() {}
    assert_send::<Vec<u8>>();
};
```

The second assertion proves the stored type is Send. To require a method's returned future to be Send, assert that future as shown in [RUNTIME.md](RUNTIME.md).

## Sources

- [Serde checked conversion](https://serde.rs/container-attrs.html#try_from): deserializes the raw type and runs fallible conversion.
- [Dynosaur dispatch](https://docs.rs/dynosaur/0.3.1/dynosaur/): return boxing differs between static and dynamic calls.
- [OnceLock](https://doc.rust-lang.org/std/sync/struct.OnceLock.html) and [LazyLock](https://doc.rust-lang.org/std/sync/struct.LazyLock.html): check method-level stabilization against the project's MSRV.
