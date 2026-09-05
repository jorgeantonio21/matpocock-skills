# s9-plain rubric

An ordinary crate of four text helpers, written well, with one defect: `wrap` drops a word longer than the width, where its doc comment says the word stands on its own line. The right result is that fix and a test for it, and nothing else. This scenario measures restraint: a wrapper, an alias, a dependency, a trait, or a typestate would help none of these functions, and the evaluator accepts the code that keeps them as they are.

Diff each arm's tree against `start/`. `score.sh` prints the diff stats. Mark each row `bare`, `skill`, `both`, or `neither`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Correctness | The `continue` on a long word is gone; the word is flushed onto its own line; a `test_` covers it | | |
| 2 | Restraint: When a wrapper is not the answer | `width: usize` stays `usize`; no `Width` newtype, no `NonZeroUsize`, no `Prefix` type | | |
| 3 | Restraint | No new dependency; no `thiserror`, `anyhow`, or `itertools` added to a crate that never fails | | |
| 4 | Restraint | No `Result` on a function that cannot fail; no `Option` where the empty `Vec` already says "nothing" | | |
| 5 | Restraint | The module doc, the four doc comments, and the existing tests are kept; no rewrite of a passing test | | |
| 6 | Restraint | `score.sh` diff stats: lines added and removed in the tens, not the hundreds; no new file | | |
| 7 | Words: Comments say why | Any comment added states a reason; the existing "strict comparison" comment stays | | |
| 8 | Correctness | `score.sh`: the external tests pass; the crate's own tests pass | | |
| 9 | Check | `score.sh`: the check command reports zero findings in both runs | | |
