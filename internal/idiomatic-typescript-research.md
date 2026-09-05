# Idiomatic TypeScript: research and proposed skill plan

Research date: 2026-09-05. Repository inspected at `c7ff31aa6aa220a9bed8b73f64e50ab1dc392b2e`.

This is a proposal, not an implemented skill or a new repository coding standard. Only this research note was added. Existing files in `internal/` were left unchanged.

## Recommendation

Add **`idiomatic-typescript`** as a **model-invoked, in-progress** baseline, parallel to `idiomatic-rust`. Cover writing, refactoring, and reviewing TypeScript in applications and libraries. Keep the core framework-neutral, including TypeScript in `.tsx`; leave React component design, framework conventions, and deployment configuration outside v1.

Use Rust's instruction-plus-reason format, conditional references, compiled examples, and comparative evaluations. Do not translate Rust's ownership, enum, newtype, error-library, or concurrency prescriptions into TypeScript.

The defining idea should be **honest guarantees**: express what the compiler can check, establish runtime facts at trust boundaries, and state where neither the type nor the check proves enough. TypeScript explicitly favors JavaScript compatibility and productivity over a fully sound type system. This is a language-specific starting point, not a weaker version of Rust. [S1, S2]

## Existing-skill audit

Searched skill names, frontmatter, bodies, companion files, agent instructions, the top-level README, and the plugin manifest for TypeScript and idiom references. There is **no general TypeScript idiom baseline in this checkout**. This conclusion does not claim that no such skill exists elsewhere or in unmerged branches.

| Existing material | Actual scope | Relationship to the proposal |
| --- | --- | --- |
| [`idiomatic-rust`](../skills/in-progress/idiomatic-rust/SKILL.md) | Rust design and coding rules, with invariant, runtime, lint, and crate references | Structural model, not a source of TypeScript rules |
| [`setup-ts-deep-modules`](../skills/in-progress/setup-ts-deep-modules/SKILL.md) | Installs dependency-cruiser and a particular package-boundary convention | Complementary setup tool, not everyday coding guidance |
| [`migrate-to-shoehorn`](../skills/misc/migrate-to-shoehorn/SKILL.md) | Replaces assertions in test fixtures with a specific helper library | Narrow migration tool; not runtime validation or a general safety baseline |
| [`codebase-design`](../skills/engineering/codebase-design/SKILL.md) | Language-independent deep-module vocabulary | Reuse rather than duplicate architecture guidance |
| [`tdd`](../skills/engineering/tdd/SKILL.md) | Test-first workflow, with TypeScript examples in references | Owns the workflow; the new skill adds TypeScript-specific verification concerns |
| [`pragmatic-programming`](../skills/in-progress/pragmatic-programming/SKILL.md) | General correctness, design, and craft principles | Keep overlapping findings deduplicated |
| [`implementer`](../agents/implementer.md), [`craft-reviewer`](../agents/craft-reviewer.md) | Explicitly load the Rust baseline for Rust work | Natural integration points for the TypeScript equivalent |

The Rust [`evaluation guide`](../skills/in-progress/idiomatic-rust/evals/README.md) is especially useful: it separates correctness, maintainability, review false positives, cost, and rule adherence. Its loaded-skill experiments do not test automatic invocation, so TypeScript needs that separate check too.

## Source-backed findings

The facts below come from the linked primary sources. The proposed rules are a synthesis of those facts, not claims that TypeScript's maintainers endorse this exact skill. A documented repository convention overrides a proposed default. It cannot make an erased type check into runtime validation.

### 1. Establish runtime facts at trust boundaries

**Proposed requirement:** accept untrusted data as `unknown`, validate or decode it before use, and return the checked representation. Keep unsafe third-party typing inside an adapter.

Type assertions and non-null assertions perform no runtime checking. `any` disables checks and can leak from dependencies even when source code contains no explicit `any`. Neither `JSON.parse(...) as User` nor an unchecked `fetchJson<T>()` establishes that the payload is a `User`. [S3, S4, S12]

Use an existing schema library, generated validator, or small checked parser as appropriate. Zod demonstrates both throwing and result-returning validation and deriving static types from schemas. A schema transform can have different input and output types; name the right one. A parser already establishing the required facts needs no second validator merely to follow the skill. [S13]

**Limit:** validation establishes the checked properties at that boundary. It does not prove authorization, freshness, later mutation safety, or compatibility with every future wire format. This limitation follows from structural typing, erasable types, and aliasing. [S1, S2, S5]

### 2. Model alternatives, not bags of optional fields

**Proposed default:** use discriminated unions for mutually exclusive states, with the data belonging to each state in that variant. Use a plain boolean when the concept really is a boolean.

The Handbook shows why a tagged union narrows correctly while a single interface with optional `radius` and `sideLength` requires unsafe assertions. A `never` check can make a newly added case fail compilation. Typed ESLint can also check exhaustiveness. [S6, S14]

**Limit:** a closed local union does not constrain an external server's future messages. Define unknown-version behavior at the decoder. A runtime fallback may be deliberate; it should not silently mask unhandled local variants. [S14]

### 3. Name absence and sparse lookup honestly

**Proposed requirement:** preserve meaningful `0`, `false`, and empty-string values when checking for absence. Distinguish an omitted field from a present `undefined` or `null` where the API cares.

Truthiness checks can accidentally discard valid values. `exactOptionalPropertyTypes` makes omitted properties distinct from explicitly assigned `undefined`. `noUncheckedIndexedAccess` adds possible absence to unchecked indexed reads. Neither flag is enabled merely by setting `strict` in the probed compiler. [S6, S7; probes below]

**Proposed default:** distinguish a complete finite-key lookup from a sparse dictionary. Do not use `Record<string, T>` as evidence that an arbitrary runtime key exists. Choose a lookup representation and return type that expose absence, and check it before use. [S2, S7]

### 4. Infer locally; state deliberate API contracts

**Proposed default:** infer obvious locals and contextually typed callbacks. Annotate parameters and important exported contracts when the annotation protects a stable interface or prevents accidental inference changes.

TypeScript recommends using inference rather than annotating every variable. Return annotations can document a contract or prevent accidental changes, but are not universally necessary. Named annotations can also reduce expensive inference and declaration output in demonstrated compiler bottlenecks. [S3, S8]

**Limit:** generated routers, schema-driven APIs, and generic factories can intentionally expose inferred types. A blanket explicit-return-type rule would damage some of the ecosystem's useful patterns.

### 5. Use `satisfies`, annotations, and assertions for different jobs

**Proposed default:** use `satisfies` to check an authored value against a contract while retaining useful expression-specific information. Use an annotation when deliberate widening is wanted. Use `as const` when literal and readonly inference is wanted.

`satisfies` was introduced in TypeScript 4.9. It is not a validator, does not freeze values, and is not an exact-object mechanism for arbitrary existing values. Its contextual typing can affect inference, despite simplified descriptions saying it never changes an expression's type. The TypeScript issue tracker labels the boolean-literal example working as intended. [S3, S9, S10; probes below]

A justified assertion may be appropriate where the compiler cannot express a checked relationship. Keep its scope narrow and its evidence visible. Replacing `as` with an unchecked helper only hides the assertion.

### 6. Make generics express a relationship

**Proposed default:** introduce type parameters to relate inputs, outputs, or members. Prefer inference-friendly parameters and the fewest useful constraints.

The Handbook explicitly recommends fewer type parameters, pushing parameters down, and reconsidering parameters that appear only once, including in the inferred return type. It also prefers union parameters over overloads when the union expresses the same API. [S4]

**Limit:** correlated input/output overloads and library-level conditional or mapped types can be justified. Name complex reusable types, test their behavior, and investigate checker performance before claiming an abstraction is too expensive. Do not implement elaborate type-level machinery for a relationship ordinary code can express. [S8]

### 7. Treat user-written predicates as code to verify

**Proposed requirement:** test the positive and negative behavior of a custom type predicate. Prefer ordinary narrowing or inferred predicates where they express the intended check.

A declaration `value is T` is not proof that its implementation recognizes `T`. The TypeScript 5.5 notes explicitly say manual predicates are no safer than assertions and explain their two-sided semantics: true means `T`, false means not `T`. A predicate for a small number cannot honestly claim only `value is number` when larger numbers return false. [S11]

TypeScript 5.5 can infer predicates for suitable expressions such as `.filter(x => x !== undefined)`. Version-gate this advice instead of generating legacy helpers automatically. [S11]

### 8. Make mutation permissions explicit without pretending to have ownership

**Proposed default:** accept `readonly T[]` or `ReadonlyArray<T>` when a function only reads a collection. Keep mutations local or behind the interface responsible for maintaining the invariant.

`readonly` communicates and checks access intent. It does not freeze an object, make its nested values immutable, or prevent another writable alias from changing it. Readonly arrays also do not make their element objects deeply immutable. [S5]

**Limit:** neither a deep mapped type nor a brand establishes exclusive ownership. Where a stable snapshot matters, choose encapsulation, copying, or suitable runtime immutability deliberately and test the claimed behavior. Mutation inside a small implementation can be simpler than repeated immutable rebuilding.

### 9. Introduce brands only for a named mistake

**Proposed default:** use a branded primitive or opaque API when it prevents a realistic role mix-up or records a checked value property. Keep ordinary text and numbers ordinary when a wrapper adds no protection.

TypeScript is structurally typed. Branded types simulate nominal distinctions, but the brand is static-only and affects no runtime result. A checked parser or constructor establishes the value property; the brand records it for checked callers. Assertions and `any` can bypass it. [S2, S13]

**Limit:** a brand is not a security boundary. For mutable aggregates, audit aliases and mutation routes. For authorization or version-sensitive admission, enforce the current policy at the authoritative operation rather than treating an old brand as perpetual permission. [S1, S2, S5]

### 10. Match errors to the calling ecosystem

**Proposed requirement:** handle unknown caught values safely, preserve the original cause when adding useful context, and avoid swallowing failures into success-shaped defaults.

Thrown values are not necessarily `Error` objects. Node supports exceptions, promise rejections, callbacks, and error events. `Error` supports a `cause`; Node's documentation advises matching its stable error codes rather than message text. [S7, S15]

**Proposed default:** return a discriminated outcome when the caller must handle expected alternatives as data. Preserve a framework's throwing convention when that is its contract. Zod itself offers both `.parse` and `.safeParse`, illustrating that both shapes can be legitimate. There is no basis here for importing Rust's mandatory `Result<Outcome, Error>` pattern or requiring a new result library. [S13, S15]

### 11. Own promise completion and failure

**Proposed requirement:** await or return work whose completion belongs to the operation. Give deliberately detached work an explicit failure handler and lifecycle owner.

`no-floating-promises` and `no-misused-promises` detect different problems. The former's default permits `void task()`, but its docs explicitly warn that `void` does not handle rejection or change runtime behavior. A promise-returning callback can also land in a `void`-returning slot where its caller ignores the result. [S16, S17]

Use sequential iteration when order or rate requires it, and deliberate concurrency otherwise. `Promise.all` aggregates results and rejects on failure; it does not cancel sibling operations. Bound fan-out when inputs can exceed a dependency's capacity. The capacity rule is a workload-dependent design recommendation, not a promise API guarantee. [S18; probe below]

### 12. Keep cleanup and cancellation in the actual control flow

**Proposed requirement:** await a returned promise when a surrounding catch or cleanup scope must observe its completion. Do not mechanically remove `return await`.

Typed ESLint documents the rejection and cleanup differences between `return promise` and `return await promise`; ordinary contexts can remain a style choice. It also rejects the old performance folklore behind blanket `no-return-await` advice. [S19]

**Proposed default:** pass an `AbortSignal` through cancelable I/O rather than inventing cancellation flags. Cancellation is cooperative and applies only to APIs that honor the signal. Own timer and listener cleanup, including non-abort completion paths; `{ once: true }` only removes an abort listener when that event fires. [S20]

### 13. Make module checking match the runtime and consumers

**Proposed requirement:** inspect the runtime, emitter, package module mode, and consumer targets before changing module settings or imports.

Bundled applications, JavaScript emitted for Node, directly executed TypeScript, and published libraries do not share one universal `tsconfig`. A `paths` alias changes TypeScript resolution, not emitted imports. An extensionless import accepted by a bundler may fail in a published Node ESM package. Libraries need their emitted JavaScript and declarations checked against supported consumers. [S21, S22]

Use type-only imports for genuinely type-only dependencies. Preserve runtime imports and explicit side effects when needed; legacy decorator metadata is a documented caveat. `verbatimModuleSyntax` and `consistent-type-imports` overlap but are not interchangeable in every detail. [S7, S23]

Node's built-in type stripping performs no type checking and ignores `tsconfig.json`. Syntax support is version-sensitive, and `.tsx` is not supported by that execution path. Successful direct execution is therefore not the typecheck gate. [S24]

### 14. Separate semantic guidance from mechanical checks

**Proposed default:** retain a short semantic core and disclose tooling details. Use the project's existing package manager, compiler, lint configuration, and formatter.

When establishing tooling for a new strict project, `recommendedTypeChecked` is a defensible typed ESLint starting point. `strictTypeChecked` is more opinionated and may change outside major versions; `all` is explicitly discouraged. Typed linting requires project information and has a cost. Formatting belongs to the existing formatter, not repeated prose rules. [S25, S26]

Candidate optional additions are `switch-exhaustiveness-check` and the error-handling-correctness setting of `return-await`, after verifying installed-version support. Keep unsafe-value flow, promise handling, and suppression policies mapped to their actual rules. Do not assume every candidate is already included in the chosen preset. [S12, S14, S16, S17, S19, S27]

**Hard boundary:** applying this skill does not authorize installing packages, enabling strict flags across an old repository, replacing its lint stack, or repairing an unrelated backlog.

### 15. Test runtime behavior and type contracts separately

**Proposed requirement:** run the repository's real typecheck as well as relevant runtime tests. For public generics and refined types, add type-level tests for accepted and rejected use.

Vitest's type-testing mode invokes a compiler; its type-test files are not inherently executed as runtime tests. Type assertions embedded in an ordinary transpiled test run are not evidence that type tests ran. [S28]

Use negative examples with an explained `@ts-expect-error` or the repository's existing type-test facility. Check that the expected failure is the intended one: an unrelated typo can satisfy `@ts-expect-error`. Runtime tests should cover malformed input, meaningful falsy values, aliasing, rejection propagation, and cancellation where relevant. [S27, S28]

## What should remain a choice

| Question | Proposed treatment |
| --- | --- |
| `type` or `interface`? | Follow the repository. Use types for unions and type operators; use interfaces where declaration merging or suitable object extension is intended. Avoid a blanket ban on either. [S3, S8] |
| String union or enum? | Prefer a literal union for a plain closed set in new framework-neutral code. Preserve enum-based contracts and generated code; inspect runtime emission and execution-path restrictions. [S3, S24] |
| Classes or functions? | Follow the actual encapsulation and framework needs. Do not translate Rust structs into mandatory classes. [S1, S2] |
| Zod, another validator, or a handwritten parser? | Reuse the installed approach and required guarantees. Zod is a source of concrete examples here, not a mandatory dependency. [S13] |
| Exceptions or result values? | Match the public contract and whether callers need structured alternatives. [S13, S15] |
| Named exports, default exports, array syntax, semicolons? | Repository or formatter conventions, not universal correctness findings. Module compatibility still matters. [S21, S26] |
| Maximum parameters, line counts, comments per guard? | No arbitrary thresholds in v1. Require a concrete readability or maintenance consequence. |

## Verification performed during research

Fetched primary documentation concurrently and read the relevant sections. No background-agent tool was available, so synthesis and verification were performed in this session. Sources are live documents and may change after the research date. Runtime-specific advice must be rechecked against the target project's supported versions.

Ran eight isolated probes using **TypeScript 5.9.3** and **Node v25.6.1**. This is a pinned verification baseline, not a claim that 5.9.3 is the newest TypeScript release.

| Probe | Observed result |
| --- | --- |
| Empty-array indexing under `strict` | Accepted; `noUncheckedIndexedAccess` additionally rejects assignment to `string` |
| `{ name: undefined }` assigned to `{ name?: string }` | Accepted under `strict`; rejected with `exactOptionalPropertyTypes` |
| Incorrect `as` and explicit `value is T` | Both compile; the wrongly asserted value fails when used at runtime |
| Mutation through a writable alias | Compiles and changes what the readonly view observes |
| `satisfies` boolean inference and extra properties | Boolean literal can stay narrow; an existing value can retain additional properties |
| `.filter(x => x !== undefined)` | Inferred result is assignable to `number[]` |
| Returned promise inside local `try/catch` | Bare return bypasses local rejection handling; `return await` reaches it |
| Sibling work after `Promise.all` rejection | Sibling remains runnable and completes when its controlled gate opens |

All eight passed their expected assertions. Compiler package: [TypeScript 5.9.3 registry metadata](https://registry.npmjs.org/typescript/5.9.3). Source cache, scratch probe script, and results are at `/tmp/idiomatic-typescript-research/`. These are temporary research artifacts, not a committed evaluation suite. Step 2 below turns the relevant cases into durable fixtures.

The `void` rejection warning was verified against the linter's primary documentation, not a separate runtime probe. No model comparison, repository-wide typecheck, framework evaluation, or claim of skill effectiveness was made.

## Proposed skill design

### Contract and invocation

Suggested description:

> Idiomatic TypeScript with honest type and runtime guarantees. Use when writing TypeScript, refactoring existing TypeScript, reviewing changes to .ts, .tsx, .mts, or .cts files, designing a TypeScript API or validation boundary, or when another skill needs the TypeScript idiom baseline.

Changes to TypeScript build or module configuration in a TypeScript project should also trigger the tooling branch. A generic `package.json` change in an unrelated project should not.

Omit `disable-model-invocation` and the OpenAI implicit-invocation policy block. Supply `agents/openai.yaml` UI metadata. Humans can still invoke `/idiomatic-typescript`. This matches the repository's [invocation policy](../.agents/invocation.md).

### Rule format

Give each rule a stable name and explicit kind, improving on requiring the reader to infer the kind:

- **Requirement:** names the applicable contract and concrete failure. A finding must demonstrate the unsafe path or violated guarantee, not merely point at a keyword.
- **Default:** recommends a sound choice, explains its benefit, and names when a different choice is appropriate.
- **Convention:** concerns presentation and is advisory unless the repository makes it a standard.

Each entry should carry an instruction and reason. Add applicability, limits, and a small verified example where those change the decision. A rule's source and verification method live in the companion source ledger rather than making every core bullet a miniature literature review.

### Files

```text
skills/in-progress/idiomatic-typescript/
  SKILL.md
  INVARIANTS.md
  RUNTIME.md
  TOOLING.md
  SOURCES.md
  agents/openai.yaml
  examples/
  evals/
```

| File | Contents and read trigger |
| --- | --- |
| `SKILL.md` | Short environment check; core rules organized as Shape, Boundaries, Functions, Mutation, Errors, Surface; writing/review modes; verification gate. Initial target: roughly 20 to 25 rules and 1,200 to 1,800 words, adjusted by evidence rather than padding. |
| `INVARIANTS.md` | Read when choosing or reviewing a claimed guarantee: structural versus nominal distinctions, validation, aliases, predicates, intrinsic versus aggregate properties, changing policy, and compatibility at persistence boundaries. Include when a primitive is enough. |
| `RUNTIME.md` | Read for async work or resources: promise ownership, sequencing, bounded concurrency, failure propagation, cancellation, and cleanup. Keep runtime- and version-specific assumptions explicit. |
| `TOOLING.md` | Read for checks, compiler options, or module/build changes: discover existing commands; separate compiler/linter/test roles; conditional Node/bundler/library guidance; optional lint profiles; checker performance investigation. |
| `SOURCES.md` | Rule-to-source ledger with applicability, version assumptions, and links to verifying examples. Read when challenging a rule or updating a version-sensitive recommendation. |
| `examples/` | Small standalone checked examples, runtime tests, and negative type tests. Every nontrivial snippet comes from these files. Tooling remains local to the skill fixtures, not a new TypeScript dependency at repository root. |
| `evals/` | Independent acceptance tests, task prompts, rubrics, scorer tests, and concise published results. Implement only the harness needed for the initial scenarios. |

Do not add a `LIBRARIES.md` catalogue in v1. It would invite package prescriptions before there is evidence they improve the skill. Likewise, create additional type-performance or framework references only when an actual branch earns them.

### Execution modes

1. **Orient:** inspect repo instructions, neighboring code, runtime/compiler support, relevant configuration, and existing check commands. Done when the agent knows which contracts and environment govern the change.
2. **Write/refactor:** apply relevant rules, preserve behavior and compatibility, and avoid unrelated changes. Load companions only on their triggers.
3. **Review:** inspect the changed code without editing it. Name the rule, kind, concrete consequence or tradeoff, and smallest useful rewrite. Deduplicate mechanical findings and overlapping general principles.
4. **Verify/report:** run existing typecheck, applicable lint, runtime tests, and type tests. Expand beyond changed files when their consumers can be affected. Report each command as passed, failed, or not run; distinguish a pre-existing failure from a newly introduced one. Missing tooling is not a pass.

## Build sequence and completion gates

### Step 1: Agree scope and the rule inventory

Approve the framework-neutral scope, the in-progress bucket, and the model-invoked role. Turn the findings above into a rule matrix: name, kind, trigger, consequence, alternative, source, mechanical enforcement, and example.

**Done when:** every proposed rule has a source or is explicitly labeled a design recommendation; no preference masquerades as a defect; overlap with existing skills is assigned to one owner.

### Step 2: Verify the highest-risk rules before drafting around them

Create an isolated fixture package with a pinned compiler and runtime. Start with validation versus assertion, optional/indexed access, readonly aliasing, predicate correctness, `satisfies` inference, exhaustiveness, and async cleanup. Add a Node-emitted consumer smoke test alongside a bundler-oriented fixture.

**Done when:** intended valid examples compile and run; invalid uses fail for the intended reasons; regression tests catch the planted faults; the harness distinguishes infrastructure failure from a clean result.

### Step 3: Write the core and disclosed references

Use [`writing-for-agents`](../skills/productivity/writing-for-agents/SKILL.md): retain universal instructions in the core, disclose branch-specific details, co-locate caveats, and remove facts available by reading one project config file. Write instruction-plus-reason prose, with no em dashes.

**Done when:** every snippet matches tested source; each pointer names its trigger; all rules have a kind; wording does not overclaim runtime protection or trigger automatic configuration migrations.

### Step 4: Evaluate correctness, restraint, and review quality

Start with these independent scenarios:

| Scenario | What must improve |
| --- | --- |
| Untrusted API/config data | Reject malformed input and preserve valid falsy values; no cast-based fake validation |
| State extension and sparse lookup | New variants force handling; missing keys remain explicit; old behavior survives |
| Small public generic API | Useful inference, rejected invalid calls, small readable signatures, no unnecessary type parameters |
| Async worker or batch operation | Completion and rejection handling, bounded fan-out where specified, cleanup and cancellation correctness |
| Published package consumed by Node and a bundler | Runtime imports and declarations work for the declared consumers |
| Ordinary well-written module with one defect | Fix the defect without gratuitous brands, classes, packages, or API churn |
| Review-only diff mixing defects and valid alternatives | Find real problems while accepting justified interfaces, enums, assertions, and framework contracts |

Run bare and skill-loaded arms on identical prompts and fixtures with pinned toolchains. Use at least three repetitions for the critical scenarios before making effectiveness claims. Keep acceptance tests and reference solutions outside the agent's task directory. Test automatic invocation separately from forced loading, including a non-TypeScript negative case.

**Done when:** publish correctness results, review false positives, API/dependency churn, changed-line counts, and time/token cost. Do not equate more rule citations or more brands with better code. Paid evaluation runs require a separately agreed budget.

### Step 5: Integrate the beta skill

- Add a linked one-line entry to `skills/in-progress/README.md`.
- Update `skills/engineering/ask-jorge/SKILL.md` and re-sync `docs/engineering/ask-jorge.md`. Both currently count and describe two baselines; the new skill changes that map.
- Add conditional TypeScript invocation to `agents/implementer.md` and `agents/craft-reviewer.md`, parallel to Rust. Generalize the craft reviewer's Rust-only deduplication wording.
- Add a minor changeset following the existing in-progress baseline precedent.
- Leave `ask-matt` and `docs/engineering/ask-matt.md` byte-identical.
- Do not add the beta skill to the top-level README, plugin skills array, or a new human-facing docs page. Those are for promoted buckets only.
- Re-run `scripts/link-skills.sh` after adding the skill. If the agent files were already linked, their edits flow through the existing links; otherwise link them for integration testing.

**Done when:** metadata and invocation agree; local links and router pointers resolve; TypeScript work reaches the correct baseline; promoted packaging remains unchanged.

Promotion is a later decision. It would move the skill into `engineering/`, add its linked top-level and bucket entries, add the plugin entry and human docs page, update routing and links, and require `claude plugin validate . --strict` after manifest edits. Do not promote solely because the prose is finished.

## Decisions to confirm

1. **Scope:** recommend framework-neutral application and library TypeScript, with generic `.tsx` guidance but no React-specific playbook in v1.
2. **Opinion level:** recommend strict correctness guidance and contextual defaults, without mandatory Zod, result libraries, brands, or `type`/`interface` preferences.
3. **Delivery:** recommend a beta core plus verified examples first, then funded model evaluations and integration. Promotion waits for evidence.

## Primary source ledger

All sources below were accessed on 2026-09-05. GitHub branch documents and live websites are not immutable; version-sensitive examples must stay pinned in the eventual fixtures.

- **S1:** [TypeScript design goals](https://github.com/microsoft/TypeScript/wiki/TypeScript-Design-Goals), particularly erasability, recognizable JavaScript, and the soundness non-goal.
- **S2:** [Type compatibility](https://www.typescriptlang.org/docs/handbook/type-compatibility.html), structural typing and soundness limits.
- **S3:** [Everyday types](https://www.typescriptlang.org/docs/handbook/2/everyday-types.html), inference, assertions, literals, interfaces, nullability, and enums.
- **S4:** [More on functions](https://www.typescriptlang.org/docs/handbook/2/functions.html), generics, overloads, `unknown`, and function assignability.
- **S5:** [Object types](https://www.typescriptlang.org/docs/handbook/2/objects.html), readonly properties and arrays, aliasing, and excess property checks.
- **S6:** [Narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html), truthiness, tagged unions, and exhaustiveness.
- **S7:** TSConfig reference: [`strict`](https://www.typescriptlang.org/tsconfig/#strict), [`noUncheckedIndexedAccess`](https://www.typescriptlang.org/tsconfig/#noUncheckedIndexedAccess), [`exactOptionalPropertyTypes`](https://www.typescriptlang.org/tsconfig/#exactOptionalPropertyTypes), [`useUnknownInCatchVariables`](https://www.typescriptlang.org/tsconfig/#useUnknownInCatchVariables), [`verbatimModuleSyntax`](https://www.typescriptlang.org/tsconfig/#verbatimModuleSyntax).
- **S8:** [TypeScript performance guidance](https://github.com/microsoft/TypeScript/wiki/Performance), interface extension, annotations, named complex types, and measurement.
- **S9:** [TypeScript 4.9: `satisfies`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-4-9.html#the-satisfies-operator).
- **S10:** [TypeScript issue #55189](https://github.com/microsoft/TypeScript/issues/55189), `satisfies` affecting literal inference, labeled working as intended; independently reproduced here.
- **S11:** [TypeScript 5.5: inferred type predicates](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#inferred-type-predicates), including the limitations of explicit predicates.
- **S12:** [typescript-eslint: `no-unsafe-assignment`](https://typescript-eslint.io/rules/no-unsafe-assignment/), including assignment to `unknown` and related unsafe-flow rules.
- **S13:** Zod's own [basic usage](https://zod.dev/basics) and [branded types](https://zod.dev/api#branded-types), runtime parsing, result shapes, input/output inference, and static-only brands.
- **S14:** [typescript-eslint: `switch-exhaustiveness-check`](https://typescript-eslint.io/rules/switch-exhaustiveness-check/), local exhaustiveness and runtime version mismatch caveats.
- **S15:** [Node error documentation](https://github.com/nodejs/node/blob/main/doc/api/errors.md), propagation styles, error codes, and causes.
- **S16:** [typescript-eslint: `no-floating-promises`](https://typescript-eslint.io/rules/no-floating-promises/), including the explicit warning about `void`.
- **S17:** [typescript-eslint: `no-misused-promises`](https://typescript-eslint.io/rules/no-misused-promises/), conditions and void-returning callback contracts.
- **S18:** [ECMAScript: `Promise.all`](https://tc39.es/ecma262/multipage/control-abstraction-objects.html#sec-promise.all), including the `PerformPromiseAll` algorithm; sibling non-cancellation also reproduced here.
- **S19:** [typescript-eslint: `return-await`](https://typescript-eslint.io/rules/return-await/), error-handling and cleanup semantics.
- **S20:** [Node globals documentation](https://github.com/nodejs/node/blob/main/doc/api/globals.md#class-abortcontroller), `AbortController`, `AbortSignal`, timeouts, and listener lifecycle.
- **S21:** [Choosing compiler options](https://www.typescriptlang.org/docs/handbook/modules/guides/choosing-compiler-options.html), application versus library and runtime-specific configuration.
- **S22:** [Module reference: `paths`](https://www.typescriptlang.org/docs/handbook/modules/reference.html#paths), emitted imports, package exports, and workspace limitations.
- **S23:** [typescript-eslint: `consistent-type-imports`](https://typescript-eslint.io/rules/consistent-type-imports/), including decorator metadata and compiler overlap.
- **S24:** [Node TypeScript execution documentation](https://github.com/nodejs/node/blob/main/doc/api/typescript.md), type stripping, unsupported syntax, ignored configuration, and imports.
- **S25:** [typescript-eslint: typed linting](https://typescript-eslint.io/getting-started/typed-linting/), project information and performance costs.
- **S26:** [typescript-eslint: shared configurations](https://typescript-eslint.io/users/configs/), recommended versus strict profiles, stability, and formatting separation.
- **S27:** [typescript-eslint: `ban-ts-comment`](https://typescript-eslint.io/rules/ban-ts-comment/), explained and scoped compiler suppressions.
- **S28:** [Vitest's type-testing guide](https://github.com/vitest-dev/vitest/blob/main/docs/guide/testing-types.md), separate compiler execution and negative-test false positives.
