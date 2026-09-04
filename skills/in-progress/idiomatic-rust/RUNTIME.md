# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Two regimes.** The **edge** is the async side: tokio tasks that do I/O, handle thousands of events per second, and can afford a short lock or one allocation per event. The **hot path** is a loop that handles a message in microseconds: the pinned engine thread, a ring-buffer consumer, a decoder. On the hot path, a lock, a heap allocation, or a syscall per message is a defect. Every rule below applies to the edge unless it says hot path. Measure before you move a rule across that line.

- **Cancel first, at the edge.** Pass a `CancellationToken` to every long-lived task. Put `cancel.cancelled()` as the first arm of every `select!`, and write `biased;` as the first line of the `select!`. With `biased;`, tokio polls the arms in the written order and skips the random branch shuffle, so the cancel arm is checked first. Create one token per subsystem at setup with `child_token()`. Do not signal shutdown with a `broadcast` channel of `()`.

  ```rust
  tokio::select! {
      biased;
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(backoff) => {}
  }
  ```

- **Cancellation on the hot path.** `CancellationToken::is_cancelled()` takes a mutex on every call, and `cancelled()` takes it on every poll. `clone()`, `child_token()`, and drop take the mutex too and can restructure the token tree. Under contention with `cancel()`, these calls can stall for milliseconds. So: do not create, clone, or drop a token per request. Do not poll `cancelled()` per message in a fast loop. Bridge the token to an atomic once, in one watcher task, and read the atomic in the loop. The flag carries no data, so `Relaxed` ordering is enough. A sync OS thread reads the same atomic. Check the flag once per batch, not once per message.

  ```rust
  let stop = Arc::new(AtomicBool::new(false));
  tokio::spawn({
      let stop = Arc::clone(&stop);
      async move { cancel.cancelled().await; stop.store(true, Ordering::Relaxed); }
  });
  // In the hot loop, once per drained batch:
  if stop.load(Ordering::Relaxed) { break; }
  ```

- **The spawn shape.** Write `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`. State the exit conditions in the doc comment. Keep each `select!` arm to one line. Move a longer arm body into a named `async fn`.

- **Own every handle.** Store a `JoinHandle` or await it. Do not drop it. Use `JoinSet` when you read the results. Use `TaskTracker` when you do not. At shutdown, call `tracker.close()` and then `tracker.wait().await`. Both operate at spawn granularity, so they cost nothing per message.

- **One channel per role, at the edge.** Use `mpsc` for a work queue. Use `oneshot` for a request and its reply. Use `watch` for current state that a task reads before it acts. Use `broadcast` for fan-out, and handle `Lagged` explicitly. These channels take a lock or a semaphore per operation, and a `oneshot` allocates per request. On the hot path, use an SPSC ring such as `rtrb` between two dedicated threads, or a bounded `ArrayQueue` for many producers and one consumer. Read a `watch` value once per batch, not per message.

- **A queue and a thread at the boundary.** Push from async producers into a lock-free queue. Drain the queue on one OS thread. Create that thread with `thread::Builder::new().name(..)` and pin it to a core. Drain everything that is ready, process the batch, then check the stop flag. Send a reply through a `oneshot` that you find by request id at the edge. On the hot path, correlate by id in a preallocated slab, not in a map that allocates. Name every long-lived OS thread, so it appears under its own name in a profile. Do not call `block_on` inside async code. Do not share a mutex between the edge and the hot path.

  ```rust
  thread::Builder::new()
      .name(format!("leader-transport-{service}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, stop)
      })?;
  ```

- **Async stays out of traits.** Write async code in free functions and inherent `async fn`. Define a trait seam over sync methods, such as `PollAcks` or `TryDequeue<T>`, with a no-op impl for tests. Do not use `#[async_trait]`. Use a native `async fn` in a trait only when you never need a trait object.

- **Locks and `.await`.** Do not hold a `std::sync` or `parking_lot` guard across an `.await`. Drop the guard before the await, or move the work into a sync fn. Use `parking_lot` for a short critical section at the edge. Alias it with `use parking_lot::Mutex as SyncMutex;` when a tokio lock is in scope. Do not use `tokio::sync::Mutex`, `Notify`, or any lock on the hot path.

- **Use the named combinators, at the edge.** Use `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered`. Use `tokio::select!` and `tokio::join!`. Do not use `futures::select!`. Do not call `.boxed()` per message. These combinators allocate, so keep them off the hot path. Use `Framed` with a `Decoder` at the wire edge (see [CRATES.md](CRATES.md)).
