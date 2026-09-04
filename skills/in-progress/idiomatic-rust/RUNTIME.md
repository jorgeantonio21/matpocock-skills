# Runtime

Async tasks, OS threads, and the boundary between them. The runtime branch of [idiomatic-rust](SKILL.md); same reading, *what it looks like* → *the move*.

- **Cancel first.** An `Arc<AtomicBool>` polled by an async loop; a `broadcast::channel::<()>` every task subscribes to; a task that ends only when its channel drops. → A `CancellationToken` passed to every long-lived task, `select!` with `cancel.cancelled()` as the first arm, `child_token()` per subsystem, `drop_guard()` where unwinding must cancel. An `AtomicBool` read by a sync OS thread stays correct; this rule is for tasks.

- **The spawn shape.** `tokio::spawn` inline in a handler; a task whose exit conditions live in the reader's head. → `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>` with the exit conditions in its doc comment. `select!` arms are one line, delegating anything longer to a named `async fn`. "Sleep unless cancelled" is the two-arm `select!` with an empty body.

  ```rust
  tokio::select! {
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(backoff) => {}
  }
  ```

- **Own the handles.** `Vec<JoinHandle<()>>` joined in a loop; a `JoinHandle` dropped on the floor. → `JoinSet` when results are read, `TaskTracker` when they are not; `tracker.close(); tracker.wait().await` at shutdown.

- **Channels by role.** One `mpsc` for everything; a lock shared between a handler and a worker. → `mpsc` for a work queue, `oneshot` for request/response correlation, `watch` for current state read before acting, `broadcast` for fan-out with `Lagged` handled explicitly.

- **The sync/async boundary is a queue and a thread.** `block_on` inside async; a mutex shared by the async edge and the engine thread. → Async producers push into a lock-free queue (`ArrayQueue`, `rtrb`); one named, core-pinned `thread::Builder` thread drains it. The reply path is correlate-by-id into a `oneshot`. Every long-lived OS thread is named, so it shows up as itself in a profile.

  ```rust
  thread::Builder::new()
      .name(format!("leader-transport-{service}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, cancel)
      })?;
  ```

- **Async stays out of traits.** `#[async_trait]`; `Box<dyn Future>` in a trait seam. → Free functions and inherent `async fn`; a trait seam over sync methods (`PollAcks`, `TryDequeue<T>`) with a no-op impl for tests; native `async fn` in a trait only where no trait object is needed.

- **Locks and `.await`.** A `std::sync` or `parking_lot` guard alive across an `.await`; `tokio::sync::Mutex` on a hot path. → Drop the guard before the await or move the work into a sync fn; `parking_lot` for short critical sections, aliased (`Mutex as SyncMutex`) whenever a tokio lock is in scope. Message passing first (Ownership).

- **Combinators worth naming.** A hand-written loop over a stream of futures; `futures::select!`; `.boxed()` per message. → `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, `FuturesUnordered`; `tokio::select!` and `tokio::join!`; `Framed` with a `Decoder` at the wire edge ([CRATES.md](CRATES.md)).
