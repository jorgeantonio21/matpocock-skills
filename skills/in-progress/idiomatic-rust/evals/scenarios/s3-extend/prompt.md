Add session renewal to this crate.

- `renew` extends an active session's expiry by the configured lease, measured from the time of the call.
- A session that has expired cannot be renewed. An unknown session id cannot be renewed. A session can be renewed at most `max_renewals` times, a new config field where zero means no renewals. When a renewal does not happen, the caller needs to know which of these three was the reason.
- When a renewal does happen, the caller needs the new expiry.

Add tests for each case. `cargo test` must pass when you finish. Write the code you would put in a pull request.
