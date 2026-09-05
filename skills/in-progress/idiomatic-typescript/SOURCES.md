# Idiomatic TypeScript source and rule ledger

This ledger maps each rule to its trigger, consequence, alternative, mechanical support, and evidence. The skill's requirements and defaults are a synthesis of these sources, not an assertion that TypeScript's maintainers prescribe one architecture. Sources were checked on 2026-09-05. Version-sensitive behavior is pinned in `examples/` to TypeScript 5.9.3.

## Core rule matrix

| Rule | Kind | Trigger and consequence | Sound alternative | Mechanical support | Verified or evaluable example | Evidence |
| --- | --- | --- | --- | --- | --- | --- |
| Honest guarantee | Requirement | A type claims facts its construction or mutation routes do not establish, so checked callers rely on fiction | Narrow the claim or establish the fact at the boundary | Unsafe-flow lint can expose some routes | [`examples/src/invariants.ts`](examples/src/invariants.ts) separates parser evidence, brands, and aliases | S1, S2, S3 |
| Alternatives carry their data | Requirement | Mutually exclusive state is represented by optional fields, permitting invalid combinations | A boolean for a true boolean concept | Narrowing and exhaustiveness checks | [`examples/src/state.ts`](examples/src/state.ts) and the [negative exhaustiveness fixture](examples/negative-types/exhaustiveness.ts) | S6 |
| Meaningful absence | Requirement | Truthiness loses valid `0`, `false`, or empty text | Truthiness when all falsy values mean absent | `strictNullChecks`, exact optional types | [Boundary runtime tests](examples/runtime-tests/boundaries.test.mjs) and [`s6-plain`](evals/scenarios/s6-plain) | S6, S7 |
| Sparse lookup | Requirement | An arbitrary key is typed as present and later dereferenced | Complete `Record<K, T>` for a finite key union | `noUncheckedIndexedAccess` | [Indexed-access negative fixture](examples/negative-types/unchecked-index.ts) and [`s2-state-lookup`](evals/scenarios/s2-state-lookup) | S2, S7 |
| Named brand | Default | Two roles mix up, or a parser-established property needs to travel | A primitive or named object fields | Assignability tests | [`parseUserId`](examples/src/invariants.ts) and the [brand-role negative fixture](examples/negative-types/brand-role.ts) | S2, S13 |
| Unknown in, checked value out | Requirement | External values enter core code through an assertion or generic fiction | Existing schema, generated decoder, or checked parser | Unsafe-flow lint, runtime tests | [`parseApiUser`](examples/src/boundaries.ts) and [`s1-untrusted-data`](evals/scenarios/s1-untrusted-data) | S3, S4, S12, S13 |
| Unsafe dependency adapter | Requirement | `any` or inaccurate declarations spread through core code | A typed dependency that already establishes the facts | Unsafe assignment and call rules | The [`parseApiUser` boundary](examples/src/boundaries.ts) has the same unsafe-in, checked-out seam | S12 |
| Visible assertion evidence | Default | An assertion has no nearby evidence and fails at runtime | Narrowing, parser, or a documented external contract | Assertion search plus review | [`parseUserId`](examples/src/invariants.ts) justifies its assertion; the [erased-guarantee probe](examples/scripts/probes.mjs) does not | S3, S11 |
| Operator by job | Default | `satisfies`, annotation, assertion, or `as const` is used for a guarantee it cannot provide | Pick the operator matching check, widening, or literal inference | Compiler | [`examples/src/generics.ts`](examples/src/generics.ts) and [satisfies negative](examples/negative-types/satisfies-context.ts) and [probe](examples/scripts/probes.mjs) fixtures | S3, S9, S10 |
| Local inference | Default | Obvious local annotations repeat information and obscure the contract | Annotation where inference would be unstable or costly | Compiler | Locals in `indexBy` infer while its signature states the contract | S3, S8 |
| Deliberate API contract | Default | Exported inference changes accidentally or declarations become costly | Intentional inferred schema or factory APIs | Declaration build and API tests | [Examples](examples/tsconfig.build.json) and [`s5-package-consumers`](evals/scenarios/s5-package-consumers) emit declarations | S3, S8 |
| Relational generic | Default | A type parameter appears once and adds no relationship | Concrete, union, or ordinary inferred type | Type tests and performance diagnostics | [`indexBy`](examples/src/generics.ts) plus [`s3-generic-api`](evals/scenarios/s3-generic-api) accepted and rejected calls | S4, S8 |
| Union before overload | Default | Equivalent overloads hide one union-shaped contract | Correlated overloads or compatibility signatures | Compiler and type tests | [`s7-review`](evals/scenarios/s7-review) includes correlated overloads that the rubric accepts | S4 |
| Two-sided predicate | Requirement | A manual predicate lies about its false or true branch | Ordinary narrowing or supported inferred predicate | Runtime and type tests | [`isNonEmptyString`](examples/src/invariants.ts) and its [positive and negative runtime tests](examples/runtime-tests/invariants.test.mjs) | S11 |
| Readonly permission | Default | Callers can mutate through an interface that only needs reads | Mutable input when mutation is the contract | Compiler | [`indexBy`](examples/src/generics.ts) and [`mapConcurrent`](examples/src/runtime.ts) accept readonly arrays | S5 |
| Deliberate snapshot | Requirement | Stable behavior relies on `readonly` despite writable aliases | Copying, encapsulation, or runtime immutability | Runtime alias tests | [`observeReadonlyAlias`](examples/src/invariants.ts) proves the writable-alias path | S5 |
| Unknown caught value | Requirement | Code reads properties from an arbitrary thrown value | Narrow with `instanceof` or a checked shape | `useUnknownInCatchVariables`, unsafe-flow lint | [`startDetached`](examples/src/runtime.ts) reports an `unknown` error without inventing its shape | S7, S15 |
| Preserved cause | Default | Added context discards the original diagnostic chain | Ecosystem-specific causal wrapper | Runtime tests | Review can compare a causal `Error` wrapper with message-only rethrowing | S15 |
| Expected alternative | Default | Callers must branch on an expected outcome hidden in exceptions | Throwing when the framework defines it | Exhaustiveness and API tests | `parseApiUser` legitimately throws; `LoadState` carries expected alternatives | S13, S15 |
| Failure stays failure | Requirement | Failure becomes a success-shaped default and corrupts later decisions | An explicit documented fallback outcome | Runtime tests | [`recoverLocalFailure`](examples/src/runtime.ts) makes its documented fallback visible | S15 |
| Owned promise | Requirement | Work outlives its operation or rejects without an observer | Explicit detached owner with rejection handling | S16 and S17 lint rules | [`startDetached` and `mapConcurrent`](examples/src/runtime.ts), plus their [runtime tests](examples/runtime-tests/runtime.test.mjs) | S16, S17 |
| Observed cleanup | Requirement | A local catch or finally is bypassed by a returned rejection | Bare return where no local scope observes it | Error-handling-aware `return-await` | [`recoverLocalFailure`](examples/src/runtime.ts) and the [async runtime fixtures](examples/runtime-tests/runtime.test.mjs) | S19 |
| Runtime-shaped modules | Requirement | Emitted imports load under the checker but fail for a consumer | Configuration and specifiers for the declared path | Build and consumer smoke tests | [Node](examples/consumers/node) and [bundler](examples/consumers/bundler) consumers plus [`s5-package-consumers`](evals/scenarios/s5-package-consumers) | S21, S22, S24 |
| Type-only dependency | Default | An erased import is retained, or a runtime dependency is erased | Preserve runtime import for side effects or metadata | Compiler and lint, version dependent | [`examples/consumers/node/consumer.ts`](examples/consumers/node/consumer.ts) imports `ApiUser` with `import type` | S7, S23 |
| House syntax | Convention | Presentation diverges from the repository | Repository formatter or existing style | Formatter and lint | [`s7-review`](evals/scenarios/s7-review) treats its interface and enum as valid alternatives | S3, S8, S26 |
| Compatibility before cleanup | Requirement | A rewrite breaks public, generated, framework, or stored contracts | Explicit migration or compatibility adapter | Consumer and historical fixture tests | [`s5-package-consumers`](evals/scenarios/s5-package-consumers) preserves package exports; every scenario pins public names | S21, S28 |

## Runtime recommendations

| Recommendation | Trigger and consequence | Mechanical support | Evidence |
| --- | --- | --- | --- |
| Explicit detached owner | Fire-and-forget work can reject invisibly | `no-floating-promises`, with caveats for `void` | S16 |
| Callback owns its promise | A `void` callback slot ignores async failure | `no-misused-promises` | S17 |
| Deliberate sequencing | Order, rate, or capacity is part of correctness | Runtime concurrency tests | S18 plus workload design recommendation |
| `Promise.all` does not cancel | Sibling work continues after aggregate rejection | Runtime test | S18 and examples probe |
| Cooperative cancellation | A signal only affects APIs that observe it | Runtime cancellation tests | S20 |
| Cleanup on every exit | Timers and listeners leak on success or cancellation paths | Runtime resource tests | S20 |

## Verification rules

| Rule | Kind | Evidence |
| --- | --- | --- |
| Runtime and type checks are separate | Requirement | S27, S28 |
| Negative tests fail for the intended diagnostic | Requirement | S27, S28 and `examples/scripts/negative-types.mjs` |
| Tooling changes stay scoped | Default | Repository compatibility boundary, S21, S25, S26 |
| Performance claims are measured | Requirement before changing an API for speed | S8 |

## Primary sources

- **S1:** [TypeScript design goals](https://github.com/microsoft/TypeScript/wiki/TypeScript-Design-Goals), including erasability and the soundness non-goal.
- **S2:** [Type compatibility](https://www.typescriptlang.org/docs/handbook/type-compatibility.html), structural typing and soundness limits.
- **S3:** [Everyday types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html), inference, assertions, literals, interfaces, nullability, and enums.
- **S4:** [More on functions](https://www.typescriptlang.org/docs/handbook/2/functions.html), generics, overloads, `unknown`, and function assignability.
- **S5:** [Object types](https://www.typescriptlang.org/docs/handbook/2/objects.html), readonly properties, arrays, aliasing, and excess property checks.
- **S6:** [Narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html), truthiness, discriminated unions, and exhaustiveness.
- **S7:** TSConfig reference for [`strict`](https://www.typescriptlang.org/tsconfig/#strict), [`noUncheckedIndexedAccess`](https://www.typescriptlang.org/tsconfig/#noUncheckedIndexedAccess), [`exactOptionalPropertyTypes`](https://www.typescriptlang.org/tsconfig/#exactOptionalPropertyTypes), [`useUnknownInCatchVariables`](https://www.typescriptlang.org/tsconfig/#useUnknownInCatchVariables), and [`verbatimModuleSyntax`](https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax).
- **S8:** [TypeScript performance guidance](https://github.com/microsoft/TypeScript/wiki/Performance).
- **S9:** [TypeScript 4.9 release notes for `satisfies`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-9.html#the-satisfies-operator).
- **S10:** [TypeScript issue 55189](https://github.com/microsoft/TypeScript/issues/55189), contextual effects of `satisfies`, labeled working as intended.
- **S11:** [TypeScript 5.5 inferred predicates](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#inferred-type-predicates), including manual predicate limits.
- **S12:** [typescript-eslint `no-unsafe-assignment`](https://typescript-eslint.io/rules/no-unsafe-assignment/) and its related unsafe-flow rules.
- **S13:** Zod [basic usage](https://zod.dev/basics) and [branded types](https://zod.dev/api#branded-types), used as examples rather than package prescriptions.
- **S14:** [typescript-eslint `switch-exhaustiveness-check`](https://typescript-eslint.io/rules/switch-exhaustiveness-check/).
- **S15:** [Node errors](https://github.com/nodejs/node/blob/main/doc/api/errors.md), failure channels, stable codes, and causes.
- **S16:** [typescript-eslint `no-floating-promises`](https://typescript-eslint.io/rules/no-floating-promises/), including the warning that `void` does not handle rejection.
- **S17:** [typescript-eslint `no-misused-promises`](https://typescript-eslint.io/rules/no-misused-promises/).
- **S18:** [ECMAScript `Promise.all`](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.all).
- **S19:** [typescript-eslint `return-await`](https://typescript-eslint.io/rules/return-await/), local error and cleanup semantics.
- **S20:** [Node `AbortController` and `AbortSignal`](https://github.com/nodejs/node/blob/main/doc/api/globals.md#class-abortcontroller).
- **S21:** [Choosing compiler options](https://www.typescriptlang.org/docs/handbook/modules/guides/choosing-compiler-options.html).
- **S22:** [Module reference for `paths`](https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths).
- **S23:** [typescript-eslint `consistent-type-imports`](https://typescript-eslint.io/rules/consistent-type-imports/).
- **S24:** [Node TypeScript execution](https://github.com/nodejs/node/blob/main/doc/api/typescript.md).
- **S25:** [typescript-eslint typed linting](https://typescript-eslint.io/getting-started/typed-linting/).
- **S26:** [typescript-eslint shared configurations](https://typescript-eslint.io/users/configs/).
- **S27:** [typescript-eslint `ban-ts-comment`](https://typescript-eslint.io/rules/ban-ts-comment/).
- **S28:** [Vitest type testing](https://github.com/vitest-dev/vitest/blob/main/docs/guide/testing-types.md), used for the distinction between compiler and runtime tests.
