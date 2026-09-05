---
name: idiomatic-typescript
description: Idiomatic TypeScript with honest type and runtime guarantees. Use when writing TypeScript, refactoring existing TypeScript, reviewing changes to .ts, .tsx, .mts, or .cts files, designing a TypeScript API or validation boundary, changing TypeScript build or module configuration, or when another skill needs the TypeScript idiom baseline.
---

# Idiomatic TypeScript

Write TypeScript whose guarantees are honest: express what the compiler checks, establish runtime facts at trust boundaries, and say when neither proves enough. This baseline covers framework-neutral application and library code, including TypeScript inside `.tsx`. Framework component design belongs to a framework-specific skill.

A documented repository standard overrides a default or convention here. It cannot turn an erased type into a runtime check. Existing public contracts, generated code, framework contracts, and supported consumers constrain every rewrite.

Each rule has a stable name and one kind:

- A **requirement** names a concrete unsafe path or broken guarantee. A finding demonstrates that path, not merely a keyword.
- A **default** selects a sound starting point and names when another choice fits. Repository practice can override it.
- A **convention** concerns presentation. A finding is advisory unless the repository adopts it as a standard.

## Orient

Before writing, inspect repository instructions, sibling code, `package.json`, relevant TypeScript and lint configuration, runtime and package module mode, supported consumers, and existing check commands. Determine whether the code is an application, a library, generated output, or directly executed TypeScript. Done when you can name the contracts, versions, consumers, and commands that govern the change.

For a module seam or architecture decision, call the Skill tool with "codebase-design" rather than inventing TypeScript-specific architecture rules here.

## Shape

- **Honest guarantee (requirement).** State what a type claims and what establishes that claim: compiler structure, a runtime check, or a convention. Treat assertions, brands, `readonly`, and access modifiers according to what they actually enforce. Read [INVARIANTS.md](INVARIANTS.md) whenever a type claims more than its fields show.

- **Alternatives carry their data (requirement).** Represent mutually exclusive states as a discriminated union with each variant's data on that variant. This makes invalid combinations unrepresentable to checked callers and lets a `never` check expose a new local case. Keep a plain boolean when the concept really has two values and carries no variant-specific data.

```ts
export type LoadState =
  | { status: "idle" }
  | { status: "loaded"; value: string }
  | { status: "failed"; error: Error };

export function stateLabel(state: LoadState): string {
  switch (state.status) {
    case "idle":
      return "Idle";
    case "loaded":
      return state.value;
    case "failed":
      return state.error.message;
    default:
      return assertNever(state);
  }
}
```

- **Meaningful absence (requirement).** Test `null` or `undefined` explicitly when `0`, `false`, or `""` is valid. Distinguish omission, present `undefined`, and `null` when the API assigns them different meanings. A truthiness shortcut is a defect only when it loses a meaningful falsy value.

- **Sparse lookup (requirement).** Distinguish a complete finite-key table from a sparse dictionary. Do not present `Record<string, T>` as evidence that an arbitrary runtime key exists. Pick a representation and return type that expose absence, then handle it before use.

- **Named brand (default).** Introduce a branded primitive only for a realistic role mix-up or a value property established by one checked parser or constructor. Keep ordinary strings and numbers ordinary when the wrapper prevents no named mistake. A brand records evidence for checked callers; it is not runtime validation or authorization.

## Boundaries

- **Unknown in, checked value out (requirement).** Accept untrusted JSON, request data, configuration, environment values, storage rows, and unsafe dependency output as `unknown`. Validate or decode the properties this operation needs before returning the checked representation. Reuse the repository's schema library, generated decoder, or a small parser rather than adding a package by default.

```ts
export function parseApiUser(input: unknown): ApiUser {
  if (!isRecord(input)) {
    throw new TypeError("user must be an object");
  }
  if (typeof input.name !== "string") {
    throw new TypeError("user.name must be a string");
  }
  if (typeof input.loginCount !== "number" || !Number.isInteger(input.loginCount)) {
    throw new TypeError("user.loginCount must be an integer");
  }
  if (typeof input.enabled !== "boolean") {
    throw new TypeError("user.enabled must be a boolean");
  }
  return {
    name: input.name,
    loginCount: input.loginCount,
    enabled: input.enabled,
  };
}
```

- **Unsafe dependency adapter (requirement).** Contain `any`, inaccurate declarations, unchecked deserialization, and third-party assertions in the smallest adapter that can establish the required facts. Return checked project types so unsafe flow does not spread through the core.

- **Visible assertion evidence (default).** Use an assertion only when the compiler cannot express a relationship already established by nearby code or an external contract. Keep it narrow and put the evidence at the assertion site. An unchecked helper that hides `as` provides no stronger guarantee.

- **Operator by job (default).** Use `satisfies` to check an authored value while retaining useful expression-specific information, an annotation when deliberate widening is wanted, and `as const` for literal and readonly inference. None validates runtime data. `satisfies` can contextually affect inference and does not make an existing object exact.

## Functions

- **Local inference (default).** Infer obvious locals and contextually typed callbacks. Repeating visible types adds noise without strengthening a contract.

- **Deliberate API contract (default).** Annotate parameters and important exported returns when the annotation protects a stable interface, prevents accidental inference changes, or improves declarations or checker performance. Preserve intentional inferred APIs such as schema-driven routers and generic factories.

- **Relational generic (default).** Add a type parameter only when it relates inputs, outputs, or members. Use the fewest useful parameters, keep constraints close to their use, and shape parameters so inference works from caller values. Name complex reusable type expressions and test their accepted and rejected use.

- **Union before overload (default).** Use a union parameter when all alternatives have the same return contract. Keep overloads when inputs and outputs are genuinely correlated or a public compatibility surface requires them.

- **Two-sided predicate (requirement).** A custom `value is T` predicate must be true for members of `T` and false for nonmembers under the claimed contract. Test positive and negative behavior. Prefer ordinary narrowing or inferred predicates when the supported TypeScript version can express the check.

## Mutation

- **Readonly permission (default).** Accept `readonly T[]` or `ReadonlyArray<T>` when a function only reads a collection, and expose readonly properties when callers must not mutate through that interface. `readonly` is shallow and another writable alias can still change the value.

- **Deliberate snapshot (requirement).** When behavior requires a stable snapshot, choose encapsulation, copying, or runtime immutability and test that behavior. A deep readonly mapped type or brand alone does not establish ownership, deep freezing, or freedom from aliases.

## Errors

- **Unknown caught value (requirement).** Narrow a caught value before reading `message`, `code`, or custom fields. JavaScript can throw any value.

- **Preserved cause (default).** When adding context, preserve the original error as `cause` or through the ecosystem's established error mechanism. Match stable error codes rather than message text where the runtime supplies them.

- **Expected alternative (default).** Return a discriminated outcome when callers must handle expected alternatives as data. Preserve throwing when the framework or public API defines failure that way. Do not add a result library merely to imitate another language.

- **Failure stays failure (requirement).** Do not swallow a failure into a success-shaped default unless that fallback is the documented contract. A best-effort operation owns its reporting and makes the fallback visible.

- **Owned promise (requirement).** Await or return work whose completion belongs to the operation. Give deliberately detached work an explicit rejection handler and lifecycle owner. For async callbacks, cancellation, concurrency, or resources, read [RUNTIME.md](RUNTIME.md).

- **Observed cleanup (requirement).** Await a promise when a surrounding `catch` or `finally` must observe its rejection or completion. `return await` is correct in that control flow and remains a style choice where no surrounding scope observes it.

## Surface

- **Runtime-shaped modules (requirement).** Match imports and compiler settings to the runtime, emitter, package module mode, and declared consumers. A `paths` alias does not rewrite emitted imports, and a bundler-valid extensionless import can fail in Node ESM. Read [TOOLING.md](TOOLING.md) before module or build changes.

- **Type-only dependency (default).** Use type-only imports for dependencies that are genuinely erased. Preserve runtime imports and explicit side effects. Check legacy decorator metadata and the installed compiler and lint behavior before applying this mechanically.

- **House syntax (convention).** Follow the repository for `type` versus `interface`, enum compatibility, exports, array syntax, semicolons, naming, and formatting. Prefer types for unions and type operators; use interfaces where declaration merging or suitable object extension is intended. A formatter owns presentation.

- **Compatibility before cleanup (requirement).** Preserve public APIs, serialized meanings, generated contracts, and framework-required shapes unless the task explicitly changes them. A locally cleaner type is not an improvement when a declared consumer no longer works.

## Write and review

For writing or refactoring, apply only the rules triggered by the change, preserve behavior and compatibility, and keep tooling and dependency changes inside the task's scope.

For review, inspect without editing. Each finding names the rule and kind, shows the concrete unsafe path or tradeoff, and gives the smallest useful rewrite. Accept valid alternatives and justified assertions, interfaces, enums, overloads, and framework contracts. Deduplicate formatter, linter, repository-standard, and general design findings.

## Verify and report

Run the repository's real typecheck plus relevant runtime tests. Run its lint and dedicated type tests where they exist. For public generics or refined types, verify accepted and rejected use separately from runtime behavior. Expand beyond changed files when declarations, emitted modules, or consumers can be affected.

Report every applicable command as passed, failed, or not run. Separate pre-existing failures from failures introduced by the change. Missing tooling is not a pass. Adding packages, enabling strict flags across an established project, replacing its lint stack, or repairing an unrelated backlog requires explicit task scope.

## Pointers

- **A type claims validation, nominal identity, immutability, admission, or persistence compatibility.** Read [INVARIANTS.md](INVARIANTS.md).
- **The code owns promises, concurrency, cancellation, or cleanup.** Read [RUNTIME.md](RUNTIME.md).
- **The task changes checks, compiler options, imports, package output, or consumer support.** Read [TOOLING.md](TOOLING.md).
- **A rule or version-sensitive recommendation is challenged or updated.** Read [SOURCES.md](SOURCES.md).
