---
name: full-review
description: "Four-axis review of the diff since a fixed point: Standards and Spec via code-review, plus Bugs (proven by running them) and Craft (idiom and design). Writes one verdict file split into blocking and advisory findings. Use when the user wants a branch, PR, or commit reviewed for bugs and idiom as well as standards and spec, asks to fully review since a fixed point, or when a workflow skill needs a verdict before a commit lands."
---

# Full Review

Review the diff between a fixed point and `HEAD` along four axes, run as parallel sub-agents, and write one **verdict file**:

- **Standards** and **Spec**: the two axes of `code-review`, unchanged.
- **Bugs**: defects **proven** by running them, plus tests that cannot fail. The `bug-hunter` agent.
- **Craft**: idiom and design, each finding with its rewrite. The `craft-reviewer` agent.

The verdict is **blocked** or **clean**. Findings sort into two tiers, and only the first can block:

- **Blocking**: a proven bug; a test that cannot fail; a spec requirement missing, partial, or implemented wrongly; a breach of a documented repo standard.
- **Advisory**: everything else: smells, suspected bugs, craft and design judgement calls.

Axes are reported separately and never reranked against each other (see _Why two axes_ in `code-review`). The tiers come from the rules above, applied within each axis, never from weighing one axis against another.

## Arguments

- **Fixed point** (required): a commit, branch, tag, `main`, `HEAD~1`. Ask if it wasn't given.
- **Output path** (optional): where the verdict file goes. Default `.scratch/reviews/<short-sha>.md`. Workflow skills pass `.scratch/<feature-slug>/reviews/NN.md`.
- **Spec** (optional): a path or reference. Otherwise found the way `code-review` finds it.
- **Scope** (optional): when the diff is one commit of a commit plan, the plan file and the commit number. The Spec axis then judges completeness against that commit's plan entry rather than the whole target, and work belonging to a later commit counts as scope creep.
- **Axes** (optional): a subset, for a **targeted re-check** after a fix round. Default all four.
- **Previous reviews** (optional): earlier verdict files to hand the sub-agents, so a finding from commit 2 is in view when commit 5 is reviewed.

## Process

### 1. Pin the fixed point

`git rev-parse <fixed-point>` must resolve and `git diff <fixed-point>...HEAD --stat` must be non-empty. Stop here otherwise, before anything is spawned.

Capture the diff command (`git diff <fixed-point>...HEAD`, three-dot), the commit list (`git log <fixed-point>..HEAD --oneline`), and the repo's typecheck and test commands from its docs or package manifest.

### 2. Run the four axes in parallel

Dispatch all of these at once, then wait for all of them:

- **Standards and Spec**: call the Skill tool with "code-review", passing the fixed point, the spec, and the scope. Its two sub-agents run inside it. Add one line to each of their briefs: _Do not invoke code-review or spawn agents; perform this review directly._
- **Bugs**: spawn the `bug-hunter` agent.
- **Craft**: spawn the `craft-reviewer` agent.

The two agent briefs are **context pointers**, nothing pasted: the diff command, the commit list, the fixed point, the spec path, the scope, the test commands, and the previous review files where the caller gave them. Each agent carries its own protocol and the Pragmatic baseline already.

On a targeted re-check, dispatch only the axes named and pass the previous verdict file, so each agent re-examines its own blocking findings first.

### 3. Write the verdict file

<verdict-template>

# Review: <what was reviewed>

**Fixed point:** <ref> (<sha>). **Commits:** <n>. **Verdict:** blocked | clean

## Blocking

- [Bugs] <file:line> <finding>. Proof: `<command>`, output: <the failing line>. Fix: <the move>.
- [Bugs] Test cannot fail: <test>, stayed green under <mutation>.
- [Spec] <finding>, quoting: "<spec line>".
- [Standards] <finding>, citing <file>: <rule>.

## Advisory

- [Standards] possible <smell>: <hunk>.
- [Craft] <finding>. Idiomatic form: <rewrite>.
- [Bugs] Suspected: <finding>. Trigger: <input>.

## Standards

## Spec

## Bugs

## Craft

</verdict-template>

The four axis sections carry each report verbatim or lightly cleaned. Sort every finding into a tier by the rules at the top: a finding that meets no blocking rule is advisory whatever its author called it. Where the Spec axis was skipped for want of a spec, say so under `## Spec` and in the reply.

### 4. Reply

Reply with the verdict, the verdict file's path, the count of blocking and advisory findings per axis, and the worst issue within each axis. Name no single winner across axes.
