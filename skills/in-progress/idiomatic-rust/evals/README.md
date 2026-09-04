# idiomatic-rust evaluation

Five scenarios, each run twice on the same model and toolchain: once with the `idiomatic-rust` skill loaded (arm `skill`) and once without any Rust guidance at all (arm `bare`). The point is a human comparison: read the two results side by side and decide whether the skill changed the code in ways that matter. The mechanical checks in `score.sh` are there to keep the reading honest, not to replace it.

## The arms

Both arms run headless (`claude -p`) with `--safe-mode`, which turns off every customisation: the user and project `CLAUDE.md`, every installed skill, memory, hooks, and plugins. Auth and the model are unchanged.

- **bare**: exactly that. The model, the prompt, the starting crate, and the Rust toolchain.
- **skill**: the same, plus `SKILL.md` appended to the system prompt the way the Skill tool would inject it (with its base directory named, so `RUNTIME.md`, `CRATES.md`, and `LINTS.md` are one `Read` away), and one line at the end of the prompt that says to follow it, including the Check step.

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

Each scenario folder holds `prompt.md` (identical for both arms), a `start/` crate, and a `rubric.md` (or `answer-key.md` for `s2`) listing the skill entries the scenario can exercise and what to look for under each.

## Running

```bash
./run.sh s1-ratelimit bare     # writes results/s1-ratelimit/bare/{tree,transcript.jsonl,meta.json}
./run.sh s1-ratelimit skill
./score.sh s1-ratelimit        # cargo test, the LINTS.md check command, and pattern counts for both arms
```

`run.sh` copies `start/` to `/tmp/idiomatic-rust-eval/<scenario>/<arm>/`, runs the agent there with all permissions granted (it is a throwaway directory), then copies the final tree back under `results/` without `target/`. `score.sh` runs on the copied trees. The transcript is `stream-json`, so `jq` can list the tool calls, which is how you check whether the skill arm read `RUNTIME.md` or ran the check command.

## Reading

For each scenario, open `results/<scenario>/bare/tree/src` and `results/<scenario>/skill/tree/src` side by side (or diff each against `start/` for `s2` and `s3`), with the rubric beside them. For each rubric row mark the arm that did it, both, or neither. A row where both arms do the same thing is a rule the model already follows, which is evidence the sentence is a no-op on this model. A row where only the skill arm does it is the skill earning its load. A row where neither does it is a rule that is not landing as written.

`results/<scenario>/comparison.md` is the place for the verdict per row and for anything the rubric did not anticipate.

`results/` is untracked (see `.gitignore`), so a run's trees, transcripts, and comparisons stay on the machine that ran it. The 2026-09-04 run on `claude-fable-5-1` is summarised in the owner's `results/README.md`; its conclusions are recorded in `.scratch/rust-skill/spec.md` (Revision 3) and `rules.md` (Revision 3, after the evaluation).

## Why not `claude plugin eval`

The CLI ships `claude plugin eval` with a built-in with/without ablation arm, which is this suite's design. It is early access and gated on this account as of 2026-09-04, and its case format is only partly documented (`.scratch/rust-skill/plugin-eval-notes.md`). The scenarios are already `prompt.md` plus a rubric, so porting them to `case.yaml` files once the command is available is a small change.
