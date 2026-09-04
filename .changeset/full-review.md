---
"mattpocock-skills": minor
---

Add the `full-review` skill (in-progress bucket, model-invoked) and the two agents behind it. `full-review` reviews the diff since a fixed point along four axes run in parallel: **Standards** and **Spec** by calling `code-review` unchanged, **Bugs** by the new `bug-hunter` agent, which proves each defect by running it in a throwaway worktree and mutation-probes every test in the diff, and **Craft** by the new `craft-reviewer` agent, which learns the house idiom from sibling files and attaches the idiomatic rewrite to each finding. It writes one verdict file: a verdict of blocked or clean, a **Blocking** list (proven bugs, tests that cannot fail, spec misses, documented-standard breaches), an **Advisory** list (everything else), and the four axis reports verbatim. Axes are never reranked against each other. It accepts a scope (one commit of a plan), a subset of axes for a targeted re-check, and earlier verdict files.

Agent definitions live in a new root `agents/` folder, linked locally by `scripts/link-agents.sh`.
