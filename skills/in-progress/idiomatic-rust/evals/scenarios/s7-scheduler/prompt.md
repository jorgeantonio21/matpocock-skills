Extend this inference request scheduler.

- Add `Scheduler::set_policy(&mut self, policy: Policy)`. A policy change applies to submissions after it. A request already queued keeps its admission, whatever the new limits say, and `next_batch` still serves it.
- Add `Scheduler::retire_model(&mut self, model: &ModelId) -> RetireOutcome`. Retiring removes the model's queue. `RetireOutcome::Retired { drained: Vec<Request> }` carries the requests that were queued, in submission order, so the caller can reply to each client. Retiring a model a second time is `RetireOutcome::Rejected(RetireReason::AlreadyRetired)`. Retiring a model no request ever named is `RetireOutcome::Rejected(RetireReason::UnknownModel)`.
- A submission for a retired model is rejected with `RejectReason::ModelRetired`.

Keep the existing public API. Add tests. `cargo test` must pass when you finish. Write the code you would put in a pull request.
