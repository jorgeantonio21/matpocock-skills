# mattpocock-skills

## 1.3.0

### Minor Changes

- [`86c8841`](https://github.com/mattpocock/skills/commit/86c88411c9d40d5eafff8f57127409cac9f3234c) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`ask-jorge`**, this fork's router: `ask-matt`'s map extended with the skills the fork adds on top of upstream (`implement-by-commit`, `implement-by-plan`, `open-pr`). `ask-matt` and its docs page are restored byte-identical to upstream `mattpocock/skills` and stay frozen, so upstream merges never conflict on them; route changes land in `ask-jorge`.

- [`f8178b5`](https://github.com/mattpocock/skills/commit/f8178b587cb5a7480c545661775bbdf35a3b485a) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add the `full-review` skill (in-progress bucket, model-invoked) and the two agents behind it. `full-review` reviews the diff since a fixed point along four axes run in parallel: **Standards** and **Spec** by calling `code-review` unchanged, **Bugs** by the new `bug-hunter` agent, which proves each defect by running it in a throwaway worktree and mutation-probes every test in the diff, and **Craft** by the new `craft-reviewer` agent, which learns the house idiom from sibling files and attaches the idiomatic rewrite to each finding. It writes one verdict file: a verdict of blocked or clean, a **Blocking** list (proven bugs, tests that cannot fail, spec misses, documented-standard breaches), an **Advisory** list (everything else), and the four axis reports verbatim. Axes are never reranked against each other. It accepts a scope (one commit of a plan), a subset of axes for a targeted re-check, and earlier verdict files.

  Agent definitions live in a new root `agents/` folder, linked locally by `scripts/link-agents.sh`.

- [`5aef373`](https://github.com/mattpocock/skills/commit/5aef3739298730c0b4a6d43548467cd66aa43490) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`idiomatic-rust`** (in-progress bucket, model-invoked): a reference skill for concise, idiomatic, readable Rust, written in Simplified Technical English. Six sections in the order an author decides things (Shape, Errors, Ownership, Flow, Surface, Words), each rule an instruction with its reason, with a snippet where the shape is not obvious from prose. The first rule makes the compiler the guardrail: one type per kind of value, so a swapped argument does not compile. Three disclosed references sit beside it: `LINTS.md` carries the clippy check command the agent runs on touched crates (pedantic and restriction lints as command-line flags, no repo change) and the rules it retires from prose; `RUNTIME.md` carries async and thread idiom; `CRATES.md` names a short, optional set of crates that remove hand-written impls, with a short skip list. The `implementer` and `craft-reviewer` agents point at it for Rust diffs.

- [`51c7aee`](https://github.com/mattpocock/skills/commit/51c7aee79e3aa49f3e33ed95e40118e2864b1d16) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`idiomatic-typescript`** to the in-progress bucket as a model-invoked, framework-neutral baseline for honest compiler and runtime guarantees. The core covers TypeScript shape, trust boundaries, functions, mutation, errors, modules, review, and verification, with disclosed invariant, runtime, tooling, and source references. Pinned examples verify runtime behavior, negative diagnostics, and Node and bundler consumers. A seven-scenario evaluation harness measures correctness, restraint, review false positives, churn, and cost without running paid comparisons before a budget is agreed. The `implementer` and `craft-reviewer` agents load the baseline for TypeScript work, and `ask-jorge` maps it alongside the Rust and Pragmatic baselines.

- [`9ba0768`](https://github.com/mattpocock/skills/commit/9ba07680a4521d11c6aef96601ec40eaf3096805) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`implement-by-commit`**, a user-invoked engineering skill that builds a spec or ticket as a planned sequence of human-reviewable commits rather than one. It agrees a commit plan up front and writes it to `.scratch/<feature-slug>/commits.md`, then builds the commits in order, driving `/tdd` at the agreed seams and stopping at a **gate** before each one to present what it built and a suggested message. Feedback folds into the commit under review; `git commit` runs only on approval, and messages carry no trailer lines. `/code-review` runs once at close-out over the whole landed sequence, by which point every commit is in `HEAD` and inside the reviewed diff.

  `ask-jorge` routes to it beside `/implement`, picked on whether a human reads every commit, and it has a docs page at `docs/engineering/implement-by-commit.md`.

- [`f0372a9`](https://github.com/mattpocock/skills/commit/f0372a9fb9cc5219a556caec40f0b52e1c96dc1d) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`implement-by-plan`**, a user-invoked engineering skill that builds a spec or ticket as the same planned sequence of commits as `implement-by-commit`, with the gates removed. The plan is the **contract**: agreed up front (slicing, order, open decisions, target branch) and written to `.scratch/<feature-slug>/commits.md`. Once approved, the run goes unattended to the last commit, driving `/tdd` at the agreed seams and ticking each commit off as it lands; every commit is green on its own, and messages carry no trailer lines. The only interrupt is **drift**: work that proves the plan wrong stops the run to re-agree it. `/code-review` runs once at close-out over the whole landed sequence.

  `ask-jorge` routes to it between `/implement` and `/implement-by-commit`, picked on how much you want to review (nothing, the plan once, or every commit), and it has a docs page at `docs/engineering/implement-by-plan.md`.

- [`ce2be33`](https://github.com/mattpocock/skills/commit/ce2be330116bd722ff6ac5ec44b8ba959dd1f554) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`open-pr`**, a user-invoked engineering skill that ships the commits on the current branch as a pull request. It runs the repo's own checks before anything is pushed, rescues commits stranded on `main` onto a `<type>/ja/<short-description>` branch, and asks which repo and base the PR targets rather than inheriting `gh`'s fork default. The title and body are drafted from the diff alone (plain, factual, no trailer lines) and one **gate** shows the PR exactly as it will appear before it is pushed and created. The run ends at the PR's URL; the merge stays with the reviewer.

  `ask-jorge` routes to it as the step after the build skills, which commit to the current branch and have no PR mode, and it has a docs page at `docs/engineering/open-pr.md`.

- [`8a122fd`](https://github.com/mattpocock/skills/commit/8a122fd6aa1496e53f06acfe2d82013fe76767c9) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add **`pair-by-commit`** and **`pair-by-plan`** (in-progress bucket, user-invoked): the `implement-by-commit` and `implement-by-plan` loops with two agents on the job. An `implementer` agent builds each commit of the agreed plan in the working tree and lands it as a **provisional commit**; `full-review` then checks it on four axes (Standards, Spec, proven Bugs, Craft) and blocking findings go back to the implementer for a fix and amend, at most two rounds. In `pair-by-commit` the user still opens every gate, now with the review beside the code and `git show HEAD` as the thing to read; in `pair-by-plan` a clean verdict is the gate, the run goes unattended to the last commit, and a blocker that survives two rounds stops it as drift. Both close with a whole-sequence review. Advisory findings are never applied unattended. The two original skills are untouched. ADR 0003 records the provisional-commit and fan-out decisions.

- [`1667d60`](https://github.com/mattpocock/skills/commit/1667d60198bb3bdaeec58a328f6dc7283f7096da) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Expose the promoted engineering and productivity skills as a Pi package.

### Patch Changes

- [#926](https://github.com/mattpocock/skills/pull/926) [`84b5ee5`](https://github.com/mattpocock/skills/commit/84b5ee5afd738b6a3484e62509b84b3b573c5be3) Thanks [@mattpocock](https://github.com/mattpocock)! - Add the `implement-spec` skill (in-progress bucket, user-invoked). It takes a spec and its tickets and drives them to a single PR: the tickets are read as a task graph with blocking edges, so implementer subagents run in background worktrees across the ready frontier for concurrency, a merger subagent folds each one back into the PR branch, and the flow closes with `/code-review` before the PR is marked ready.

- [`81e107b`](https://github.com/mattpocock/skills/commit/81e107b413588353996e89e38740a29a11791b8c) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Route the fork's in-progress skills in **`ask-jorge`**. The main flow gains a fourth question, one agent or two, that places `pair-by-commit` and `pair-by-plan` beside the `implement-by-commit` and `implement-by-plan` loops they mirror. `full-review` sits beside `code-review` as the four-axis review the pair skills gate on. A new **Baselines underneath** section maps `pragmatic-programming` and `idiomatic-rust` as the rule references the `implementer`, `bug-hunter`, and `craft-reviewer` agents carry. The precondition names the agents those skills spawn and how to install them. The docs page is re-synced to match.

- [#848](https://github.com/mattpocock/skills/pull/848) [`f02e2ed`](https://github.com/mattpocock/skills/commit/f02e2ed3624d031272f8547742d23bf6bca8b072) Thanks [@mattpocock](https://github.com/mattpocock)! - domain-modeling: trigger on discussing codebase terminology and on writing or editing a CONTEXT.md or an ADR directly, replacing the narrower "pin down domain terminology or a ubiquitous language" / "record an architectural decision" phrasing. Also drops the "another skill needs to maintain the domain model" caveat, since that's the invoking skill's job to state explicitly, not this description's.

- [#911](https://github.com/mattpocock/skills/pull/911) [`4f28947`](https://github.com/mattpocock/skills/commit/4f289474bad013fe2be8f8769d733f59d9103d6b) Thanks [@mattpocock](https://github.com/mattpocock)! - Quote the `description` front matter in `to-spec`, `code-review`, `setup-matt-pocock-skills`, `writing-fragments`, `writing-shape`, and `wait-what`. An unquoted colon-space left over from the em-dash sweep in [#905](https://github.com/mattpocock/skills/issues/905) made each block invalid YAML, so `skills.sh` skipped all six during discovery and they couldn't be listed or installed via `npx skills`.

- [`73624a2`](https://github.com/mattpocock/skills/commit/73624a2d7ea1cb33a8d68a3bc23691ee52723cbf) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Apply the repo's no-em-dash rule to the fork's own skills. `ask-jorge`, `implement-by-commit`, `implement-by-plan`, and `open-pr`, along with their docs pages and changesets, are rewritten so each sentence reaches for the punctuation it actually wants rather than an em-dash. The three changesets that claimed `ask-matt` routes to the fork's skills now name `ask-jorge`, which is the router that does.

- [#917](https://github.com/mattpocock/skills/pull/917) [`85f83d3`](https://github.com/mattpocock/skills/commit/85f83d3fde1d3a90d5c9a657f6998c79a6c37308) Thanks [@mattpocock](https://github.com/mattpocock)! - grilling: update the round template so consecutive questions are separated by a horizontal rule (`---`) instead of running together.

- [#879](https://github.com/mattpocock/skills/pull/879) [`d419977`](https://github.com/mattpocock/skills/commit/d419977fe07d9e1607d3523f3579310bbb076b93) Thanks [@mattpocock](https://github.com/mattpocock)! - grilling: remove em-dashes from `SKILL.md`, replacing them with colons and semicolons so the instructions read as plain text.

- [`3b9db7d`](https://github.com/mattpocock/skills/commit/3b9db7d0c6549b396d0703d6c785e56238d35ad8) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - `idiomatic-rust`: close the gaps from the quality evaluation (fork issue [#6](https://github.com/mattpocock/skills/issues/6)). The preamble states the scope (a library, a service, or an engine) and each house rule now says where it does not apply: rejection-as-outcome, `#[must_use]` on an outcome, `Copy` errors, wire types, `new_const`, the `test_` prefix, and the hot-path regime. `RUNTIME.md` decides async in a trait by dispatch (a sync seam in an engine, a native `async fn` trait behind `trait-variant` and `dynosaur` in a service with a pluggable backend), adds a `Send`-at-the-spawn-boundary entry (which types are `!Send`, which are only `!Sync`, and a compile-time assertion), and says a work queue at the edge is an `mpsc`. New idioms: let chains (edition 2024), `impl Iterator` and `Cow<'_, str>` in return position ("Return by need"), and `#[non_exhaustive]`. Inconsistencies fixed: `Default` off the `JobId` derive stack, tests return `anyhow::Result<()>` everywhere, the sentence cap follows ASD-STE100 (20 for an instruction, 25 for a description), `-A unreachable_pub` for a binary crate, and a bounded newtype returns `Result` with the rejected value at a config or request boundary. `expect_used` leaves the check command and the workspace block, because `expect` with the invariant as its message is the form the skill asks for; `unwrap_used` stays.

  The skill now ships an evaluation suite under `evals/`: five scenarios (write a crate, refactor a crate with thirteen planted drift patterns, extend a crate, write a tokio worker pool, review a crate), each with a prompt, a starting crate pinned to Rust 1.97.1, and a rubric naming the skill entries it exercises. `run.sh` runs a scenario headless in a bare arm and a skill arm on the same model; `score.sh` runs the tests, the LINTS.md check (with the flags read from `LINTS.md`), and pattern counts on both trees. The first run (Fable 5.1) showed the skill's effect on the type-level rules and none on the mechanical ones. The bare arm fixed 12 of the 13 planted patterns, so the fixture probe's threshold (the bare arm misses at least three) was not met. Four sentences the run showed to be no-ops or misfires were cut, and `evals/README.md` records the run.

- [`fcaa223`](https://github.com/mattpocock/skills/commit/fcaa223a2e20d5a756831d13cd97fb2b988a65f3) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - `idiomatic-rust`: teach general invariant patterns and contextual Rust idioms (fork issue [#8](https://github.com/mattpocock/skills/issues/8)). A new `INVARIANTS.md` names three kinds of guarantee (intrinsic, aggregate, contextual admission) and eight patterns, each with the guarantee, what it prevents, when it applies, and what is enough instead. Every Rust block in it, and in `SKILL.md`, `RUNTIME.md`, and `CRATES.md`, is a verbatim excerpt of a compiled, tested, lint-clean `examples/` crate, and `evals/check.sh` fails when they drift. Two snippets changed to pass the skill's own check: `select!` arms bind `()` rather than `_`, and the drain closure ends its call with a semicolon. `SKILL.md` opens the Shape section with "State the guarantee before you pick the type" (a wrapper for an invariant, a confusable role, or behavior; the primitive otherwise), adds "Close every route" (a private field stops nothing that a derived `Deserialize` or a delegating `FromStr` builds), fixes the `const fn new` example (`then_some` is not const-callable on stable Rust; an `if` is), makes typestate and phantom types a response to a named bug, adds the outcome-carries-the-facts and shared-interpretation rules and an optional module-local `Result` alias, and says which rules are requirements, defaults, or conventions. `RUNTIME.md` decides a hot path by requirement and measurement rather than by program category, keeps the hot-path rule absolute (no lock, and no structure that locks or parks inside, whatever the alternative costs), lists the alternatives in order, and leaves the choice of mutex for the edge to the project. It also corrects the dynosaur claim (the trait object still boxes each future) and the `Send` advice (assert the future, not only the type). `CRATES.md` replaces the "Do not reach for" list with selection guidance by capability for error, builder, validation, and async-trait crates, keeps the three std replacements, states the proc-macro cost precisely (the emitted code, read with `cargo expand`), and corrects the `EnumIter` cost (a stack struct, no heap allocation).

  The evaluation suite gains four scenarios, each with external semantic tests and a reference solution: a decoder library (two routes that bypass `new`, a negation at `i32::MIN`), an inference request scheduler (a policy change that must keep admitted work, a retirement whose outcome is the record), a small CLI (a stricter config version that must keep a version 1 file's meaning), and a plain crate with one defect where the right diff is tiny. `run.sh` accumulates runs (`r1`, `r2`, ...) and takes a `skill@<git-ref>` arm, so the merged skill and a revision are compared on one model, and every run records the skill commit it loaded. `score.sh` runs the external tests, reports the diff against `start/`, applies a scenario's `check-flags` to both passes, and reports `BUILD FAILED` and `INCOMPLETE` instead of zero findings when cargo fails. `analyze.py matrix` aggregates runs into a result table. `test_score.py` drives `score.sh` with a stub `cargo` and pins every verdict, and `s4-async` gets the `print_stdout` relaxation in both passes through `check-flags`. Two fixtures were cleaned (`s1`'s empty file, two unplanned lint findings in `s3`'s tests). The 2026-09-04 results are preserved. The revision has not been run against a model, and `evals/README.md` lists what promotion still needs.

- [`01de312`](https://github.com/mattpocock/skills/commit/01de3125df2baccd0478f9736d7c9d5658165d29) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - `idiomatic-rust`: trim the guidance files without dropping a rule. Every section name, rule heading, and Rust block stays (the rubrics cite the headings and `evals/check.sh` pins the blocks to `examples/`), and every atomic instruction was inventoried before the rewrite and mapped back after it. What changed is the prose: "Do not X. Do Y." pairs become the positive form, each rule keeps one example, a reason stays only where it carries a non-obvious fact, and each meaning lives in one file with a clause-length pointer from the others (the hot-path lock rule in `RUNTIME.md`, the async-trait boxing cost in `CRATES.md`, the three kinds of guarantee in `INVARIANTS.md`). Two redundant code blocks leave `INVARIANTS.md` (the `FromStr` impl for `Concurrency`, the `remove_and_announce` consumer). The lint calibration diary moves from `LINTS.md` to `evals/README.md` under "Lint calibration", and `LINTS.md` keeps the name check to run after a toolchain bump. The flags block, the retire table, and the workspace TOML are byte-identical.

- [`ba108db`](https://github.com/mattpocock/skills/commit/ba108dbfc8e4c40b2da8aa9069085dc6d6f4be9e) Thanks [@jorgeantonio21](https://github.com/jorgeantonio21)! - Add the `pragmatic-programming` skill (in-progress bucket, model-invoked). A reference of tips from _The Pragmatic Programmer_ cut down to the ones a diff can be judged against, each reading what it looks like → the move, in four sections: **Bugs** (prove it, contracts, crash early, assertions, resources, shared state, coincidence), **Craft** (DRY, orthogonality, tell don't ask, method chains, globals, inheritance tax, design to test, naming), **Design** (easier to change, fortune-telling, decoupling, data over state, configuration, simplicity), and **Building** (tracer bullets, small steps, quality as a requirement). Entries that overlap a Fowler smell from `code-review`'s Standards baseline are bracketed so a review running both reports the smell once. Foundation for the `full-review`, `pair-by-commit`, and `pair-by-plan` skills that follow.

- [#905](https://github.com/mattpocock/skills/pull/905) [`e6e9577`](https://github.com/mattpocock/skills/commit/e6e957797d8cceb5b351c0dc840369523f9fb8fb) Thanks [@mattpocock](https://github.com/mattpocock)! - Remove every em-dash from the repo's prose (docs, `SKILL.md` files, ADRs, `README.md`, scripts, JSON/YAML metadata), hand-rewriting each sentence with a comma, colon, period, parentheses, or conjunction rather than mechanically substituting the character. `CLAUDE.md`/`AGENTS.md` now says not to reintroduce them.

- [#878](https://github.com/mattpocock/skills/pull/878) [`e3e547b`](https://github.com/mattpocock/skills/commit/e3e547b57d549110a0aa6ff40fd7b871c01c76c9) Thanks [@mattpocock](https://github.com/mattpocock)! - Standardize cross-skill invocation on an explicit "call the Skill tool" instruction instead of bare `/skill`-style prose, across `code-review`, `diagnosing-bugs`, `grill-with-docs`, `grill-me`, `improve-codebase-architecture`, `tdd`, `to-spec`, `to-tickets`, `triage`, and `wayfinder`.

  - A skill that names another skill in prose ("run the `/grilling` skill") does not reliably cause it to load. This is the documented rough edge behind `grill-with-docs`'s most-reported problem. Naming the tool directly (`Call the Skill tool with "grilling"`) is intended to raise the hit rate. Dropping the leading `/` also makes the instruction harness-neutral rather than less: it no longer assumes Claude Code's trigger syntax.
  - A step needing more than one skill now says so as multiple calls ("Call the Skill tool twice, for `grilling` and `domain-modeling`"), not one call carrying two names.
  - Documents the convention in `.agents/invocation.md` for future skills to follow.

- [#880](https://github.com/mattpocock/skills/pull/880) [`1dab982`](https://github.com/mattpocock/skills/commit/1dab98299c3b81f560026c01b7ebf55ed5d91373) Thanks [@mattpocock](https://github.com/mattpocock)! - Stop skills from trying to reach user-invoked skills through the Skill tool: fix cross-skill references that violated the "no other skill can call it" invariant in `.agents/invocation.md`, in `to-spec`, `wayfinder`, `to-tickets`, `triage`, `code-review`, and `diagnosing-bugs`.

  - `to-spec`, `wayfinder`, `to-tickets`, `triage`, and `code-review` each carried a precondition ("...run `/setup-matt-pocock-skills` if not") that PR [#878](https://github.com/mattpocock/skills/issues/878) rewrote into a literal `Call the Skill tool with "setup-matt-pocock-skills"` instruction. `setup-matt-pocock-skills` is user-invoked, so none of these skills (user-invoked or model-invoked) can call it. Reworded all five as instructions for the agent to tell the human to run it instead.
  - `diagnosing-bugs`'s Phase 6 post-mortem hand off to `improve-codebase-architecture` (also user-invoked) the same way, from an autonomous, often-unattended bug-fixing flow with no human in the loop to catch the failed call. Removed the hand-off outright rather than softening it, since it rarely fired in practice. Phase 6 is now "Cleanup" only; the mechanical checklist is untouched.
  - Added a carve-out paragraph to `.agents/invocation.md`'s "Dependencies between them" section: the `Call the Skill tool with "name"` convention only applies when the named skill is model-invoked. This is the section PR [#878](https://github.com/mattpocock/skills/issues/878) introduced without reconciling it against the user-invoked/model-invoked invariant stated eight lines above it; the gap is most of why this bug reached six call sites instead of one.

  Fixes [#453](https://github.com/mattpocock/skills/issues/453).

- [#904](https://github.com/mattpocock/skills/pull/904) [`594f0f8`](https://github.com/mattpocock/skills/commit/594f0f83188921a60d45d63d6cdac509de20df2c) Thanks [@mattpocock](https://github.com/mattpocock)! - wait-what: follow `CONTEXT-MAP.md` to the right `CONTEXT.md` when a repo indexes multiple contexts that way instead of keeping a single root `CONTEXT.md`.

## 1.2.3

### Patch Changes

- [#779](https://github.com/mattpocock/skills/pull/779) [`efce423`](https://github.com/mattpocock/skills/commit/efce423018fc6468a3239621f1c1bcaacc723801) Thanks [@mattpocock](https://github.com/mattpocock)! - Make `diagnosing-bugs` redact secrets.

  - Add a **Redact** section to `SKILL.md`. The skill has the agent show commands, outputs and captured artifacts; the section makes redaction the first move on each — write `<REDACTED>`, build loops against env vars so the credential stays in the environment, and quote only the signal-carrying lines of a captured artifact.
  - The Phase 1 completion criterion said "paste the invocation and its output". It now says show it redacted, and Phase 1 asks the user for a **redacted** captured artifact.
  - Note in `scripts/hitl-loop.template.sh` that `capture` prints its value back to the terminal, so it takes observations while signing in stays a `step`.

- [#781](https://github.com/mattpocock/skills/pull/781) [`14bfbbd`](https://github.com/mattpocock/skills/commit/14bfbbd8654a8d2910299e1a004c19c1979687d8) Thanks [@mattpocock](https://github.com/mattpocock)! - Drop Claude Code's tool and agent-type names from the subagent-dispatch instructions in `code-review`, `codebase-design`, and `improve-codebase-architecture`, so the step is followable on Codex and other harnesses.

- [#783](https://github.com/mattpocock/skills/pull/783) [`c0fd1e9`](https://github.com/mattpocock/skills/commit/c0fd1e973e040347d424e09934099f1bd6c2dee0) Thanks [@mattpocock](https://github.com/mattpocock)! - wizard: remove the time estimate. The template drops `TOTAL_MINUTES` and the time-remaining display, `stage` takes a name only, and progress is counted in stages.

## 1.2.2

### Patch Changes

- [#766](https://github.com/mattpocock/skills/pull/766) [`4aaccb5`](https://github.com/mattpocock/skills/commit/4aaccb58d40559d7e3c59a029b2290ae5ba538de) Thanks [@mattpocock](https://github.com/mattpocock)! - Make `writing-for-agents` model-invokable in Codex again.

  - Drop `policy.allow_implicit_invocation: false` from `agents/openai.yaml`. Codex filtered the skill out of the model-visible skills list, so its description could not trigger it — only an explicit `$writing-for-agents` mention worked.
  - Update the stale `interface.display_name` and `interface.short_description`, which still named the old `writing-great-skills` skill.
  - Move the skill from the **User-invoked** list to the **Model-invoked** list in `README.md` and `skills/productivity/README.md`.

## 1.2.0

### Minor Changes

- [#551](https://github.com/mattpocock/skills/pull/551) [`697d4ce`](https://github.com/mattpocock/skills/commit/697d4ce9742da558fd1ba6697c8e9775e2e302dd) Thanks [@mattpocock](https://github.com/mattpocock)! - Add Codex metadata alongside each skill's Claude Code frontmatter so the set works in both harnesses without generated copies.

  - Add an `agents/openai.yaml` beside every `SKILL.md` with Codex UI metadata (`interface.display_name`, `interface.short_description`).
  - Mark every user-invoked skill with `policy.allow_implicit_invocation: false`, the Codex analog of `disable-model-invocation: true`, so Codex excludes it from implicit invocation while explicit `$skill` invocation still works.
  - Document the dual-harness invocation model in `.agents/invocation.md`, `CLAUDE.md`, and the promoted-bucket READMEs.
  - Add `AGENTS.md` as a symlink to `CLAUDE.md` so Codex reads the same repo instructions.

- [#593](https://github.com/mattpocock/skills/pull/593) [`0f2bdbd`](https://github.com/mattpocock/skills/commit/0f2bdbdb06220d2df3718b8f0483157c6c8a8600) Thanks [@mattpocock](https://github.com/mattpocock)! - Graduate **`to-questionnaire`** out of `in-progress/` into the **Productivity** bucket, so it ships in the plugin. It turns a decision you can't answer alone into a Markdown questionnaire for the one person who can — filled in async, or worked through together in a meeting.

  Its defining move is that it grills you about the **send**, not the subject: a normal grilling session interrogates the topic, which is exactly what you can't answer here, so the interview asks only who the questionnaire is going to and what you need back, then aims every question at the gap between the two.

  Now wired as a promoted skill — plugin entry, top-level + Productivity READMEs under **User-invoked**, a docs page at `docs/productivity/to-questionnaire.md`, and a Standalone route in `ask-matt` framing it as the inverse of `/grill-me` (mine someone else, not yourself).

- [#680](https://github.com/mattpocock/skills/pull/680) [`b3376f8`](https://github.com/mattpocock/skills/commit/b3376f8d39848dd08572ec2667da4739a67c8c04) Thanks [@mattpocock](https://github.com/mattpocock)! - Graduate **`wizard`** out of `in-progress/` into the **Engineering** bucket, so it ships in the plugin — and make it model-invoked. It generates an interactive bash script that walks a human through a manual procedure — third-party setup, a one-off migration, an A→B state transition — opening each URL, saying what to click, capturing the values, and writing them into `.env` files and GitHub Actions secrets.

  The delightful UX is pre-solved by the bundled `template.sh` (progress with time-remaining, confirmation gates, cross-platform URL opening including WSL, hidden secret entry, idempotent `.env` upserts, `gh secret`/`gh variable` writes with graceful degradation, closing skip summary). Everything above the `STAGES` marker is a fixed library that's never hand-edited — the skill's job is only to scope the procedure and author its **stages**.

  Engineering rather than Productivity: it reads `.env*`, `docker-compose*`, framework config and every `secrets.*`/`vars.*` reference in `.github/workflows/` to scope itself, writes CI secrets, and verifies its output with `bash -n` and `shellcheck`.

  Because it is model-invoked, the agent can reach for it the moment it hits a step only a human can perform, instead of dumping numbered instructions into the chat and hoping you follow them. Typing `/wizard` works exactly as before — model-invocation only ever _adds_ the agent's reach. The description is written as the pointer that decides when it fires: what it produces, four trigger branches (provisioning infrastructure, setting up credentials or CI secrets, walking an unfamiliar third-party dashboard, a one-off migration or cutover), and an explicit non-trigger — don't invoke it for steps the agent can perform itself. Work an agent can do, an agent should do; the wizard is for the clicks, approvals and dashboard trips you would not hand to one. The stage-list confirmation before a line is written now doubles as the proposal when the agent fires it mid-build.

  Now wired as a promoted skill — plugin entry, top-level + Engineering READMEs under **Model-invoked**, a docs page at `docs/engineering/wizard.md`, and a Standalone route in `ask-matt` for the steps only a human can take. Model-invocation also puts it out of the reach of [#693](https://github.com/mattpocock/skills/issues/693), which drops user-invoked skills from the listing on Claude's desktop and web surfaces.

- [#763](https://github.com/mattpocock/skills/pull/763) [`77d207e`](https://github.com/mattpocock/skills/commit/77d207ef03219cc603e2832e1159cbdd1c91818e) Thanks [@mattpocock](https://github.com/mattpocock)! - Reshape the **`prototype`** skill around two ideas: the demo is **a single shareable HTML file**, and the prototype is **a primary source**.

  The logic branch now produces one self-contained file (plain HTML/CSS/JS, no build, no server) instead of a terminal app — a non-developer can open it by double-click and drive it in their own domain language: a labelled state panel, always-available free-play buttons, and a set of tabbed **guided walkthroughs**, each a scenario with the ordered buttons to press underneath it. The portable pure-logic module still lifts into the real code; the HTML shell is the throwaway.

  Throwaway no longer means deleted. Rather than being removed once it has answered its question, the prototype is captured as runnable evidence on a throwaway branch (`prototype/<name>`) out of main, with a context pointer to it left on the implementation issue — so the main branch keeps only the validated decision while the exploration stays findable. The answer (verdict + question) is still captured durably in an issue/ADR/commit.

- [#536](https://github.com/mattpocock/skills/pull/536) [`42a5b70`](https://github.com/mattpocock/skills/commit/42a5b70fcacc7baff1977b13f3919fb2f63af14e) Thanks [@mattpocock](https://github.com/mattpocock)! - Ship the skill set as a native **Claude Code plugin**, listed in Claude Code's official marketplace. You can now subscribe to the promoted skills as a managed, read-only bundle instead of copying editable files:

  ```bash
  claude plugins install mattpocock-skills
  ```

  Or, from inside a session:

  ```
  /plugin install mattpocock-skills
  ```

  There is no marketplace to add first — the official marketplace is configured by default.

  `.claude-plugin/plugin.json` carries the full plugin metadata (version, description, author, license, keywords) and the explicit list of promoted skills. `skills.sh` remains the universal installer (and the path for Codex and other harnesses today); a native Codex plugin is deferred — see `.agents/adr/0002-ship-as-a-claude-code-plugin.md` for why.

- [#751](https://github.com/mattpocock/skills/pull/751) [`355fa74`](https://github.com/mattpocock/skills/commit/355fa7420b418af838998f7ec4365ceda1c8dfcc) Thanks [@mattpocock](https://github.com/mattpocock)! - Add **`wait-what`** — a one-word corrective for model verbosity. Type it the moment a message doesn't land, and the agent re-pitches it: a little context, ASD-STE100 Simplified Technical English, and the ubiquitous language from your `CONTEXT.md`. User-invoked, three lines long.

  The mechanism is the name. Concision skills fail by growing — a 400-line skill still leaves the model verbose — so this one is a single precise leading word and nothing else. Names that describe the _output_ (`/tldr`, `/no-fluff`) make the model clip words and lose you further; naming the _listener's_ state asks for both halves at once, fewer words **and** the context you were missing. It also reuses the leading words already in your global `CLAUDE.md`, so the skill, `CLAUDE.md` and every `CONTEXT.md` reach for the same tokens.

  It repairs one message; it doesn't prevent the next one. The cure for jargon is a shared language built upfront with `/grill-with-docs`; this is what you reach for when you don't have one yet.

- [#763](https://github.com/mattpocock/skills/pull/763) [`77d207e`](https://github.com/mattpocock/skills/commit/77d207ef03219cc603e2832e1159cbdd1c91818e) Thanks [@mattpocock](https://github.com/mattpocock)! - Name the `/wayfinder` unit a **decision ticket**, and burn research tickets down with subagents.

  People kept reading a wayfinder ticket as an ordinary _implementation_ ticket — a slice of a build to execute — when wayfinder uses them as **decision tickets**: questions whose resolution is a decision. The skill description and its opening line now introduce the term (and say what makes it one), with the `ask-matt` / engineering README blurbs and the docs page matching — while "ticket" stays the everyday word once the term is established. `CONTEXT.md` records **Decision ticket** as a domain term, so the "avoid: ticket" guidance no longer contradicts wayfinder's deliberate use of the word.

  Research tickets are no longer parked for a separately-launched session. Research stays a real ticket type — it's a genuine shared blocker that downstream decisions hang on, and that dependency is exactly what the frontier's blocking edges exist to render. What changes is how it's resolved: because research is AFK, charting doesn't stop and read it. After creating the tickets, the charting session fires a `/research` subagent for each research ticket to burn it down in parallel, capturing the findings on a throwaway `research/<name>` branch with a context pointer. Research tickets are the one exception to _one ticket per session_.

- [#763](https://github.com/mattpocock/skills/pull/763) [`77d207e`](https://github.com/mattpocock/skills/commit/77d207ef03219cc603e2832e1159cbdd1c91818e) Thanks [@mattpocock](https://github.com/mattpocock)! - **Breaking:** rename **`writing-great-skills`** → **`writing-for-agents`**, restructure it, and add a new leading word.

  The reference now covers any document an agent consumes — skills, `AGENTS.md` / `CLAUDE.md`, docs reached by a pointer — not just skills. `GLOSSARY.md` is merged into `SKILL.md` (one authoritative treatment per term; the `_Avoid_` synonym lists and the standalone Predictability definition are gone); the skill-only mechanics (frontmatter, model- vs user-invoked, router skills, the invocation cut of splitting) are disclosed to a new `SKILL-MECHANICS.md`. The skill is now **model-invoked**: it fires when creating or editing skills or modifying `AGENTS.md`/`CLAUDE.md`. `ask-matt`'s pointer updated. Reinstall under the new name; the old name is gone (no alias).

  The pruning section gains **cache**. Single source of truth now reaches past the document into the environment — `package.json` scripts, config files, directory layout, `--help` output are themselves authoritative, so a doc that restates them is a cache of a lookup, earning its load only when the lookup is expensive. The positive target: cache what the agent cannot find by looking (unwritten conventions, the reason behind a choice, gotchas no config confesses), and leave one-file, one-command lookups to the environment, where they cannot go stale.

- [#533](https://github.com/mattpocock/skills/pull/533) [`45afd80`](https://github.com/mattpocock/skills/commit/45afd8074a8b7de5fe073845d080fa9dd6c429fa) Thanks [@mattpocock](https://github.com/mattpocock)! - Add a YAGNI scoping filter to the **`improve-codebase-architecture`** skill's Explore step. Instead of scanning the whole repo evenly, it now scopes to where change is actually landing: if you name a direction it takes it, otherwise it reads the last ~20 commit messages to bias exploration toward actively-developed paths. A deepening opportunity in code nobody touches is a refactor you'll never cash in — the leverage only pays off where you keep editing — so the report stops tidying dormant corners of the repo.

### Patch Changes

- [#763](https://github.com/mattpocock/skills/pull/763) [`77d207e`](https://github.com/mattpocock/skills/commit/77d207ef03219cc603e2832e1159cbdd1c91818e) Thanks [@mattpocock](https://github.com/mattpocock)! - Sharpen `/ask-matt` — the router now covers phase boundaries, the two wayfinder mistakes, and two skills it never mentioned.

  **Phase boundaries.** A **phase** is a chunk of work inside a session — the grilling, the implementation, the QA — and the boundary between two of them is where you decide what to do with the context you've built. The two-bullet `Crossing sessions` section is replaced by a decision tree carrying all five options in order (**continue**, `/clear`, `/handoff`, **subagent**, `/compact`), with the reasoning disclosed in a new `PHASE-BOUNDARIES.md`. Three fixes come with it:

  - **`/handoff` was oversold.** It read as the general bridge between context windows. It's narrow: you need it only when something has to _travel_ — a new harness, a new directory, a colleague, or a side task forked mid-phase. What it buys is portability.
  - **`/compact` is the default, not the first reach.** It sits at the bottom of the tree, after the four cheaper or more precise questions above it. Starting there produces a session that's confidently wrong about whatever the summary flattened.
  - **Two branches were missing entirely.** **Continue** is the one to rule out first — it's the only move that keeps the conversation as a primary source rather than a summary of one — and a **subagent** handles anything scoped tightly enough to run AFK.

  Context hygiene's escape hatch now says `/compact` rather than `/handoff` (same harness, same directory, at a boundary — the handoff clause doesn't apply), and the smart zone figure is updated from ~120k to ~150k tokens.

  **Wayfinder routing.** The two mistakes people most often make with the heaviest, most cognitively demanding flow:

  - **Over-reaching for it.** It's slower and denser than a single grill, so it's flagged as the heaviest flow and reserved for the idea that genuinely won't fit one session — a well-scoped feature belongs on `/grill-with-docs`, not here.
  - **Losing the way at the handoff.** When the map clears, wayfinder hands off, it doesn't build: merge onto the main flow at `/to-spec` (which collapses the map's linked decisions into a buildable plan) rather than looping the map straight into `/implement`. Straight-to-`/implement` is only for efforts that turned out genuinely small.

  **Missing routes.** `/grilling` and `/resolving-merge-conflicts` were absent from the router altogether and are now in it, and `grill-me` splits from `grill-with-docs` on whether you are in a working directory.

- [#502](https://github.com/mattpocock/skills/pull/502) [`44eed54`](https://github.com/mattpocock/skills/commit/44eed545186ffd0263e8004867750b80cfddd215) Thanks [@mattpocock](https://github.com/mattpocock)! - Make `/setup-matt-pocock-skills` friendlier and align the local-markdown tracker with the current spec.

  - **Triage labels** are now asked about only when the `triage` skill is installed, and then as a single recommended-yes question ("keep the default triage labels?") instead of an override interrogation. When `triage` isn't installed, the section — and `docs/agents/triage-labels.md` — are skipped.
  - **External PRs as a request surface** is no longer a setup question. The GitHub/GitLab templates still carry the flag, defaulted off; a user can flip it in `docs/agents/issue-tracker.md` later.
  - **Domain docs** default to single-context without asking; multi-context is only offered when the repo shows monorepo signals.
  - **Local-markdown tickets** are now one file per ticket under `.scratch/<feature>/issues/<NN>-<slug>.md` — never a single combined `tickets.md`. `/to-tickets` and the local issue-tracker template now agree, and the spec file is `spec.md` (not `PRD.md`) to match `/to-spec`.

  Docs pages for `setup-matt-pocock-skills` and `to-tickets` re-synced.

- [#532](https://github.com/mattpocock/skills/pull/532) [`170ad48`](https://github.com/mattpocock/skills/commit/170ad48655825783d0193e850e31a9aac957bb95) Thanks [@mattpocock](https://github.com/mattpocock)! - Reword **`grilling`** for general use. Its description and body no longer scope the interview to a software plan: "this plan" → "this", "enact the plan" → "act on it", and "exploring the codebase" → "exploring the environment". The technique is unchanged; it now reads as a stress-test of any plan, decision, or idea.

- [#593](https://github.com/mattpocock/skills/pull/593) [`a4b2009`](https://github.com/mattpocock/skills/commit/a4b2009a1a3ac9575506c10b4c84f08f9bba7a38) Thanks [@mattpocock](https://github.com/mattpocock)! - Rework **`grilling`** from one-question-at-a-time to round-by-round. It now maps the decision tree and asks the whole **frontier** — every question whose prerequisites are already settled — in a single numbered round, then recomputes the frontier from the user's answers and asks the next round. Same 13 questions land in ~3 rounds instead of 13. Facts the environment can answer are dispatched to background sub-agents so research never blocks the round: only questions downstream of a running exploration wait for it. The session ends when the frontier is empty.

  Every question in a round is emitted in one fixed shape — `❓ **Q1** - **<title>**`, then the body (prose or multiple choices), then the recommendation on its own `➡️` line. A round reads as a scannable numbered list with each recommendation visually separated from the question, so you can answer by number instead of quoting questions back.

  `grill-me`, `grill-with-docs` and `triage` run the frontier a round at a time as well — `triage`'s grill step and `grilling`'s Codex `short_description` now say so instead of describing the old rhythm. The opt-out for one-question-at-a-time (a line in your global `CLAUDE.md`) is unchanged.

- [#752](https://github.com/mattpocock/skills/pull/752) [`c66bdee`](https://github.com/mattpocock/skills/commit/c66bdeeee002d81e3f8b21403c07f9a0d7bea6da) Thanks [@mattpocock](https://github.com/mattpocock)! - Remove six skills from the repo. None of them was in the Claude Code plugin, but all six were installable through [skills.sh](https://skills.sh/mattpocock/skills), which serves every skill in the repo — so this is what leaves that listing, and where each one went.

  Four retired skills, each already absorbed by a skill that does the job better:

  - **`ubiquitous-language`** → **`/domain-modeling`**, which builds and maintains the whole domain model rather than dumping a glossary from one conversation.
  - **`design-an-interface`** → **`/codebase-design`**. Nothing is lost: the "design it twice" technique — parallel sub-agents generating radically different designs, from Ousterhout — ships inside that skill as `DESIGN-IT-TWICE.md`.
  - **`qa`** → **`/triage`** and **`/to-tickets`**.
  - **`request-refactor-plan`** → **`/to-spec`** and **`/improve-codebase-architecture`**.

  And two that were only ever mine — tied to my own machine and never meant for anyone else. The `personal/` bucket goes with them:

  - **`edit-article`**
  - **`obsidian-vault`**, which hardcoded a path to my own Obsidian vault.

  `skills/deprecated/` stays as a bucket, now empty. `skills/in-progress/` is unchanged and is now described for what it actually is: a beta channel, published on purpose, installable one skill at a time through skills.sh.

- [#734](https://github.com/mattpocock/skills/pull/734) [`a2f9333`](https://github.com/mattpocock/skills/commit/a2f9333669ff53db762c87ecda5a15442060a3be) Thanks [@mattpocock](https://github.com/mattpocock)! - Finish the `to-prd` → `to-spec` rename: "spec" is now the only term in the shipped text.

  - **`to-spec`** no longer opens with "you may know this document as a PRD" — the parenthetical is dropped from the skill and its docs page. The local-markdown tracker template drops the same hedge.
  - **`code-review`** talks about the originating issue/spec rather than issue/PRD, in its frontmatter description, its two-axis summary, and the spec-source search order. Both READMEs re-synced.
  - **The GitHub and GitLab tracker templates** now say "Issues and specs for this repo live as GitHub/GitLab issues" — they had been left on "PRDs" when the local template was updated, so the stale term propagated into every repo they were written into.
  - **`docs/engineering/research.md`** pointed at `https://aihero.dev/skills-to-prd`, a dead slug for the renamed skill; it now links `to-spec` like the other nineteen docs pages do.

  The CHANGELOG and existing changesets still name PRDs where they document the rename itself, which is correct.

## 1.1.0

### Minor Changes

- [#406](https://github.com/mattpocock/skills/pull/406) [`930a450`](https://github.com/mattpocock/skills/commit/930a450089f77a49af09001d955db8452a4b867d) Thanks [@mattpocock](https://github.com/mattpocock)! - Bring the **`ask-matt`** router up to date with the full skill set. It now maps five skills it was missing: **`tdd`** (woven into the main flow as the red-green engine `implement` drives), **`diagnosing-bugs`** (a new "Something's broken" on-ramp — there was previously no route for a bug), **`domain-modeling`** and **`codebase-design`** (a new "Vocabulary underneath" section), and **`grilling`** (the shared interview primitive). `prototype` is fleshed out as a standalone and the description broadens from "user-invoked skills" to "the skills". A maintenance rule is added to `CLAUDE.md` so any future skill add/rename/remove or flow change triggers an `ask-matt` re-check, beside the existing docs-page re-sync rule.

- [#464](https://github.com/mattpocock/skills/pull/464) [`639df6e`](https://github.com/mattpocock/skills/commit/639df6e7386dfddc739b2aecdeff37a876f2483b) Thanks [@mattpocock](https://github.com/mattpocock)! - Promote and harden **`code-review`**. The in-progress **`review`** skill is renamed to **`code-review`** and moved from `in-progress/` into `engineering/`: it now ships in the plugin, is listed in the top-level and Engineering READMEs (Model-invoked), and has a docs page at `docs/engineering/code-review.md`. The `/implement` skill and docs point at `/code-review`.

  It also gains an always-on **Fowler smell baseline** on its Standards axis — a curated ~12 high-signal "Bad Smells in Code" (Mysterious Name, Duplicated Code, Feature Envy, Data Clumps, Primitive Obsession, Repeated Switches, Shotgun Surgery, Divergent Change, Speculative Generality, Message Chains, Middle Man, Refused Bequest) inlined into `SKILL.md` as a fixed baseline alongside whatever the repo documents, not a new third axis. Two binding rules keep it safe: a documented repo standard overrides the baseline, and every smell is reported as a judgement call, never a hard violation.

- [#464](https://github.com/mattpocock/skills/pull/464) [`639df6e`](https://github.com/mattpocock/skills/commit/639df6e7386dfddc739b2aecdeff37a876f2483b) Thanks [@mattpocock](https://github.com/mattpocock)! - Sharpen **`grilling`** on two fronts.

  **A confirmation gate.** The agent won't enact the plan until you confirm the shared understanding has been reached — turning the skill's existing "shared understanding" completion criterion into an explicit stop-gate. The `description` also recruits the pretrained **`grill`** leading word ("Grill the user relentlessly") to sharpen invocation, and the docs page is re-synced.

  **Facts vs. decisions.** Grilling now splits _facts_ (look them up — explore the codebase) from _decisions_ (put each one to the human and wait for their answer). The old blanket line — "if a question can be answered by exploring the codebase, explore the codebase instead" — was written for the live-human case, but once another skill runs grilling inside a resolve-the-ticket frame it read as license to answer _decisions_ autonomously too. Separating the two keeps a grilling agent from racing ahead and answering its own questions.

- [#463](https://github.com/mattpocock/skills/pull/463) [`af6d692`](https://github.com/mattpocock/skills/commit/af6d6922c3e2b5288eef155346cbe319e4ed3bd0) Thanks [@mattpocock](https://github.com/mattpocock)! - Add two adjacent Steering failure modes to **`writing-great-skills`**, both about how language you think of as "off" still steers the agent. **Negation** — the _elephant_ — is steering by prohibition: naming what _not_ to do drags the forbidden behaviour into context and makes it _more_ available, not less (_don't think of an elephant_), so the cure is to prompt the **positive**. **Negative Space** — the void — is blindness to the steering done by what you leave _out_: every decision a skill declines is delegated to the agent's priors rather than left neutral, so the cure is to read a draft for its silences and decide each omission deliberately (fill it, or leave it open as a real **branch**). Kept as two entries, not one — they carry different diagnostics and different cures — each a full `GLOSSARY.md` entry plus a `SKILL.md` failure-mode bullet, matching how every other failure mode is carried.

- [`850873c`](https://github.com/mattpocock/skills/commit/850873cd73d5f81826ebf512ad35d2b1e113001f) Thanks [@mattpocock](https://github.com/mattpocock)! - Make the **`prototype`** skill model-invoked, so the agent can reach for it autonomously (and other skills can too). Its description is rewritten around the leading word _prototype_ — throwaway code that answers a design question — with one trigger per branch (state/logic sanity-check, or UI exploration).

- [#409](https://github.com/mattpocock/skills/pull/409) [`0d74d01`](https://github.com/mattpocock/skills/commit/0d74d01cbc64ca27778a49b38599f70c534e76a0) Thanks [@mattpocock](https://github.com/mattpocock)! - Add the **`research`** skill — a small, model-invoked skill that spins up a **background agent** to investigate a question against **primary sources** (official docs, source code, specs, first-party APIs), then leaves a single cited Markdown file wherever the repo keeps such notes. It's delegable reading legwork: you keep working while it reads, and get back a document to grill, plan, or design against. Listed in the top-level and Engineering READMEs (Model-invoked), added to `.claude-plugin/plugin.json`, given a docs page at `docs/engineering/research.md`, and routed as a Standalone in `ask-matt`.

- [#469](https://github.com/mattpocock/skills/pull/469) [`a0329ba`](https://github.com/mattpocock/skills/commit/a0329ba95751f58566ed7ab484475917a68f1629) Thanks [@mattpocock](https://github.com/mattpocock)! - Split the **`to-issues`** skill into a lean **Process** and a **Reference** section, and teach it to handle a **wide refactor** — a single mechanical change (like renaming a column) whose **blast radius** fans across the whole codebase, breaking thousands of call sites at once so no vertical slice can land green. The drafting step now points at two co-located reference blocks: the **Vertical slice rules** for ordinary tracer bullets, and **Wide refactors**, which slices the change by **expand–contract** (expand the new form beside the old, migrate call sites in batches sized by blast radius, then contract the old form away) so CI stays green batch to batch — or, when it can't, only at a final integrate-and-verify issue. The issue body template moves into Reference too.

- [#464](https://github.com/mattpocock/skills/pull/464) [`386d4ff`](https://github.com/mattpocock/skills/commit/386d4ff719a7c420ad1454232d0436b01f1b8c17) Thanks [@mattpocock](https://github.com/mattpocock)! - Unify the planning skills. **`to-prd` is renamed to `to-spec`** — "spec" is now the single through-line term (it still opens with "you may know this document as a PRD" for discoverability). **`to-plan` and `to-issues` are merged into one `to-tickets` skill, and `to-issues` is deleted.**

  `to-tickets` breaks a plan, spec, or conversation into a set of **tickets** — tracer-bullet vertical slices, each declaring its **blocking edges**. That one artifact reads two ways depending on the tracker `/setup-matt-pocock-skills` configured: a **local file** (`tickets.md`) writes the edges as text and you work it top-to-bottom by hand; a **real tracker** writes them as native blocking links, so any ticket whose blockers are done is on the frontier and several agents can run at once. The edges live in the ticket either way — the medium only decides whether anything acts on them in parallel.

  Publishing prefers the tracker's **native sub-issues** for parent → slice and **native blocking edges** for `Blocked by` where the tracker supports them, keeping the `## Parent` / `## Blocked by` body sections as the fallback. The "What to build" template points at where a `/prototype`'s code lives rather than inlining a snippet from it.

  `ask-matt`'s main flow now routes `idea → /to-spec → /to-tickets → /implement`, and there are human-facing docs pages at `docs/engineering/to-spec.md` and `docs/engineering/to-tickets.md`.

- [#464](https://github.com/mattpocock/skills/pull/464) [`0557d57`](https://github.com/mattpocock/skills/commit/0557d57579d9b3d39839fdaf8d4a6542b17539ce) Thanks [@mattpocock](https://github.com/mattpocock)! - Settle wayfinder's place in the docs as a **situational on-ramp**, not the new main entry flow — the grill-led _idea → ship_ chain stays the front door (crowning wayfinder as the default spine is a v2-sized move, not a 1.1). The **`ask-matt`** router now names wayfinder's concrete triggers — a greenfield project or a huge feature build, too big for one session — and the two grill front doors (**`grill-me`**, **`grill-with-docs`**) signpost _up_ to wayfinder for the effort that's too big to hold in one session, so the on-ramp is discoverable from where a reader actually starts.

- [#464](https://github.com/mattpocock/skills/pull/464) [`639df6e`](https://github.com/mattpocock/skills/commit/639df6e7386dfddc739b2aecdeff37a876f2483b) Thanks [@mattpocock](https://github.com/mattpocock)! - Graduate and reframe **`wayfinder`** — the skill for planning a huge chunk of work, more than one agent session can hold. It moves out of `in-progress/` into `engineering/` (plugin entry, top-level + Engineering READMEs under **User-invoked**, a docs page at `docs/engineering/wayfinder.md`, and a route in `ask-matt`), landing as a mature skill. The rename and reframe that got it there:

  - **`decision-mapping` is renamed to `wayfinder`**, invoked as `/wayfinder`. "Decision map" was jargony and inaccurate — only one ticket type is actually a decision. The reframe charts a route through a foggy problem instead, giving one coherent leading-word frame — **fog of war**, **frontier**, **the map** — rather than an invented term layered on top.
  - **Destination as the leading word.** Wayfinding finds the _way_ to a destination; it doesn't charge at building it. Naming the destination is the first act of charting — it fixes the scope and shapes every ticket — so the map gains a `## Destination` field every session orients to, and triage pins it before any ticket exists.
  - **Plan, don't do.** The map produces **decisions, not deliverables**; it's done when nothing is left to decide before someone builds the thing. An effort can override this in its Notes.
  - **The map is an index, not a store.** A decision lives in exactly one place — its ticket — so the map only gists and links, never restates; graduating fog into a ticket clears the graduated patch so nothing lingers in two places.
  - **Collaborative by default.** The map moves off a local Markdown file onto the repo's issue tracker: a single `wayfinder:map` issue whose tickets are its child issues — one shared URL the team can watch. Sessions load the map at low resolution and zoom into tickets on demand. Wayfinder stays tracker-agnostic (GitHub, GitLab, local-markdown) behind a pointer in `docs/agents/issue-tracker.md`, and `setup-matt-pocock-skills` seeds the "Wayfinding operations" section.
  - **Claim by assignment, not a label.** A session claims a ticket by assigning it to the driving dev — the assignee _is_ the claim — freeing the label vocabulary to `wayfinder:<type>` alone.
  - **Native blocking.** Blocking prefers the tracker's native dependency relationship, which renders the frontier visually in the tracker's own UI so the human sees what's takeable without opening the map. GitHub and GitLab templates spell out the native recipe, with a body-convention fallback.
  - **Fog vs. out of scope, split.** Two plainly-named map sections — `## Not yet specified` (in-scope fog that graduates as the frontier advances) and `## Out of scope` (work ruled beyond the destination, closed, never graduating) — so beyond-destination work no longer reads as takeable frontier.
  - **A fourth `task` ticket type.** For literal manual work that blocks a decision (provisioning access, moving data, signing up for a service) — the one type that _does_ rather than decides, earning its place by unblocking a decision.
  - **HITL / AFK ticket classification.** Every ticket type is **HITL** (human in the loop — grilling, prototype) or **AFK** (agent alone — research; task is either). A HITL ticket only resolves through the live exchange, so "wait for the human" falls out of the label — a grilling agent that answers its own questions has, by definition, broken HITL. (This fixes students' reports of `/wayfinder` grilling _itself_ instead of the human.)
  - **No-fog early exit restored.** If the opening breadth-first grilling surfaces no fog, the journey is small enough for one session — so it stops and asks how you'd like to proceed rather than building a map nobody needs.

### Patch Changes

- [#464](https://github.com/mattpocock/skills/pull/464) [`639df6e`](https://github.com/mattpocock/skills/commit/639df6e7386dfddc739b2aecdeff37a876f2483b) Thanks [@mattpocock](https://github.com/mattpocock)! - Reshape **`tdd`** into a reference-only skill and add a missing anti-pattern.

  **Reference-only.** The red → green → refactor loop is anchored by leading words the model already holds, so the step-by-step Workflow was largely restating the loop. Dropped the Workflow and per-cycle checklist; folded their one durable idea — vertical slices / tracer bullets — into the Anti-patterns section and a short Rules-of-the-loop list. Introduced **seam** as the leading word for where tests go: test only at pre-agreed seams, confirmed with the user before any test is written. Also dropped the refactor stage — TDD is now red → green; refactoring belongs to the review stage, so the refactor rule and `refactoring.md` moved out (its home is `code-review`).

  **Tautological tests.** Added the tautological-test anti-pattern: a test whose assertion is recomputed the way the code computes it passes by construction and gives zero confidence — distinct from the implementation-coupling anti-pattern already covered. Added as a peer at the same sites: a Philosophy principle (expected values must come from an independent source of truth), a checklist gate, and a BAD/GOOD example pair in `tests.md`.

- [`e00eadb`](https://github.com/mattpocock/skills/commit/e00eadb4bb32c3d5a631ead1a5ed5d6a7c5f74e2) Thanks [@mattpocock](https://github.com/mattpocock)! - Extend the **`triage`** skill to triage external pull requests, treating a PR as an issue with attached code that runs through the same roles and state machine. PRs flow inline alongside issues (gated by a per-repo setup toggle), discovery surfaces only external PRs, the bug-only "reproduce" step is generalized into a single "verify the claim" step, and a redundancy check resolves already-implemented requests to `wontfix` without polluting the out-of-scope knowledge base. `setup-matt-pocock-skills` gains the PRs-as-a-request-surface toggle for GitHub/GitLab.

- [#472](https://github.com/mattpocock/skills/pull/472) [`d869d45`](https://github.com/mattpocock/skills/commit/d869d45afc32beab1c2d1350f8de5e81589512cd) Thanks [@mattpocock](https://github.com/mattpocock)! - Fix **`wayfinder`** hardcoding the issue-tracker doc path, which broke the indirection the rest of the suite relies on.

  `to-issues`, `to-prd`, and `triage` never name a path — they resolve the tracker through the `### Issue tracker` block that `setup-matt-pocock-skills` writes into `CLAUDE.md` / `AGENTS.md`, which points at the tracker doc wherever it lives. Wayfinder instead pinned the literal `docs/agents/issue-tracker.md`, so in a repo that keeps its agent docs elsewhere it silently fell back to the local-markdown tracker — even one whose `CLAUDE.md` clearly declares GitHub issues. It now resolves the doc via that same pointer and reads its "Wayfinding operations" section by name, keeping the indirection consistent across the suite.

## 1.0.1

### Patch Changes

- [`d20ee26`](https://github.com/mattpocock/skills/commit/d20ee2684e2a9442698ac3c1e0f2c5b68c4cf296) Thanks [@mattpocock](https://github.com/mattpocock)! - Make the **`teach`** skill reuse-first. Lessons are now built from reusable **components** in `./assets/` — stylesheets, quiz widgets, simulators, diagram helpers. Reuse is the default: the agent reads `./assets/` before authoring a lesson, builds from what's there, and extracts anything new and reusable into a component rather than inlining it.

## 1.0.0

### Major Changes

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Add the **`ask-matt`** skill — a user-invoked router that points you at the right skill or flow for your situation.

  **Breaking:** `ask-matt` routes over the other user-invoked skills in this repo, so it expects them to be installed.

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Add the shared design skills and rewire existing skills onto them.

  - New **`codebase-design`** skill — the deep-module vocabulary (module, interface, depth, seam, adapter) and the principles for putting a lot of behaviour behind a small interface. The language that previously lived in `improve-codebase-architecture/LANGUAGE.md` now lives here, generalized for reuse across skills.
  - New **`domain-modeling`** skill — actively build and sharpen a project's domain model, stress-testing terms against the glossary and keeping `CONTEXT.md` and ADRs current.
  - `improve-codebase-architecture` now draws its architecture vocabulary from `/codebase-design` and its domain model from `/domain-modeling`.
  - `tdd` now leans on `/codebase-design` for interface-design guidance — its inline `deep-modules.md` / `interface-design.md` notes were removed in favour of the shared skill.
  - `grill-with-docs` now builds the domain model inline via `/domain-modeling`.

  **Breaking:** these skills now depend on the new `codebase-design` / `domain-modeling` skills, so you must install them too.

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Remove the **`caveman`** and **`zoom-out`** skills.

  - `caveman` was a duplicate of another skill I was testing and was never meant to be public.
  - `zoom-out` went unused in practice, so it's been removed from the repo.

  **Breaking:** both skills have been removed.

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Rename the **`diagnose`** skill to **`diagnosing-bugs`**.

  **Breaking:** invoke it as `/diagnosing-bugs` — the old `/diagnose` name no longer exists.

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Replace **`write-a-skill`** with **`writing-great-skills`**.

  - Removed `write-a-skill`.
  - Added `writing-great-skills` (plus its `GLOSSARY.md`) — a reference for writing and editing skills well: the vocabulary and principles that make a skill predictable, hunting no-ops down to the sentence level.
  - Exposed `grilling` as a model-invoked skill — the reusable interview loop behind `grill-me` and `grill-with-docs`.

  **Breaking:** `write-a-skill` has been removed; use `writing-great-skills` instead.

### Minor Changes

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Add the **`resolving-merge-conflicts`** skill — a loop for resolving an in-progress git merge or rebase conflict. Standalone, with no dependencies on other skills.

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Rename the skill taxonomy from **Commands / Skills** to **User-invoked / Model-invoked** across the docs, and add `docs/invocation.md` defining the split: user-invoked skills are reachable only when you type them and exist to orchestrate; model-invoked skills can also be reached automatically when the task fits. A user-invoked skill may invoke model-invoked skills, but never another user-invoked one.

### Patch Changes

- [`47bde84`](https://github.com/mattpocock/skills/commit/47bde84da032afb2e5058f997f3bbca47d321dbd) Thanks [@mattpocock](https://github.com/mattpocock)! - Tighten the **`review`** skill: fail-fast ref check, single-sourced rules, and no-op cuts.
