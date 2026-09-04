# s3-extend rubric

Diff each arm's `src/lib.rs` against `start/src/lib.rs`. Mark each row `bare`, `skill`, `both`, or `neither`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Errors: Rejection is an outcome | `renew` returns an outcome enum: renewed with the new expiry, or rejected with a reason (unknown, expired, limit reached). Not `Result<Instant, RenewError>` with the three rejections as `Err`, and not `Option` | | |
| 2 | Surface: Attributes say why | `#[must_use = "reason"]` on the outcome; `#[must_use]` on nothing that returns `()` | | |
| 3 | Shape: The compiler is the guardrail | Whether `max_renewals` and the per-session count got a type (`Renewals`) or stayed `u32`; either is defensible here, note the choice | | |
| 4 | Shape: Make invalid states impossible | The rejection reasons are an enum, not a `&'static str` or a `String`; no `bool` return anywhere | | |
| 5 | Flow: Guards first, happy path flat | `let Some(session) = self.sessions.get_mut(&id) else { return .. }`, then the expired guard, then the limit guard, then the renewal at the lowest indentation; a reason comment above each guard | | |
| 6 | Flow: Check arithmetic on external values | The renewal count is compared before it is incremented; no bare `+= 1` that could wrap; `checked_add` or `saturating_add` where a wrap is possible | | |
| 7 | Ownership: Time and randomness are inputs | `renew(&mut self, id, now: Instant)`; the new expiry is `now + lease`, not `expires_at + lease` | | |
| 8 | Flow: Match exhaustively | Any `match` on the new reason enum names every variant; no `_` arm | | |
| 9 | Words: Tests read as sentences | One `test_renew_*` per case (renewed, unknown, expired, limit, zero limit), the expected expiry derived in a comment, values in the assert message, the same `t0` origin pattern the file already uses | | |
| 10 | Words: Docs state the rule and the consequence | A doc comment on `renew` that states the rule (lease measured from the call; at most `max_renewals`); the module doc's numbered list gains a line for renewal | | |
| 11 | Surface: Names follow the std conventions | `RenewOutcome`, `RenewReason` (or `RejectReason`), no `get_` getters added; `Config` field named `max_renewals` as asked | | |
| 12 | Shape: Private by default | The per-session renewal count is a private field with a getter, not a `pub` field | | |
| 13 | Correctness | An expired session is rejected as expired even when the limit is also reached (or the order is documented); the count increments only on a successful renewal; `expire_all` still works after renewals | | |
| 14 | Check | `score.sh`: tests pass, the check command reports zero findings in both runs | | |
