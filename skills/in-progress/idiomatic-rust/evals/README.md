# idiomatic-rust evaluation

Five scenarios, each run twice on the same model and toolchain. One run has the `idiomatic-rust` skill loaded (arm `skill`), and one has no Rust guidance at all (arm `bare`). The point is a human comparison: read the two results side by side and decide whether the skill changed the code in ways that matter. The mechanical checks in `score.sh` keep the reading honest. They do not replace it.

## The arms

Both arms run headless (`claude -p`) with `--safe-mode`, which turns off every customisation: the user and project `CLAUDE.md`, every installed skill, memory, hooks, and plugins. Auth and the model are unchanged.

- **bare**: exactly that. The model, the prompt, the starting crate, and the Rust toolchain.
- **skill**: the same, plus `SKILL.md` appended to the system prompt the way the Skill tool would inject it. The injection names the skill's base directory, so `RUNTIME.md`, `CRATES.md`, and `LINTS.md` are one `Read` away. One line at the end of the prompt says to follow the skill, including the Check step.

The model is whatever `claude -p` resolves by default (`claude-opus-5` on 2026-09-04); pass a third argument to `run.sh` to override. Each starting crate pins Rust 1.97.1 through `rust-toolchain.toml`, so the agent's `cargo` and the scorer's `cargo` agree.

## The scenarios

| Scenario | Mode | What it exercises | Reading cost |
| --- | --- | --- | --- |
| `s1-ratelimit` | Write a library from a requirements list | Shape, Errors (rejection as an outcome), time as an input, tests | two crates of ~200 lines |
| `s2-refactor` | "Make this crate idiomatic" on a crate with 13 planted drift patterns | Every section; an answer key names the expected rewrite per pattern | two diffs against one start |
| `s3-extend` | Add one feature to a small, decent crate | Errors (outcome vs error), Flow (guards, exhaustive match), Words (tests, docs) | two diffs of ~100 lines |
| `s4-async` | Write a tokio worker pool with graceful shutdown | `RUNTIME.md`: cancellation, task ownership, channels by role, locks | two crates of ~250 lines |
| `s5-review` | Review the `s2` crate without editing it | The reviewing branch of the skill: findings, citations, rewrites | two findings lists |

`s3-extend` and `s1-ratelimit` are the two to read if time is short: the smallest diffs with the clearest rule-level differences.

Each scenario folder holds `prompt.md`, identical for both arms, and a `start/` crate. It also holds a `rubric.md`, or `answer-key.md` for `s2`, that lists the skill entries the scenario can exercise and what to look for under each.

## Running

```bash
./run.sh s1-ratelimit bare      # one arm; the outputs are listed below
./run.sh s1-ratelimit skill
./score.sh s1-ratelimit         # cargo test, the LINTS.md check command, and pattern counts for both arms
./analyze.py summary            # one line per run: exit, wall, turns, cost, skill files read, check and fmt
```

`run.sh` copies `start/` to a throwaway directory under `/tmp/idiomatic-rust-eval/` and runs the agent there with all permissions granted. It then writes `results/<scenario>/<arm>/`:

- `prompt.txt`: the prompt as sent, with the skill arm's extra line.
- `transcript.jsonl`: the `stream-json` transcript. `jq` can list the tool calls from it.
- `stderr.log`: the CLI's stderr.
- `tree/`: the final crate without `target/` and `.claude/`.
- `final-message.md`: the agent's last message. For `s5-review` this is the findings list.
- `meta.json`: exit status, wall time, turns, cost, and one line per tool call.

`analyze.py meta` writes the last two, and `run.sh` calls it. `score.sh` runs on the copied trees. It reads the check flags from the `flags=( ... )` block in `LINTS.md`, so the lint set is written once. It appends `cargo` output to `results/<scenario>/<arm>/score.log`, and a tree that does not compile scores as `BUILD FAILED`, not as zero findings.

Two environment variables adjust a run. `EVAL_WORK_ROOT` moves the throwaway directory (default `/tmp/idiomatic-rust-eval`). `EVAL_LIMIT_SECONDS` caps one run's wall time (default 2400). The `s4-async` skill arm took 26 minutes on 2026-09-04, so the cap is not generous.

## Reading

For each scenario, open `results/<scenario>/bare/tree/src` and `results/<scenario>/skill/tree/src` side by side, with the rubric beside them. For `s2` and `s3`, diff each tree against `start/` instead. For each rubric row mark the arm that did it, both, or neither. A row where both arms do the same thing is a rule the model already follows. That is evidence the sentence is a no-op on this model. A row where only the skill arm does it is the skill earning its load. A row where neither does it is a rule that is not landing as written.

`results/<scenario>/comparison.md` is the place for the verdict per row and for anything the rubric did not anticipate.

`results/` is untracked (see `.gitignore`), so a run's trees, transcripts, and comparisons stay on the machine that ran it. The section below records what each run concluded, so the conclusion outlives the machine.

## Results

### 2026-09-04, `claude-fable-5-1`, one run per arm per scenario

- **Rows only the skill arm did.** Rejection as an outcome enum (s1, s3). `#[must_use]` with a reason on every outcome type (s1, s3, s4). A two-variant enum for the `bool` parameter (s2, s5). Time as a parameter (s1). Newtypes past the config (s1, s3, s4). Guards with a reason comment and `let ... else` (s1, s3). Cancel first with `biased;` and the token in every worker (s4). The test conventions and module docs as a procedure (all). The check command and fmt before handing back (all). Findings that cite the rule: 13 of 13 in s5, against one citation in the bare review.
- **Rows both arms did the same way.** `&str` and `&[T]` parameters. `impl Iterator` returns. No index loops. `?` on I/O. Structured error variants. Imports at the top. Named constants. Exhaustive matches. `CancellationToken` and `JoinSet`. Paused-time tests.
- **The fixture-probe threshold was not met.** The probe's pass criterion was that the bare arm misses at least three of the planted patterns. The bare refactor (s2) fixed 12 of 13, missing only the `bool` parameter, and passed the pedantic check untouched. The bare review (s5) found 12 of 13. On this model the skill earns its load on Shape, Errors, Words, and Check, not on drift.
- **Cost.** The skill arm took 1.7× the turns, 2.3× the wall time, and 2.0× the dollars ($22.17 against $10.94 over the five scenarios).
- **Changes made to the skill from this run.** Cut as no-ops: the `NonZero` count sentence, the second define-an-enum sentence, `sum`/`any`/`find`, and the acronym rule (now a `LINTS.md` row). Cut as a misfire: a follow-the-existing-test-pattern sentence added earlier on the same branch, which kept the fixture's drift names. Added: `Result` with the rejected value at a config or request boundary, and an `mpsc` for a work queue at the edge.
- **Left for the owner.** Whether "The compiler is the guardrail" exempts two free-text `String` parameters such as `(title, body)`, and whether a renewal counter deserves a newtype.
- **Fork issue #6 item 5 (trim each entry to three or four sentences) was not applied as written.** The sentences that produced the differences above are spread through the long entries, and a blanket cap would remove them with the no-ops. The trim applied is the evidence-backed cut listed above.

## Caveats

One run per arm per scenario, so a single run's choices (the `Clock` trait, the `SegQueue`) may not repeat. A weaker model would show a larger gap on the mechanical rows. The skill arm's prompt carried one extra line: "Follow the idiomatic-rust skill ... including its Check step". That line is the cost of having the skill, not a confound.

The skill arm injects `SKILL.md` into the system prompt, so the suite measures what the skill does once loaded. It does not measure whether the skill triggers on its own from its `description`. That needs a run without `--safe-mode`, with the skill installed and nothing else, and a check in the transcript that `SKILL.md` was read. This suite does not run that.

## Why not `claude plugin eval`

The CLI ships `claude plugin eval` with a built-in with/without ablation arm, which is this suite's design. It is early access and gated on this account as of 2026-09-04, and its case format is only partly documented. The scenarios are already `prompt.md` plus a rubric, so porting them to `case.yaml` files once the command is available is a small change.
