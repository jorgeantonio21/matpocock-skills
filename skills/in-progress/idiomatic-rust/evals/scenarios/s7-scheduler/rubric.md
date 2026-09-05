# s7-scheduler rubric

An inference request scheduler: a stateful service component with no I/O. The prompt asks for a policy change that keeps earlier admissions, and a retirement whose outcome is the record of what it removed. One defect is planted in `start/src/lib.rs` and not named: `next_batch` compares with `<`, so a request that exactly fills the budget is left behind. `verify/tests/semantic.rs` checks all of it through the public API. `reference/` is one solution that passes it.

Mark each row `bare`, `skill`, `both`, or `neither`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Invariants: Contextual admission | Queued requests are not re-checked at `set_policy` or at `next_batch`; no eviction on a stricter limit; no `Admitted` type that is validated against the current policy at dequeue | | |
| 2 | Invariants: Contextual admission | The doc comment on `set_policy` says which requests the new policy applies to | | |
| 3 | Invariants: Authoritative transition result | `retire_model` returns the drained requests from the one `remove`; the caller is not expected to call `next_batch` or `queued_len` to learn them | | |
| 4 | Errors: Rejection is an outcome | `RetireOutcome` and `RetireReason` as asked; `#[must_use = "reason"]` on `RetireOutcome`; `ModelRetired` on `RejectReason` | | |
| 5 | Shape: Make invalid states impossible | Retired models are a set (or an enum state per model), not a `bool` beside the queue; `UnknownModel` and `AlreadyRetired` are told apart by state, not by a heuristic | | |
| 6 | Flow: Match exhaustively | A `match` on the new reasons names every variant; no `_` arm | | |
| 7 | Flow: Guards first, happy path flat | `retire_model` exits on `AlreadyRetired`, then `UnknownModel`, then drains at the lowest indentation | | |
| 8 | Words: Tests read as sentences | One `test_` per rule in the prompt, the grandfathered request in a test, retire-twice and never-seen in a test | | |
| 9 | Generality | No trait, no channel, no async, no lock, no new dependency; the scheduler stays a plain struct | | |
| 10 | Correctness | `score.sh`: the external tests pass, including the exact-budget batch | | |
| 11 | Check | `score.sh`: the check command reports zero findings in both runs | | |
