# idiomatic-rust evaluation

Nine scenarios, each run in two or more arms on the same model and toolchain. One arm has no Rust guidance at all (`bare`), one has the `idiomatic-rust` skill from the working tree (`skill`), and one has the skill as committed at a named revision (`skill@<git-ref>`), so the merged skill and a revision under review are compared on the same runs. The point is a human comparison: read the results side by side and decide whether the skill changed the code in ways that matter. The mechanical checks in `score.sh` keep the reading honest. They do not replace it.

## The arms

Every arm runs headless (`claude -p`) with `--safe-mode`, which turns off every customisation: the user and project `CLAUDE.md`, every installed skill, memory, hooks, and plugins. Auth and the model are unchanged.

- **bare**: exactly that. The model, the prompt, the starting crate, and the Rust toolchain.
- **skill**: the same, plus the working tree's `SKILL.md` appended to the system prompt the way the Skill tool would inject it. The injection names the skill's base directory, so `INVARIANTS.md`, `RUNTIME.md`, `CRATES.md`, and `LINTS.md` are one `Read` away. One line at the end of the prompt says to follow the skill, including the Check step.
- **skill@<git-ref>**: the same as `skill`, with the skill exported from that commit (`git archive`) instead of the working tree. `skill@9fe7b17` is the skill as merged in fork PR #7, the baseline the revision in this tree is measured against.

Every run records the skill revision it loaded in `skill-revision.txt` (the commit, with a note when the working tree had uncommitted changes), so a result matrix can name what it measured.

The model is whatever `claude -p` resolves by default (`claude-opus-5` on 2026-09-04); pass a third argument to `run.sh` to override. Each starting crate pins Rust 1.97.1 through `rust-toolchain.toml`, so the agent's `cargo` and the scorer's `cargo` agree.

## The scenarios

| Scenario | Mode | What it exercises | Independent check | Reading cost |
| --- | --- | --- | --- | --- |
| `s1-ratelimit` | Write a library from a requirements list | Shape, Errors (rejection as an outcome), time as an input, tests | none | two crates of ~200 lines |
| `s2-refactor` | "Make this crate idiomatic" on a crate with 13 planted drift patterns | Every section; an answer key names the expected rewrite per pattern | none | two diffs against one start |
| `s3-extend` | Add one feature to a small, decent crate | Errors (outcome vs error), Flow (guards, exhaustive match), Words (tests, docs) | none | two diffs of ~100 lines |
| `s4-async` | Write a tokio worker pool with graceful shutdown | `RUNTIME.md`: cancellation, task ownership, channels by role, locks | none | two crates of ~250 lines |
| `s5-review` | Review the `s2` crate without editing it | The reviewing branch of the skill: findings, citations, rewrites | none | two findings lists |
| `s6-decoder` | Extend a decoder library: an encoder, a JSON capture loader, two signed operations | `INVARIANTS.md`: close every route (a byte decoder and a serde derive both bypass `new`), raw-to-validated, an invariant-preserving operation at `i32::MIN` | 10 external tests | two diffs of ~150 lines |
| `s7-scheduler` | Extend an inference request scheduler: a policy change and a model retirement | `INVARIANTS.md`: contextual admission (work admitted under the old policy is kept), an authoritative transition result; one planted off-by-one | 8 external tests | two diffs of ~120 lines |
| `s8-cli` | Bump a CLI's config format: a stricter version 2, a migration, historical files | `INVARIANTS.md`: persistent representation change (a version 1 zero keeps its meaning, a corrupt file is never migrated); the CLI lint relaxation | 9 external tests, through the binary | two diffs of ~200 lines |
| `s9-plain` | "Make the changes this crate needs" on an ordinary, well-written crate with one defect | Restraint: a wrapper, an alias, a dependency, or a typestate would help nothing here; the diff stats say how much was touched | 7 external tests | two diffs, ideally tiny |

`s3-extend` and `s1-ratelimit` are the two to read if time is short on the style rows. `s9-plain` and `s6-decoder` are the two to read for the questions the revision in this tree asks: does the skill stop the agent from wrapping what needs no wrapper, and does it make the agent audit every route into a type.

The three workloads in `s6` to `s8` (a decoder library, a request scheduler, a small CLI) are also a check that the skill does not push one architecture on everything. The rubrics' Generality rows mark a trait, a channel, an async runtime, a lock, or a dependency that the workload did not need.

Each scenario folder holds `prompt.md`, identical for every arm, and a `start/` crate. It also holds a `rubric.md`, or `answer-key.md` for `s2`, that lists the skill entries the scenario can exercise and what to look for under each. A scenario with external tests also holds `verify/tests/*.rs`, which the scorer runs against the result through the public API the prompt pins, and `reference/`, one solution that passes them. The agent never sees `verify/` or `reference/`: `run.sh` copies `start/` alone. A scenario whose start crate plants lint findings on purpose says so in `expect-start-findings`. A scenario that relaxes a lint in both check passes (a CLI and `print_stdout`) lists the flags in `check-flags`.

## Running

```bash
./check.sh                          # no model: examples compile, snippets match, score.sh verdicts hold, fixtures are clean, external tests catch and pass
./run.sh s6-decoder bare            # one arm, one run; a second call adds r2 beside r1
./run.sh s6-decoder skill
./run.sh s6-decoder skill@9fe7b17   # the merged skill, for a three-arm comparison
./score.sh s6-decoder               # every run of every arm: tests, external tests, the check, the diff, pattern counts
./analyze.py summary                # one line per run: exit, wall, turns, cost, skill files read, check and fmt
./analyze.py matrix                 # one Markdown row per scenario and arm over its runs
```

`check.sh` is the gate before a paid run. It builds `examples/` and runs the `LINTS.md` check on it. It checks that every Rust block in `SKILL.md`, `INVARIANTS.md`, `RUNTIME.md`, and `CRATES.md` is a verbatim excerpt of a module there, so every snippet the skill shows compiles, passes a test, and passes the check. It checks that every `start/` passes its own tests and the check, so a run's findings are the agent's. It runs `test_score.py`, which drives `score.sh` with a stub `cargo` and pins each verdict. And it checks that each scenario's external tests fail on `start/` and pass on `reference/`.

`run.sh` copies `start/` to a throwaway directory under `/tmp/idiomatic-rust-eval/` and runs the agent there with all permissions granted. It then writes `results/<scenario>/<arm>/r<N>/`, where `N` is one more than the runs of that arm so far:

- `prompt.txt`: the prompt as sent, with the skill arms' extra line.
- `skill-revision.txt`: the commit the skill arm loaded, or `none`.
- `transcript.jsonl`: the `stream-json` transcript. `jq` can list the tool calls from it.
- `stderr.log`: the CLI's stderr.
- `tree/`: the final crate without `target/` and `.claude/`.
- `final-message.md`: the agent's last message. For `s5-review` this is the findings list.
- `meta.json`: exit status, wall time, turns, cost, and one line per tool call.
- `score.json` and `score.log`: written by `score.sh`.

`score.sh` runs on the copied trees. It reads the check flags from the `flags=( ... )` block in `LINTS.md` (through `flags.sh`, shared with `check.sh`), so the lint set is written once, and appends the scenario's `check-flags` to both passes. Per run it reports the crate's own tests, the external tests (run on a scratch copy, so the tree stays as the agent left it), the check in both passes, the non-blank line count, the lines added and removed against `start/`, and the dependencies added. A tree that does not compile scores `BUILD FAILED`. A run where cargo fails before it produces a diagnostic (no toolchain, a broken manifest, a dependency that did not download), stops before its `build-finished` line, or exits with a status its diagnostics do not explain scores `INCOMPLETE`. Neither is reported as zero findings or as a pass. `test_score.py` runs `score.sh` against a stub `cargo` that answers with a chosen stdout, stderr, and exit status, and pins every verdict, including the `print_stdout` relaxation in both passes.

Two environment variables adjust a run. `EVAL_WORK_ROOT` moves the throwaway directory (default `/tmp/idiomatic-rust-eval`). `EVAL_LIMIT_SECONDS` caps one run's wall time (default 2400). The `s4-async` skill arm took 26 minutes on 2026-09-04, so the cap is not generous.

## Reading

For each scenario, open the arms' `tree/src` side by side, with the rubric beside them. For `s2`, `s3`, and `s6` to `s9`, diff each tree against `start/` instead. For each rubric row mark the arm that did it, both, or neither. A row where both arms do the same thing is a rule the model already follows. That is evidence the sentence is a no-op on this model. A row where only the skill arm does it is the skill earning its load. A row where neither does it is a rule that is not landing as written.

Keep four questions apart, because a run can win one and lose another. Did the external tests pass (correctness)? Did the diff stay the size the task needed (maintainability; `s9-plain` exists for this)? Did a review finding name something that was fine (false positives; the `s5-review` rubric counts them)? What did the run cost? Rule adherence is the fifth question, and the rubric rows answer it. It is not a proxy for the other four.

`results/<scenario>/comparison.md` is the place for the verdict per row and for anything the rubric did not anticipate.

`results/` is untracked (see `.gitignore`), so a run's trees, transcripts, and comparisons stay on the machine that ran it. The section below records what each run concluded, so the conclusion outlives the machine.

## Results

### 2026-09-04, `claude-fable-5-1`, one run per arm per scenario, skill at `9fe7b17`

- **Rows only the skill arm did.** Rejection as an outcome enum (s1, s3). `#[must_use]` with a reason on every outcome type (s1, s3, s4). A two-variant enum for the `bool` parameter (s2, s5). Time as a parameter (s1). Newtypes past the config (s1, s3, s4). Guards with a reason comment and `let ... else` (s1, s3). Cancel first with `biased;` and the token in every worker (s4). The test conventions and module docs as a procedure (all). The check command and fmt before handing back (all). Findings that cite the rule: 13 of 13 in s5, against one citation in the bare review.
- **Rows both arms did the same way.** `&str` and `&[T]` parameters. `impl Iterator` returns. No index loops. `?` on I/O. Structured error variants. Imports at the top. Named constants. Exhaustive matches. `CancellationToken` and `JoinSet`. Paused-time tests.
- **The fixture-probe threshold was not met.** The probe's pass criterion was that the bare arm misses at least three of the planted patterns. The bare refactor (s2) fixed 12 of 13, missing only the `bool` parameter, and passed the pedantic check untouched. The bare review (s5) found 12 of 13. On this model the skill earns its load on Shape, Errors, Words, and Check, not on drift.
- **Cost.** The skill arm took 1.7× the turns, 2.3× the wall time, and 2.0× the dollars ($22.17 against $10.94 over the five scenarios).
- **Changes made to the skill from this run.** Cut as no-ops: the `NonZero` count sentence, the second define-an-enum sentence, `sum`/`any`/`find`, and the acronym rule (now a `LINTS.md` row). Cut as a misfire: a follow-the-existing-test-pattern sentence added earlier on the same branch, which kept the fixture's drift names. Added: `Result` with the rejected value at a config or request boundary, and an `mpsc` for a work queue at the edge.
- **Left for the owner.** Whether a renewal counter deserves a newtype. The two free-text `String` parameters question is answered in the revision: they stay `String` inside a struct with named fields.
- **Fork issue #6 item 5 (trim each entry to three or four sentences) was not applied as written.** The sentences that produced the differences above are spread through the long entries, and a blanket cap would remove them with the no-ops. The trim applied is the evidence-backed cut listed above.

### Harness and fixture changes since that run (2026-09-05)

Results moved from `results/<scenario>/<arm>/` to `results/<scenario>/<arm>/r<N>/`, and `score.sh` gained the external tests, the diff stats, and the `INCOMPLETE` verdict. Two fixtures changed. `s1-ratelimit`'s empty `lib.rs` gained a newline, so `rustfmt --check` passes on it. `s3-extend`'s tests carried two findings of the check command (`duration_suboptimal_units` and `unchecked_time_subtraction`) that the answer key never planted. They are fixed, so a run's findings there are now the agent's. The 2026-09-04 numbers for `s3` include those two findings in the bare arm. `s4-async` gained a `check-flags` file, so the `print_stdout` relaxation its rubric names is applied in both passes.

### Not yet run: the revision in this tree

The revision (`INVARIANTS.md`, the corrected constructors and derives, the contextual hot-path and crate guidance, `s6` to `s9`) has not been run against a model. `check.sh` passes on it: the examples compile and pass the check, every snippet matches its module, and each new scenario's external tests fail on `start/` and pass on `reference/`. What promotion still needs, in the order to run it:

1. `s6` to `s9` in three arms (`bare`, `skill@9fe7b17`, `skill`), at least three runs each, on one model. `analyze.py matrix` prints the result table; paste it here with the model and the date.
2. The same three arms on `s1` to `s5`, so the earlier one-run numbers get repetitions.
3. The real-ticket probe: one maintenance ticket from a real Rust repository, run in the same three arms and scored by that repository's own tests.

Promotion is a decision on that evidence: fewer external-test failures and fewer wrong review findings at a cost the owner accepts, not more rule citations.

## Lint calibration

The evidence behind the lint set in `LINTS.md`. The commands were run on 2026-09-04 against Rust 1.97.1 (clippy 0.1.97), and every lint in the set resolves on that toolchain; `LINTS.md` holds the name check to repeat after a toolchain bump.

Three flags changed as a result of the first run. `module_name_repetitions` is not in the `-A` list, because it is a `restriction` lint since 1.93 and relaxing it under pedantic is a no-op. `mem_forget` is not in the `-D` picks: in a workspace with zero-copy wire types, every one of its findings came from a serialization derive expansion and none from hand-written code, and the lint does not skip external macros. `-A clippy::inline_always` was added, because a low-latency workspace had 110 deliberate uses in one crate.

On a scratch crate, the first command passes an iterator-chain function, fails with `clippy::unwrap-used` on a `parse().unwrap()` in library code, and ignores test code. The second command passes a test module that holds an `unwrap()` and an `assert!` inside a `Result`-returning test. `panic_in_result_fn` fires on `assert!` and `panic!`, not on `debug_assert!` or `unreachable!`. A `clippy.toml` with `allow-unwrap-in-tests` clears the test `unwrap` but not `panic_in_result_fn`, so a config file cannot replace the second run.

On a 40-crate workspace that never ran pedantic, the shipped set reported 254 findings in a small primitives crate. The top three there: `doc_markdown` 51, `cast_lossless` 46, `missing_errors_doc` 35. The largest domain crate reported 407. The top four there: `missing_errors_doc` 65, `doc_markdown` 65, `expect_used` 53, `unwrap_used` 46. `expect_used` was in the set at the time. Each crate took under ten seconds once the dependencies were built. Without `--no-deps`, the run failed on two findings in a proc-macro crate in the build graph before it reached the named crate. The two binaries of the largest crate carried 82 `print_stdout` findings, which is the case the `-A clippy::print_stdout` relaxation is for. Those counts are why the check is diff-scoped. The backlog filter in `LINTS.md` printed 17 findings for one changed file.

## Caveats

The 2026-09-04 results are one run per arm per scenario, so a single run's choices (the `Clock` trait, the `SegQueue`) may not repeat. That is what the `r<N>` layout and the matrix are for. A weaker model would show a larger gap on the mechanical rows. The skill arms' prompt carries one extra line: "Follow the idiomatic-rust skill ... including its Check step". That line is the cost of having the skill, not a confound.

The skill arms inject `SKILL.md` into the system prompt, so the suite measures what the skill does once loaded. It does not measure whether the skill triggers on its own from its `description`. That needs a run without `--safe-mode`, with the skill installed and nothing else, and a check in the transcript that `SKILL.md` was read. This suite does not run that.

The external tests pin the public API the prompt names, so a solution with a different but sound API scores `BUILD FAILED` on them. The prompt is explicit about every name the tests use; the rubric's notes column is the place to record a sound design the tests could not see.

## Why not `claude plugin eval`

The CLI ships `claude plugin eval` with a built-in with/without ablation arm, which is this suite's design. It is early access and gated on this account as of 2026-09-04, and its case format is only partly documented. The scenarios are already `prompt.md` plus a rubric, so porting them to `case.yaml` files once the command is available is a small change.
