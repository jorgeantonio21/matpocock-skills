# s1-ratelimit rubric

Mark each row `bare`, `skill`, `both`, or `neither`. The entry column names the `SKILL.md` (or `RUNTIME.md`) entry the row tests.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Shape: The compiler is the guardrail | Client id, capacity, refill rate, and cost are their own types; `admit` does not take two bare numbers that a caller could swap | | |
| 2 | Shape: Parse at the boundary | Capacity and rate are validated once in a constructor with private fields; zero is unconstructible (`NonZeroU32`, or `new` returns `Option`/`Result`); the core never re-checks | | |
| 3 | Shape: Make invalid states impossible | No `bool` parameter or field; no sentinel; the idle period and retry wait are `Duration`, not a bare number of seconds | | |
| 4 | Errors: Rejection is an outcome | `admit` returns an outcome enum (admitted / rejected with `retry_after` / never admittable), not `Result<(), Error>`, `bool`, or `Option<Duration>` | | |
| 5 | Surface: Attributes say why | `#[must_use = "reason"]` on the outcome type | | |
| 6 | Errors: Library errors use `thiserror` | Config error carries the offending value in a field, message lowercase, no pre-formatted `String` | | |
| 7 | Ownership: Time and randomness are inputs | `admit` and `evict` take `now: Instant`; no `Instant::now()` outside tests; no `Clock` trait whose only second impl is the test | | |
| 8 | Ownership: Take what you use | Client key taken as `&str` or a `ClientId`; no `&String`; no clone of the key on the admitted path | | |
| 9 | Flow: Guards first, happy path flat | The never-admittable and rejected cases exit early; the admitted path sits at the lowest indentation | | |
| 10 | Flow: Name every number | No bare literal in the refill arithmetic; nanoseconds-per-second and the like are named | | |
| 11 | Flow: Check arithmetic on external values | Refill does not overflow on a long idle gap; the token count clamps at capacity; `cost` from the client is compared before it is subtracted | | |
| 12 | Surface: Names follow the std conventions | Getters without `get_`; `Outcome`, `Reason`, `Config` suffixes; conversions named by cost | | |
| 13 | Shape: Private by default | Fields private; `pub` only on the API surface | | |
| 14 | Words: Tests read as sentences | `test_` prefix, one behaviour per test, the expected value derived in a comment, values in the assert message, a named time origin, no sleep | | |
| 15 | Words: Docs state the rule and the consequence | Doc comments name an invariant (tokens never exceed capacity; a retry wait is zero only when admitted) rather than restating the signature | | |
| 16 | Check | `score.sh`: tests pass, the check command reports zero findings in both runs | | |
