---
"mattpocock-skills": minor
---

Add **`pair-by-commit`** and **`pair-by-plan`** (in-progress bucket, user-invoked): the `implement-by-commit` and `implement-by-plan` loops with two agents on the job. An `implementer` agent builds each commit of the agreed plan in the working tree and lands it as a **provisional commit**; `full-review` then checks it on four axes (Standards, Spec, proven Bugs, Craft) and blocking findings go back to the implementer for a fix and amend, at most two rounds. In `pair-by-commit` the user still opens every gate, now with the review beside the code and `git show HEAD` as the thing to read; in `pair-by-plan` a clean verdict is the gate, the run goes unattended to the last commit, and a blocker that survives two rounds stops it as drift. Both close with a whole-sequence review. Advisory findings are never applied unattended. The two original skills are untouched. ADR 0003 records the provisional-commit and fan-out decisions.
