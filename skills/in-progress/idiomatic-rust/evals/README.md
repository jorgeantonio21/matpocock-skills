# idiomatic-rust evaluation

Compare no guidance (`bare`), the merged issue-7 skill (`merged`), and the revised skill (`skill`). Use repeated runs on the same resolved model and Rust toolchain. Independent correctness, maintainability, review false positives, generality, cost, and rule adherence are separate dimensions; [RUBRIC.md](RUBRIC.md) defines the evidence for each.

The skill remains **in progress**. Neither the original fixture threshold nor the real-ticket promotion gate has been met. Preserve the historical results below; new mechanical checks are not proof that loading the revised skill improves a model's output.

## Inputs and arms

Each scenario has an identical `prompt.md` and `start/` tree for all arms. Rust 1.97.1 is pinned in the starting crates. The new scenarios also carry checked-in Cargo lockfiles. `run.sh` uses headless Claude Code with `--safe-mode` to disable installed customizations. It currently uses `--dangerously-skip-permissions` in the throwaway workspace, so running a paid campaign needs authorization for both its aggregate budget and that execution mode.

- **bare:** prompt, starting code, and toolchain only.
- **merged:** additionally inject `SKILL.md` and expose its companion references from commit `9fe7b179dd3a8652d1c0fb4d9935011f39d953af`.
- **skill:** additionally inject the revised `SKILL.md` and expose its companion references, including `INVARIANTS.md`.

Both skill arms receive the same instruction to follow the loaded skill and its Check step. `compare.sh` snapshots the revision and evaluator lint policy for the campaign. Every run records hashes of the starting files, effective prompt, skill files, and external oracle. Runtime/toolchain versions are recorded separately. If resolved models or input hashes differ unexpectedly, report an invalid comparison rather than averaging it into a result.

## Scenarios

| Scenario | Workload | Evidence |
| --- | --- | --- |
| s1-ratelimit | Write a rate-limiter library | Historical rule-adherence rubric |
| s2-refactor | Refactor thirteen planted style patterns | Historical answer key; bare already fixed twelve |
| s3-extend | Extend a session store | Historical rule-adherence rubric |
| s4-async | Async worker pool with shutdown | Historical rubric; CLI stdout exception applies in both passes |
| s5-review | Review the original refactor fixture | Historical rule-adherence rubric |
| s6-decoder | Decoder library | Constructor, FromStr, and Serde agree on all u8 inputs; malformed input; signed minimum negation |
| s7-inference | Inference admission scheduler | Prior admission survives policy changes; rejection preserves input; removal returns authoritative children and remaining state |
| s8-cli | Small configuration CLI | Literal historical bytes, migration semantics, corrupt input, new writes, stdout; simple labels need no wrapper |
| s9-review | Review the CLI without edits | Two real semantic findings; false positives on valid simple code counted separately |

The new `oracle.rs` files are kept outside `start/`. After generation, the scorer copies a tree to a temporary directory and runs the external oracle as its own integration-test target. Oracle tests check public behavior and do not require a chosen implementation. Agent-generated tests are counted separately. The review case uses its own [rubric](scenarios/s9-review/rubric.md); verify that its tree remains unchanged.

## Local validation, no model calls

Run from this directory:

```bash
python3 -m unittest discover -s . -p 'test_*.py' -v
cargo +1.97.1 test --manifest-path examples/Cargo.toml --doc --locked
python3 verify_fixtures.py
bash -n run.sh score.sh compare.sh
```

The doctest crate includes the actual Markdown references, so edited examples cannot drift from a copied test. It includes a compile-fail probe demonstrating that a Send stored type may return a non-Send future. `verify_fixtures.py` checks that each new starting crate compiles, that its oracle fails on the semantic bugs, and that minimal repairs pass. Those repairs are fixture controls, not model evaluation results.

## Model runs

Preview the concrete campaign before authorizing it:

```bash
EVAL_DRY_RUN=1 ./compare.sh claude-fable-5-1 2
# 24 runs; maximum requested model budget $48
```

After authorization, the same command without `EVAL_DRY_RUN=1` runs two repetitions of all three arms over s6 through s9. Pass scenario names after the per-run budget to narrow the campaign. `EVAL_REPEATS` must be at least two. Arm order reverses on even repetitions. The per-run wall timeout defaults to 2400 seconds; set `EVAL_LIMIT_SECONDS=300` for a five-minute cap. A budget or timeout termination is incomplete evidence, not a losing correctness score. The campaign stops on an incomplete run.

For individual authorized runs:

```bash
EVAL_MAX_BUDGET_USD=2 ./run.sh s6-decoder bare claude-fable-5-1
EVAL_MAX_BUDGET_USD=2 ./run.sh s6-decoder merged claude-fable-5-1
EVAL_MAX_BUDGET_USD=2 ./run.sh s6-decoder skill claude-fable-5-1
./score.sh s6-decoder
./analyze.py summary
```

`EVAL_RESULTS_ROOT` selects the results directory; `EVAL_WORK_ROOT` selects the throwaway workspace root (default `/tmp/idiomatic-rust-eval`). A run refuses to overwrite an existing transcript. A campaign uses `repeat-N/<scenario>/<arm>/`; single runs use `<scenario>/<arm>/`.

Each run saves `prompt.txt`, `toolchain.txt`, `inputs.json`, the injected `idiomatic-rust/` snapshot when applicable, `transcript.jsonl`, `stderr.log`, `tree/`, `final-message.md`, and `meta.json`. The last two come from `analyze.py meta`. Missing result events, error results, budget exhaustion, and nonzero CLI exits remain incomplete. Unknown cost stays unknown.

`score.sh` calls `score.py`. It reads the base flags from `LINTS.md`, plus the scenario's `lints.json` in **both** Clippy passes, and saves `score.json` and `score.log` per arm. All-target checks additionally relax test panic lints. Rustc errors are `BUILD FAILED`; failed startup, missing compiler JSON, and missing build completion are `INCOMPLETE`. A denied lint on a completed build is counted as a finding. Neither failed infrastructure nor an empty successful stub can score clean. Raw pattern counts are no longer treated as quality scores.

Results are ignored by Git. Publish a compact matrix and enough artifacts to audit each conclusion; do not publish private prompts, credentials, or source-specific investigations. The completed trial's source patch and machine-readable evidence are in [evidence/2026-09-05](evidence/2026-09-05/).

## Results

### 2026-09-04, `claude-fable-5-1`, one run per arm per scenario

- **Rows only the skill arm did.** Rejection as an outcome enum (s1, s3). `#[must_use]` with a reason on every outcome type (s1, s3, s4). A two-variant enum for the `bool` parameter (s2, s5). Time as a parameter (s1). Newtypes past the config (s1, s3, s4). Guards with a reason comment and `let ... else` (s1, s3). Cancel first with `biased;` and the token in every worker (s4). The test conventions and module docs as a procedure (all). The check command and fmt before handing back (all). Findings that cite the rule: 13 of 13 in s5, against one citation in the bare review.
- **Rows both arms did the same way.** `&str` and `&[T]` parameters. `impl Iterator` returns. No index loops. `?` on I/O. Structured error variants. Imports at the top. Named constants. Exhaustive matches. `CancellationToken` and `JoinSet`. Paused-time tests.
- **The fixture-probe threshold was not met.** The probe's pass criterion was that the bare arm misses at least three of the planted patterns. The bare refactor (s2) fixed 12 of 13, missing only the `bool` parameter, and passed the pedantic check untouched. The bare review (s5) found 12 of 13. On this model the skill earns its load on Shape, Errors, Words, and Check, not on drift.
- **Cost.** The skill arm took 1.7× the turns, 2.3× the wall time, and 2.0× the dollars ($22.17 against $10.94 over the five scenarios).
- **Changes made to the skill from this run.** Cut as no-ops: the `NonZero` count sentence, the second define-an-enum sentence, `sum`/`any`/`find`, and the acronym rule (now a `LINTS.md` row). Cut as a misfire: a follow-the-existing-test-pattern sentence added earlier on the same branch, which kept the fixture's drift names. Added: `Result` with the rejected value at a config or request boundary, and an `mpsc` for a work queue at the edge.
- **Left for the owner.** Whether "The compiler is the guardrail" exempts two free-text `String` parameters such as `(title, body)`, and whether a renewal counter deserves a newtype.
- **Fork issue #6 item 5 (trim each entry to three or four sentences) was not applied as written.** The sentences that produced the differences above are spread through the long entries, and a blanket cap would remove them with the no-ops. The trim applied is the evidence-backed cut listed above.

### 2026-09-05, issue 8 revision

Local validation: 13 Python regression tests passed; 11 executable Rust examples and one expected compile failure passed on Rust 1.97.1. The external fixtures rejected the planted semantic bugs and passed all 10 external tests after minimal repairs. This demonstrates useful correctness checks, not a model improvement.

One capped **bare decoder trial** completed on `claude-fable-5-1`: 148 seconds, 9 turns, reported cost $1.0834625, 14 generated tests and 3 external tests passed. It retained the borrowed decoder API and existing dependencies, routed parsing and Serde through checked conversion, and used checked nonzero negation. There was one library lint diagnostic and two all-target diagnostics (`missing_errors_doc`), which do not negate the semantic result. This is another example of bare-model correctness, not evidence for the revised skill.

The trial ran before lockfiles were added to the start trees. Its resolved lockfile is recorded with the evidence, and it is excluded from the planned repeated comparison. The matrix below distinguishes completed evidence from missing work.

| Probe | Bare | Merged | Revised | Conclusion |
| --- | --- | --- | --- | --- |
| Decoder trial | 3/3 independent tests; $1.0834625; 148 seconds | Not run | Not run | One successful infrastructure/correctness trial only |
| Repeated decoder comparison, two runs per arm | Not run | Not run | Not run | Pending authorization |
| Repeated inference comparison, two runs per arm | First attempt interrupted; cost unknown | Not run | Not run | Incomplete |
| Repeated CLI comparison, two runs per arm | First attempt interrupted; cost unknown | Not run | Not run | Incomplete |
| Repeated review comparison, two runs per arm | First attempt interrupted; cost unknown | Not run | Not run | Recall and false positives not measured |
| Real-ticket probe | Not run | Not run | Not run | Still required before promotion |

Automatic approval review rejected the repeated campaign because aggregate spend and unrestricted agent execution had not been explicitly approved. Three sibling runs had started; they and their descendants were stopped. Their incomplete transcripts have no final cost event, so their cost is unknown, not zero. No revised-versus-merged quality or cost ratio can be inferred from these attempts. The next concrete campaign is 24 runs, capped at $2 each ($48 total requested limit) and five minutes each, after approval. The real-ticket probe also needs a selected public ticket and an agreed budget.

The trial patch, scorer output, input hashes, toolchain record, and metadata are published beside this report. Full transcripts stay local; hashes identify them for audit. Source-specific research remains internal and is not a shipped fixture.

## Promotion gate

Complete repeated bare/merged/revised comparisons, review the separate dimensions in [RUBRIC.md](RUBRIC.md), and run its real-ticket probe before promotion. Publish resolved models, exact input revisions, per-run costs, failures, and a result matrix. The original 12-of-13 bare result and 2.0x-dollar/2.3x-time overhead remain relevant evidence. Improvements in rule adherence alone do not justify promotion.

This suite measures behavior after explicit skill loading, not automatic invocation from the skill description. A real-ticket probe is also distinct from these self-contained synthetic fixtures. Neither is claimed complete here.
