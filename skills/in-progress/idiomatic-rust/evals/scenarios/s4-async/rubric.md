# s4-async rubric

Mark each row `bare`, `skill`, `both`, or `neither`. Most rows come from `RUNTIME.md`; the skill arm has to follow the pointer to reach them, so `meta.json` should show a `Read` of `RUNTIME.md`.

| # | Entry | What to look for | Verdict | Notes |
| --- | --- | --- | --- | --- |
| 1 | Runtime: Cancel first, at the edge | A `CancellationToken` passed to every worker; `select!` with `biased;` and `cancel.cancelled()` as the first arm; child tokens per subsystem; shutdown is not a `broadcast` of `()` or a shared `AtomicBool` polled in a loop | | |
| 2 | Runtime: Only cancel-safe futures in `select!` | The arms are `recv`, `cancelled`, `sleep`, or a pinned future; no `send` on a bounded channel inside a `select!` arm | | |
| 3 | Runtime: The spawn shape | `fn spawn_worker(deps, cancel: CancellationToken) -> JoinHandle<()>` or the `TaskTracker` equivalent; exit conditions in the doc comment; each `select!` arm one line with the body in a named `async fn` | | |
| 4 | Runtime: Own every handle | Every `JoinHandle` stored or awaited; `JoinSet` or `TaskTracker` for the workers; `tracker.close()` then `tracker.wait().await` at shutdown | | |
| 5 | Runtime: One channel per role | `mpsc` for the job queue, `mpsc` or `oneshot` for results; no `Arc<Mutex<Vec<Result>>>` for the results, no `Arc<Mutex<VecDeque>>` for the queue | | |
| 6 | Runtime: Locks and `.await` | No guard held across an `.await`; no `tokio::sync::Mutex` where a channel does the job | | |
| 7 | Runtime: Share without a lock | A counter or flag is an atomic, not a mutex around an integer | | |
| 8 | Errors: Rejection is an outcome | A timed-out job is an outcome variant (`JobOutcome::TimedOut`), not an `Err` | | |
| 9 | Shape: The compiler is the guardrail | `JobName`, a `Job { name, duration, timeout }` struct; worker count as a typed value or `NonZeroUsize` | | |
| 10 | Ownership: Time and randomness are inputs | Durations come from the job; `tokio::time::timeout` wraps the run; nothing reads `Instant::now()` in the logic | | |
| 11 | Flow: Name every number | The worker count, the job list, and every duration are named constants | | |
| 12 | Words: Logs are structured | Per-job output goes through one place; `println!` only where stdout is the product (the summary in `main`), or `tracing` for the rest | | |
| 13 | Words: Tests read as sentences | `test_` prefix, `#[tokio::test(start_paused = true)]` with `tokio::time::advance`, no real sleep, values in the assert message | | |
| 14 | Correctness | Ctrl-c during a run still prints a summary with every submitted job counted once; the queue drains before the summary in the normal path | | |
| 15 | Check | `score.sh`: tests pass, the check command reports zero findings in both runs (the `print_stdout` relaxation applies to this binary) | | |
