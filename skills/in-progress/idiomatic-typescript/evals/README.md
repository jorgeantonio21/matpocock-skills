# idiomatic-typescript evaluation

Seven scenarios compare identical prompts and pinned fixtures in `bare`, working-tree `skill`, and committed `skill@<git-ref>` arms. The suite measures correctness, review false positives, API and dependency churn, changed lines, and cost. More rule citations are not a success metric.

The fixtures pin TypeScript 5.9.3 and Node 25.6.1. `fixtures/base/` owns the shared package, runtime pin, and compiler configuration. `run.sh` overlays only a scenario's `start/` and exposes a skill copy without `examples/` or `evals/`. Acceptance tests and reference solutions stay outside the agent's task directory and the supplied guidance copy.

This is layout separation, not a filesystem sandbox. `run.sh` bypasses harness permissions, so the original repository remains readable elsewhere on the host. Here, **hidden** means withheld from the supplied task fixtures, not inaccessible. Evaluations that require access isolation need an external sandbox that withholds the original repository.

## Scenarios

| Scenario | Mode | Independent acceptance |
| --- | --- | --- |
| `s1-untrusted-data` | Repair an untrusted configuration boundary | Malformed values reject and valid falsy fields survive |
| `s2-state-lookup` | Extend state and sparse lookup | Failed state works, absence stays explicit, and invalid state shapes fail typecheck |
| `s3-generic-api` | Tighten a small public generic | Exact key inference and rejected non-key callbacks |
| `s4-async-worker` | Implement a bounded batch | Ordering, fan-out, cancellation, limit validation, and rejection propagation |
| `s5-package-consumers` | Repair published ESM output | Node imports the package by name; Node and bundler consumers check its exported declarations |
| `s6-plain` | Fix one defect in a good module | Zero remains meaningful, with a minimality rubric |
| `s7-review` | Review only, with valid alternatives mixed in | Human rubric counts two defects, false positives, duplicates, and source edits |

Each implementation scenario has `prompt.md`, `rubric.md`, `start/`, `reference/`, and hidden `verify/` tests. The review scenario has no reference because the output is the review itself.

## No-model gate

```bash
./check.sh
```

The gate installs the isolated example package, runs its typecheck, runtime tests, negative diagnostic tests, compiler probes, Node and bundler consumer checks, and snippet provenance check. It then pins infrastructure verdicts with a stub `npm` and runs real-compiler regressions for script and configuration bypasses, package exports, and emitted declarations, plus matrix-output tests. Every scenario start and reference must pass its own tests. Hidden acceptance tests must reject `start/` and pass `reference/`.

An install failure, compiler crash, or unexplained command exit scores `INCOMPLETE`, never clean. This separates unavailable infrastructure from a correct result.

The typecheck and runtime columns report the candidate's own scripts. **External acceptance** is the independent gate: it uses a scratch copy with harness-owned scripts and a strict configuration that checks source and acceptance types even if the candidate excludes them or enables `noCheck`. The candidate's module settings, build configuration, package exports, and dependencies remain under test; its saved tree is not rewritten. Passing project scripts alone is not correctness evidence.

For `s7-review`, external acceptance is `n/a`, not a passing ratio. A human must score its review rubric separately.

## Paid runs

Paid model execution is locked until a budget is agreed. After agreement, opt in explicitly:

```bash
EVAL_PAID_RUNS=1 ./run.sh s1-untrusted-data bare
EVAL_PAID_RUNS=1 ./run.sh s1-untrusted-data skill
EVAL_PAID_RUNS=1 ./run.sh s1-untrusted-data skill@<git-ref>
./score.sh s1-untrusted-data
./analyze.py summary
./analyze.py matrix
```

Run all three arms on the same model and toolchain. Use at least three repetitions for critical scenarios before making effectiveness claims. A run records the skill revision, transcript, final message, command metadata, final tree, scores, diff size, and new dependencies under ignored `results/<scenario>/<arm>/r<N>/`.

Read implementation trees or diffs beside each rubric. Keep these questions separate:

1. Did runtime and type acceptance tests pass?
2. Did the API and dependency surface stay as small as the task required?
3. Did the review report either valid alternative as a defect?
4. What changed-line, wall-time, turn, and token or dollar cost bought the result?
5. Which rules changed behavior compared with the bare arm?

## Automatic invocation

Forced skill arms measure behavior after loading, not discovery. Test the description separately, including negative cases:

```bash
EVAL_PAID_RUNS=1 ./run-invocation.sh [model]
```

`invocation/cases.tsv` expects TypeScript source and TypeScript module configuration to invoke the skill. Ordinary JavaScript and prose should skip it. The runner loads project settings only and installs one project-local skill in its temporary directory. Invocation output is ignored under `.invocation-results/`.

## Implemented evidence and pending evidence

As of 2026-09-05, `./check.sh` passes: 13 runtime examples, six intended negative diagnostics, seven isolated compiler probes, both consumer checks, nine snippet provenance checks, scorer and consumer regression tests, matrix-output tests, seven clean starting fixtures, and six acceptance suites that reject `start/` and pass `reference/`.

No paid bare-versus-skill comparison has run, no numeric budget has been approved, and no effectiveness or automatic-invocation claim is made yet. Those results remain the promotion gate.
