# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Cancel first.** Pass a `CancellationToken` to every long-lived task. Put `cancel.cancelled()` as the first arm of every `select!`. Create a `child_token()` for each subsystem. Use `drop_guard()` where an unwinding task must cancel its children. Do not poll an `Arc<AtomicBool>` in an async loop. Do not signal shutdown with a `broadcast` channel of `()`. An `AtomicBool` that a sync OS thread reads is correct.

- **The spawn shape.** Write `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`. State the exit conditions in the doc comment. Keep each `select!` arm to one line. Move a longer arm body into a named `async fn`. Write "sleep unless cancelled" as a two-arm `select!` with an empty body.

  ```rust
  tokio::select! {
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(backoff) => {}
  }
  ```

- **Own every handle.** Store a `JoinHandle` or await it. Do not drop it. Use `JoinSet` when you read the results. Use `TaskTracker` when you do not. At shutdown, call `tracker.close()` and then `tracker.wait().await`.

- **One channel per role.** Use `mpsc` for a work queue. Use `oneshot` for a request and its reply. Use `watch` for current state that a task reads before it acts. Use `broadcast` for fan-out, and handle `Lagged` explicitly.

- **A queue and a thread at the boundary.** Push from async producers into a lock-free queue such as `ArrayQueue` or `rtrb`. Drain the queue on one OS thread. Create that thread with `thread::Builder::new().name(..)` and pin it to a core. Send a reply through a `oneshot` that you find by request id. Name every long-lived OS thread, so it appears under its own name in a profile. Do not call `block_on` inside async code. Do not share a mutex between the async edge and the engine thread.

  ```rust
  thread::Builder::new()
      .name(format!("leader-transport-{service}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, cancel)
      })?;
  ```

- **Async stays out of traits.** Write async code in free functions and inherent `async fn`. Define a trait seam over sync methods, such as `PollAcks` or `TryDequeue<T>`, with a no-op impl for tests. Do not use `#[async_trait]`. Use a native `async fn` in a trait only when you never need a trait object.

- **Locks and `.await`.** Do not hold a `std::sync` or `parking_lot` guard across an `.await`. Drop the guard before the await, or move the work into a sync fn. Use `parking_lot` for a short critical section. Alias it with `use parking_lot::Mutex as SyncMutex;` when a tokio lock is in scope. Do not use `tokio::sync::Mutex` on a hot path.

- **Use the named combinators.** Use `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered`. Use `tokio::select!` and `tokio::join!`. Do not use `futures::select!`. Do not call `.boxed()` per message. Use `Framed` with a `Decoder` at the wire edge (see [CRATES.md](CRATES.md)).
