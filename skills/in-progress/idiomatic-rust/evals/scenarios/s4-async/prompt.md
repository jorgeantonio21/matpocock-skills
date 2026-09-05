Build a worker pool as a binary crate in this directory. The crate is already initialised with `tokio` and `tokio-util` as dependencies; keep the name `fetchpool`. Add other dependencies with `cargo add` if you need them.

- A pool of N workers processes jobs from an in-memory queue. A job is a name and a simulated duration; a worker "runs" it by sleeping for that duration (no network).
- Each job has its own timeout. A job that exceeds it counts as timed out.
- A supervisor collects one result per job (done or timed out) and prints one summary line per job and a total when the pool has drained.
- On ctrl-c, the pool stops taking new jobs, in-flight jobs finish or time out, every task is joined, and the summary still prints. No job is lost or counted twice.
- `cargo run` runs a fixed built-in list of about ten jobs with N = 3.
- Tests cover the queue and the timeout behaviour without depending on wall-clock time (tokio's paused time is fine).

`cargo test` must pass when you finish. Write the code you would put in a pull request.
