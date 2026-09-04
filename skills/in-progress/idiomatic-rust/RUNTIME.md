# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Two regimes.** The **edge** is the async side: tokio tasks that do I/O and handle thousands of events per second. The edge can afford a short lock or one allocation per event. The **hot path** is a loop that handles a message in microseconds: a pinned worker thread, a ring-buffer consumer, a decoder. On the hot path, a lock, a heap allocation, or a syscall per message is a defect. Every rule below applies to the edge unless it says hot path. A web service, a CLI tool, or a batch job has no hot path, so only the edge rules apply there. Measure before you move a rule across that line.

- **Cancel first, at the edge.** Pass a `CancellationToken` to every long-lived task. Put `cancel.cancelled()` as the first arm of every `select!`, and write `biased;` as the first line of the `select!`. With `biased;`, tokio polls the arms in the written order and skips the random branch shuffle, so the cancel arm is checked first. Create one token per subsystem at setup with `child_token()`. Do not signal shutdown with a `broadcast` channel of `()`.

  ```rust
  tokio::select! {
      biased;
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(backoff) => {}
  }
  ```

- **Only cancel-safe futures in `select!`.** When one arm completes, `select!` drops the futures of the other arms. A future that has consumed input and not yet returned it loses that input. `recv`, `sleep`, `cancelled`, and `changed` are cancel safe. `read_exact`, `write_all`, and `send` on a bounded channel are not. Run those to completion outside the `select!`, or create the future once, pin it, and poll `&mut fut` across iterations.

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

- **Share without a lock.** Prefer a wait-free or lock-free structure to a lock, in both regimes. A lock blocks every thread that contends for it, and a blocked thread on the hot path misses its deadline. A wait-free operation completes in a bounded number of steps whatever the other threads do. A lock-free operation guarantees that some thread makes progress. Prefer wait-free where a structure offers it, lock-free otherwise. Use an atomic for a flag, a counter, or one small value: `AtomicBool`, `AtomicU64`, `AtomicUsize`. Use an SPSC ring such as `rtrb` between two dedicated threads; its push and pop are wait-free. Use `crossbeam::queue::ArrayQueue` (bounded) or `SegQueue` (unbounded) when several threads produce. Use `crossbeam-channel` or `flume` for a sync work queue with blocking receivers. Use `arc_swap::ArcSwap` to publish a snapshot that many readers load and one writer replaces. A concurrent map such as `dashmap` shards a lock. Use it only when readers and writers touch different keys, and keep it off the hot path. Take a lock only when one operation must read and write several fields together and no structure above fits. Do not use `std::sync::Mutex`. When a lock is the choice, use `parking_lot` and hold it for a few lines. Write a comment that says why no lock-free structure fits.

- **Atomic orderings.** Use `Relaxed` for a flag or a counter that carries no other data. Use `Release` on the store and `Acquire` on the load when the atomic publishes data that was written before the store. Use `SeqCst` only with a comment that names the ordering between two atomics that the code needs. Do not write `SeqCst` by default.

- **The spawn shape.** Write `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`. State the exit conditions in the doc comment. Keep each `select!` arm to one line. Move a longer arm body into a named `async fn`.

- **Own every handle.** Store a `JoinHandle` or await it. Do not drop it. Use `JoinSet` when you read the results. Use `TaskTracker` when you do not. At shutdown, call `tracker.close()` and then `tracker.wait().await`. Both operate at spawn granularity, so they cost nothing per message.

- **One channel per role, at the edge.** Use `mpsc` for a work queue. Use `oneshot` for a request and its reply. Use `watch` for current state that a task reads before it acts. Use `broadcast` for fan-out, and handle `Lagged` explicitly. These channels take a lock or a semaphore per operation, and a `oneshot` allocates per request. On the hot path, use the ring or queue from "Share without a lock". Read a `watch` value once per batch, not per message.

- **A queue and a thread at the boundary.** Push from async producers into a lock-free queue. Drain the queue on one OS thread. Create that thread with `thread::Builder::new().name(..)` and pin it to a core. Drain everything that is ready, process the batch, then check the stop flag. Send a reply through a `oneshot` that you find by request id at the edge. On the hot path, correlate by id in a preallocated slab, not in a map that allocates. Name every long-lived OS thread, so it appears under its own name in a profile. Store the `JoinHandle` the builder returns, and join it at shutdown. Do not call `block_on` inside async code. Do not share a mutex between the edge and the hot path.

  ```rust
  let drainer = thread::Builder::new()
      .name(format!("drain-{queue_name}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, stop)
      })?;
  ```

- **Async in a trait is a dispatch decision.** In an engine, define the seam over sync methods, such as `PollEvents` or `TryDequeue<T>`, with a no-op impl for tests. Keep the engine's async code in free functions and inherent `async fn`. In a service with a pluggable backend, such as a store or a transport behind a `Box<dyn Trait>`, an async trait is the right boundary. Write a native `async fn` in the trait. Add `#[trait_variant::make(Send)]` when a spawn needs the future to be `Send`. Add `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` when a caller needs a trait object (see [CRATES.md](CRATES.md)). Use `#[async_trait]` only in a crate that already uses it.

- **`Send` at the spawn boundary.** `tokio::spawn` needs a `Send + 'static` future. `Rc`, `RefCell`, `Cell`, a raw pointer, a `MutexGuard` held across an `.await`, and `dyn Trait` without `+ Send` make a future `!Send`. Fix the type, not the spawn. Use `Arc` for `Rc`, and an atomic or a `parking_lot::Mutex` for `RefCell`. Write `dyn Trait + Send + Sync` in a shared box. Drop the guard before the `.await`. Use `spawn_local` on a `LocalSet` only for a value that must stay on one thread. Pin a type's `Send` with `const _: fn() = || { fn assert_send<T: Send>() {} assert_send::<Worker>() };`, so a regression fails the build. Do not write `unsafe impl Send`.

- **Locks and `.await`.** When a lock survives "Share without a lock", do not hold its guard across an `.await`. Drop the guard before the await, or move the work into a sync fn. Use `parking_lot` for a short critical section at the edge. Alias it with `use parking_lot::Mutex as SyncMutex;` when a tokio lock is in scope. Do not use `tokio::sync::Mutex`, `Notify`, or any lock on the hot path.

- **Use the named combinators, at the edge.** Use `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered`. Use `tokio::select!` and `tokio::join!`. Do not use `futures::select!`. Do not call `.boxed()` per message. These combinators allocate, so keep them off the hot path. Use `Framed` with a `Decoder` at the wire edge (see [CRATES.md](CRATES.md)).
