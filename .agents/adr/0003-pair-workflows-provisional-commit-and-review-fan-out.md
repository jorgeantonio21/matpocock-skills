# Pair workflows: provisional commits and a review fan-out

`implement-by-commit` and `implement-by-plan` build a commit sequence with one agent, and the human is the only reviewer. The fork wanted a second agent in the loop: a reviewer that runs before every commit lands, using `code-review` for its Standards and Spec axes and adding two of its own, Bugs proven by running them and Craft judged against the house idiom. This records how that reviewer sees a commit, how it is shaped, and where its parts live. The two original skills stay untouched; the workflows are new skills, `pair-by-commit` and `pair-by-plan`, in the `in-progress/` bucket until an acceptance run promotes them.

## Provisional commit, then review, then amend

`code-review` diffs `<fixed-point>...HEAD`, which excludes the working tree, and reusing it unchanged was a requirement (it is upstream's skill, and editing it costs a conflict on every sync). So the implementer commits **before** the review: a **provisional commit** with the plan's title as subject, sitting at the top of the branch. The review runs against `HEAD~1`. Fixes and gate feedback are folded in by `git commit --amend`. In `pair-by-commit` the human's gate reads `git show HEAD` rather than the working tree; approval finalises the commit, amending the message only if the user changed it.

Invariant: **only the top, unpushed commit is ever amended.** Nothing below it is touched, and nothing is pushed by either workflow. Upstream's own docs page for `code-review` recommends the same order: commit, review, amend.

The alternative was a review skill carrying its own Standards and Spec briefs adapted to a working-tree diff. It keeps the original never-commit-before-approval contract, and it drifts from upstream's `code-review` from the first edit. Rejected.

## A fan-out at the orchestrator level, not a persistent reviewer agent

The reviewer is the `full-review` skill, run in the orchestrator's context: it calls `code-review` (which spawns Standards and Spec) and spawns `bug-hunter` and `craft-reviewer`, four sub-agents in parallel, then writes one verdict file. It is not a single long-lived reviewer agent continued by message.

Reasons: `code-review` stays byte-identical; its rule that axes are reported separately and never reranked is kept; and no sub-agent spawns sub-agents, which is where the runaway-agent bug reported against `code-review` lives. Memory across commits comes from the verdict files under `.scratch/<slug>/reviews/`, passed to each later review as pointers: file-based, resumable, harness-neutral.

## Two tiers, and a bounded fix loop

Upstream's docs warn that `code-review` never converges when looped until clean, because half of what it reports is judgement. So findings are tiered by rule, within each axis: **blocking** is a proven bug, a test that cannot fail, a spec requirement missing or wrong, or a documented-standard breach; **advisory** is everything else. Only blocking findings start a fix round, a re-check runs only the axes that blocked, and there are at most two rounds per commit. After that, `pair-by-commit` shows the blocker at the gate and `pair-by-plan` stops on drift. Advisory findings are never applied unattended; they accumulate in the review files and are reported at close-out.

## Where the agents live

Agent definitions are Claude Code files with frontmatter (tools, model, preloaded skills, turn budget). They live in a root `agents/` folder, because `skills/*/agents/` already holds the Codex `openai.yaml` metadata every skill carries. `scripts/link-agents.sh` links them into `~/.claude/agents` for local use; `scripts/link-skills.sh` stays byte-identical to upstream. On promotion the plugin manifest lists them under `agents`.

The workflows are Claude Code first. The agent files have no Codex equivalent, and ADR 0002 already defers a native Codex plugin. A harness without named agents can still run `full-review` by handing each agent file's body to a general sub-agent as its brief; that fallback is not written into the skills yet.

## Invariants this creates

- Only the top, unpushed commit is amended; neither workflow pushes or rebases.
- One writer per tree: the implementer writes in the working tree, review sub-agents read it, the bug hunter writes only in its own throwaway worktree, the orchestrator writes only under `.scratch/`.
- `code-review` is called, never copied.
- A finding blocks only by the tier rules; Craft never blocks.
