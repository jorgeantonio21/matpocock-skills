---
"mattpocock-skills": minor
---

Add **`open-pr`**, a user-invoked engineering skill that ships the commits on the current branch as a pull request. It runs the repo's own checks before anything is pushed, rescues commits stranded on `main` onto a `<type>/ja/<short-description>` branch, and asks which repo and base the PR targets rather than inheriting `gh`'s fork default. The title and body are drafted from the diff alone — plain, factual, no trailer lines — and one **gate** shows the PR exactly as it will appear before it is pushed and created. The run ends at the PR's URL; the merge stays with the reviewer.

`ask-matt` routes to it as the step after the build skills, which commit to the current branch and have no PR mode, and it has a docs page at `docs/engineering/open-pr.md`.
