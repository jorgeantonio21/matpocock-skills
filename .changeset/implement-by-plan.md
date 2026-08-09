---
"mattpocock-skills": minor
---

Add **`implement-by-plan`**, a user-invoked engineering skill that builds a spec or ticket as the same planned sequence of commits as `implement-by-commit`, with the gates removed. The plan is the **contract**: agreed up front — slicing, order, open decisions, target branch — and written to `.scratch/<feature-slug>/commits.md`. Once approved, the run goes unattended to the last commit, driving `/tdd` at the agreed seams and ticking each commit off as it lands; every commit is green on its own, and messages carry no trailer lines. The only interrupt is **drift** — work that proves the plan wrong stops the run to re-agree it. `/code-review` runs once at close-out over the whole landed sequence.

`ask-matt` routes to it between `/implement` and `/implement-by-commit`, picked on how much you want to review — nothing, the plan once, or every commit — and it has a docs page at `docs/engineering/implement-by-plan.md`.
