---
"mattpocock-skills": minor
---

Add **`implement-by-commit`**, a user-invoked engineering skill that builds a spec or ticket as a planned sequence of human-reviewable commits rather than one. It agrees a commit plan up front and writes it to `.scratch/<feature-slug>/commits.md`, then builds the commits in order — driving `/tdd` at the agreed seams, and stopping at a **gate** before each one to present what it built and a suggested message. Feedback folds into the commit under review; `git commit` runs only on approval, and messages carry no trailer lines. `/code-review` runs once at close-out over the whole landed sequence, by which point every commit is in `HEAD` and inside the reviewed diff.

`ask-matt` routes to it beside `/implement`, picked on whether a human reads every commit, and it has a docs page at `docs/engineering/implement-by-commit.md`.
