# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Two regimes.** The **edge** is the async side: tokio tasks that do I/O and handle thousands of events per second. The edge can afford a short lock or one allocation per event. The **hot path** is a loop with a per-message budget in microseconds: a pinned worker thread, a ring-buffer consumer, a decoder in a tight loop. On the hot path, a lock, a heap allocation, or a syscall per message is a defect. A path is hot because a requirement and a measurement say so: a latency budget, a deadline per message, a profile that shows the loop. The category of the program does not decide it. A web service can hold one hot loop, and an engine can be all edge. Every rule below applies to the edge unless it says hot path. Measure before you move a rule across that line.

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

- **Share without a lock.** On the hot path there is no lock (see "No lock on the hot path"). Pick the sharing structure in this order. Give the state one owner and send it messages. Partition the state so each thread owns a shard. Publish an immutable snapshot with `arc_swap::ArcSwap`, which many readers load and one writer replaces. Use an atomic for a flag, a counter, or one small value: `AtomicBool`, `AtomicU64`, `AtomicUsize`. Use a wait-free or lock-free structure. A wait-free operation completes in a bounded number of steps whatever the other threads do, and a lock-free operation guarantees that some thread makes progress. An SPSC ring such as `rtrb` between two dedicated threads is wait-free on push and pop. `crossbeam::queue::ArrayQueue` (bounded) and `SegQueue` (unbounded) are lock-free when several threads produce. Do not assume a channel or a concurrent map is lock-free inside. `crossbeam-channel` and `flume` take a lock or park a thread on some operations, and `dashmap` shards a lock. Read the crate's documentation for the operation you call. A structure that can lock or park stays off the hot path. At the edge, a short lock is a normal choice, and the order above is a preference, not a rule.

- **No lock on the hot path.** A hot path takes no mutex, no async mutex, no `Notify`, and no structure that locks or parks inside, whatever the alternative costs. A blocked thread there misses its deadline, and a few lines inside a guard do not bound the wait: contention does. When one operation must read and write several fields together, give those fields one owner thread and send it the operation. Or pack the fields into one atomic and update it with a `compare_exchange` loop. Or build the new state and publish it as a snapshot through `ArcSwap`. A lock at the edge that guards state the hot path reads is the same defect from the other side. The boundary is a queue or a snapshot, not a shared mutex. For the edge, choose the mutex from the project's requirements. `std::sync::Mutex` adds no dependency and poisons on a panic. `parking_lot::Mutex` is the choice when the project already uses it, or needs the smaller guard and no poisoning. Do not add a crate for one lock.

- **Atomic orderings.** Use `Relaxed` for a flag or a counter that carries no other data. When the atomic publishes data written before the store, use `Release` on the store and `Acquire` on the load. Use `SeqCst` only with a comment that names the ordering between two atomics that the code needs. Do not write `SeqCst` by default.

- **The spawn shape.** Write `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`. State the exit conditions in the doc comment. Keep each `select!` arm to one line. Move a longer arm body into a named `async fn`.

- **Own every handle.** Store a `JoinHandle` or await it. Do not drop it. Use `JoinSet` when you read the results. Use `TaskTracker` when you do not. At shutdown, call `tracker.close()` and then `tracker.wait().await`. Both operate at spawn granularity, so they cost nothing per message.

- **One channel per role, at the edge.** Use `mpsc` for a work queue. Use `oneshot` for a request and its reply. Use `watch` for current state that a task reads before it acts. Use `broadcast` for fan-out, and handle `Lagged` explicitly. At the edge, a work queue is an `mpsc`, not a lock-free queue plus a `Notify`. These channels take a lock or a semaphore per operation, and a `oneshot` allocates per request. On the hot path, use the ring or queue from "Share without a lock". Read a `watch` value once per batch, not per message.

- **A queue and a thread at the boundary.** Push from async producers into a lock-free queue. Drain the queue on one OS thread. Create that thread with `thread::Builder::new().name(..)` and pin it to a core. Drain everything that is ready, process the batch, then check the stop flag. Send a reply through a `oneshot` that you find by request id at the edge. On the hot path, correlate by id in a preallocated slab, not in a map that allocates. Name every long-lived OS thread, so it appears under its own name in a profile. Store the `JoinHandle` the builder returns, and join it at shutdown. Do not call `block_on` inside async code. A mutex shared with the edge puts the edge's hold time on the hot path's deadline, so the boundary is a queue, not a lock.

  ```rust
  let drainer = thread::Builder::new()
      .name(format!("drain-{queue_name}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, stop)
      })?;
  ```

- **Async in a trait is a dispatch decision.** In an engine, define the seam over sync methods, such as `PollEvents` or `TryDequeue<T>`, with a no-op impl for tests. Keep the engine's async code in free functions and inherent `async fn`. In a service with a pluggable backend, an async trait is the right boundary. A store or a transport behind a `Box<dyn Trait>` is the usual case. Write a native `async fn` in the trait. Add `#[trait_variant::make(Send)]` when a spawn needs the future to be `Send`. Add `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` when a caller needs a trait object (see [CRATES.md](CRATES.md)). Static dispatch through the native `async fn` returns the future unboxed. Dynamic dispatch through the trait object boxes each returned future, with `dynosaur` as with `#[async_trait]`. The difference between the two is one allocation per call on the dynamic path, so measure it where it matters. Keep `#[async_trait]` in a crate that already uses it; it boxes on every call, static or dynamic, and that is its one cost.

- **`Send` at the spawn boundary.** `tokio::spawn` needs a `Send + 'static` future. A type is `Send` when every field is `Send`, and `Sync` when a shared `&T` is `Send`. `Rc`, a raw pointer, a `MutexGuard` held across an `.await`, and `dyn Trait` without `+ Send` make a future `!Send`. `RefCell` and `Cell` are `Send` but not `Sync`, so an `Arc<RefCell<T>>` or a `&RefCell<T>` held across an `.await` makes the future `!Send`. Fix the type, not the spawn. Use `Arc` for `Rc`. Use an atomic or a `parking_lot::Mutex` for a `RefCell` or a `Cell` that is shared. Write `dyn Trait + Send + Sync` in a shared box. Drop the guard before the `.await`. Use `spawn_local` on a `LocalSet` only for a value that must stay on one thread. A `Send` assertion on the stored type proves nothing about the future its methods return. A guard or an `Rc` that lives across an `.await` inside `run` makes the future of `run()` `!Send` while `Worker` stays `Send`. When the future is what `spawn` needs, assert the future: `const _: () = { fn check(worker: &Worker) { fn assert_send<T: Send>(_: T) {} assert_send(worker.run()); } };`. Assert the type with `const _: fn() = || { fn assert_send<T: Send>() {} assert_send::<Worker>() };` only when the type itself is what crosses threads. Either one fails the build on a regression. Do not write `unsafe impl Send` or `unsafe impl Sync`.

- **Locks and `.await`.** At the edge, a sync mutex guards data for a few lines with no `.await` inside. `std::sync::Mutex` or `parking_lot::Mutex` is the project's choice. Do not hold its guard across an `.await`: drop the guard first, or move the work into a sync fn. Use `tokio::sync::Mutex` only when the guard must live across an `.await`, such as exclusive use of one connection. It is slower, and it is not for data. Alias the sync mutex with `use parking_lot::Mutex as SyncMutex;` when a tokio lock is in scope. On the hot path there is no lock, no `Notify`, and no async mutex (see "No lock on the hot path").

- **Use the named combinators, at the edge.** Use `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered`. Use `tokio::select!` and `tokio::join!`. Do not use `futures::select!`. Do not call `.boxed()` per message. These combinators allocate, so keep them off the hot path. Use `Framed` with a `Decoder` at the wire edge (see [CRATES.md](CRATES.md)).
