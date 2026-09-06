# TypeScript checks and modules

Read this before changing compiler options, lint rules, imports, package output, or declared consumer support. Tooling follows the repository and runtime. Applying this skill does not authorize a migration.

## Discover before prescribing

Inspect `package.json` scripts, lockfiles, every relevant `tsconfig`, lint and formatter configuration, package `type` and `exports`, build tools, test runners, runtime versions, and documented consumers. Run existing commands before changing them when that is affordable, so a pre-existing backlog does not become the change's result.

Classify the execution path:

| Path | What must agree |
| --- | --- |
| Bundled application | Bundler resolution, transformed syntax, runtime target, and test transform |
| JavaScript emitted for Node | Node module mode, emitted import specifiers, package `type` and `exports` |
| Directly executed TypeScript | The runtime's supported erasable syntax and import rules, plus a separate typecheck |
| Published library | Emitted JavaScript, declarations, package exports, and every declared consumer |

There is no universal `tsconfig` for all four paths.

## Separate the checks

- **Compiler:** assignability, narrowing, declarations, module resolution, and emit compatibility.
- **Typed lint:** unsafe value flow, promise misuse, exhaustiveness policy, and project-specific restrictions that need type information.
- **Runtime tests:** actual parsing, falsy values, mutation, failure propagation, cancellation, and consumer behavior.
- **Type tests:** accepted and rejected use of public generics, overloads, brands, and declaration output.
- **Formatter:** presentation.

A transpiling runtime test does not prove the type tests ran. A direct Node TypeScript execution path performs no type checking and can ignore `tsconfig.json`.

## Compiler options are compatibility changes

`strict` is a strong baseline for a new project. It does not currently imply every useful strictness option. In TypeScript 5.9.3, `noUncheckedIndexedAccess` and `exactOptionalPropertyTypes` add checks not enabled by `strict` alone.

Enable a stricter option in an established repository only when the task includes its migration and consumer impact. For changed code, write honest sparse lookups and optional fields even when the repository cannot enable the flag globally yet.

`paths` affects TypeScript's resolver. It does not rewrite emitted imports. Confirm that the bundler, runtime, or package manager resolves the same specifier.

## Module checks follow consumers

For Node ESM emit, use a Node-aware module and resolution mode appropriate to the supported TypeScript and Node versions, and write runtime import specifiers that Node can load. For a bundler-owned application, bundler resolution can accept extensionless imports and syntax the bundler rewrites. Do not publish that assumption as Node-ready output without a consumer smoke test.

A library check includes at least one import of its built output and declarations from each supported consumer class. Test package `exports`, not only relative source imports.

Use `import type` for erased dependencies when compiler behavior and metadata permit it. `verbatimModuleSyntax` and a lint rule for consistent type imports overlap, but they are not universal substitutes. Legacy decorator metadata can make an apparently type-only import relevant at runtime.

## Typed lint is conditional

When creating a new strict setup, typescript-eslint's `recommendedTypeChecked` is a defensible starting point. `strictTypeChecked` is more opinionated and can change outside major versions. The `all` preset is not a stable production contract.

Candidate additions include unsafe-flow rules, `no-floating-promises`, `no-misused-promises`, `switch-exhaustiveness-check`, and the error-handling-aware setting of `return-await`. Verify the installed version and preset contents before claiming a rule is active. Typed linting needs project information and has a performance cost.

A suppression is narrow and explains why the compiler or lint is wrong at that line. For negative type tests, prefer an explained `@ts-expect-error` or the repository's type-test facility, then ensure the intended diagnostic is what satisfies the test.

## Investigate checker performance

Measure before attributing a slow check to a type. Use the compiler's diagnostics or trace tools supported by the installed version. Helpful changes can include naming a repeated complex type, adding a deliberate return annotation, preferring interface extension over large intersections in demonstrated hot spots, and reducing unnecessary distributive work. Preserve public inference and behavior while measuring again.

## Report the gate

Run the existing typecheck, applicable typed lint, runtime tests, type tests, build, and consumer smoke tests. Report each as passed, failed, or not run, with the exact command. Treat a missing command, dependency installation failure, or tool crash as incomplete infrastructure, not a clean result.
