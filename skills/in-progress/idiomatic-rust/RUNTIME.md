# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Two regimes.** The **edge** is the async side, tokio tasks that do I/O at thousands of events per second, where a short lock or one allocation per event is affordable. The **hot path** is a loop with a per-message budget in microseconds (a pinned worker, a ring-buffer consumer, a tight decoder loop), where a lock, a heap allocation, or a syscall per message is a defect. A path is hot because a requirement and a measurement say so (a latency budget, a deadline per message, a profile that shows the loop), not because of the program's category: a web service can hold one hot loop, and an engine can be all edge. Every rule below applies to the edge unless it says hot path. Measure before you move a rule across that line.

- **Cancel first, at the edge.** Pass a `CancellationToken` to every long-lived task, one per subsystem from `child_token()` at setup. `cancel.cancelled()` is the first arm of every `select!`, under a `biased;` first line, so tokio polls the arms in written order and checks the cancel arm first. Shutdown is the token, not a `broadcast` channel of `()`.

  ```rust
  tokio::select! {
      biased;
      () = cancel.cancelled() => return,
      () = tokio::time::sleep(backoff) => {}
  }
  ```

- **Only cancel-safe futures in `select!`.** `select!` drops the losing arms, and a future that has consumed input loses it. `recv`, `sleep`, `cancelled`, and `changed` are cancel safe; `read_exact`, `write_all`, and `send` on a bounded channel are not. Run those to completion outside the `select!`, or create the future once, pin it, and poll `&mut fut` across iterations.

- **Cancellation on the hot path.** `is_cancelled()` takes a mutex on every call, `cancelled()` on every poll, and `clone()`, `child_token()`, and drop take it too and can restructure the token tree; under contention with `cancel()` these stall for milliseconds. So no token is created, cloned, or dropped per request, and no `cancelled()` is polled per message. Bridge the token to an atomic once, in one watcher task, and read the atomic in the loop once per batch; `Relaxed` is enough for a flag that carries no data, and a sync OS thread reads the same atomic.

  ```rust
  let stop = Arc::new(AtomicBool::new(false));
  let watcher = tokio::spawn({
      let stop = Arc::clone(&stop);
      async move { cancel.cancelled().await; stop.store(true, Ordering::Relaxed); }
  });
  // In the hot loop, once per drained batch:
  loop {
      drain_batch(&mut queue);
      if stop.load(Ordering::Relaxed) { break; }
  }
  ```

- **Share without a lock.** Pick the sharing structure in this order. Give the state one owner and send it messages. Partition the state so each thread owns a shard. Publish an immutable snapshot with `arc_swap::ArcSwap`, which many readers load and one writer replaces. Use an atomic for a flag, a counter, or one small value. Use a wait-free or lock-free structure: an SPSC ring such as `rtrb` between two dedicated threads is wait-free on push and pop; `crossbeam::queue::ArrayQueue` (bounded) and `SegQueue` (unbounded) are lock-free with several producers. A channel or a concurrent map is not lock-free until its documentation says so for the operation you call: `crossbeam-channel` and `flume` lock or park on some operations, and `dashmap` shards a lock. A structure that can lock or park stays off the hot path. At the edge a short lock is a normal choice, and the order above is a preference.

- **No lock on the hot path.** A hot path takes no mutex, no async mutex, no `Notify`, and no structure that locks or parks inside, whatever the alternative costs: a blocked thread there misses its deadline, and a few lines inside a guard do not bound the wait, contention does. When one operation must read and write several fields together, one of three shapes replaces the lock: one owner thread that receives the operation, one atomic updated with a `compare_exchange` loop, or a new state built and published through `ArcSwap`. A lock at the edge that guards state the hot path reads is the same defect from the other side, since it puts the edge's hold time on the hot path's deadline: the boundary is a queue or a snapshot, not a shared mutex. At the edge, the mutex is the project's choice: `std::sync::Mutex` adds no dependency and poisons on a panic, `parking_lot::Mutex` when the project already uses it or needs the smaller guard and no poisoning. No crate is added for one lock.

- **Atomic orderings.** `Relaxed` for a flag or a counter that carries no other data. `Release` on the store and `Acquire` on the load when the atomic publishes data written before the store. `SeqCst` only with a comment that names the ordering between two atomics that the code needs, never by default.

- **The spawn shape.** `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`, with the exit conditions in the doc comment. Each `select!` arm is one line; a longer arm body is a named `async fn`.

- **Own every handle.** Store every `JoinHandle` or await it. `JoinSet` when you read the results, `TaskTracker` when you do not; at shutdown, `tracker.close()` then `tracker.wait().await`. Both operate at spawn granularity, so they cost nothing per message.

- **One channel per role, at the edge.** `mpsc` for a work queue, `oneshot` for a request and its reply, `watch` for current state a task reads before it acts, `broadcast` for fan-out with `Lagged` handled explicitly. At the edge a work queue is an `mpsc`, not a lock-free queue plus a `Notify`. These channels take a lock or a semaphore per operation, and a `oneshot` allocates per request. The hot path uses the ring or queue from "Share without a lock", and reads a `watch` value once per batch.

- **A queue and a thread at the boundary.** Async producers push into a lock-free queue, and one OS thread drains it: created with `thread::Builder::new().name(..)` and pinned to a core, so it appears under its own name in a profile, with its `JoinHandle` stored and joined at shutdown. The thread drains everything that is ready, processes the batch, then checks the stop flag. A reply goes through a `oneshot` found by request id at the edge; on the hot path the id is correlated in a preallocated slab, not in a map that allocates. No `block_on` inside async code.

  ```rust
  let drainer = thread::Builder::new()
      .name(format!("drain-{queue_name}"))
      .spawn(move || {
          if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
          drain(queue, stop);
      })?;
  ```

- **Async in a trait is a dispatch decision.** In an engine, define the seam over sync methods, such as `PollEvents` or `TryDequeue<T>`, with a no-op impl for tests, and keep the async code in free functions and inherent `async fn`. In a service with a pluggable backend, a store or a transport behind a `Box<dyn Trait>`, an async trait is the right boundary: a native `async fn` in the trait, `#[trait_variant::make(Send)]` when a spawn needs the future to be `Send`, and `#[dynosaur::dynosaur(DynStore = dyn(box) Store)]` when a caller needs a trait object. Static dispatch returns the future unboxed and the trait object boxes each one, a difference of one allocation per call on the dynamic path (see [CRATES.md](CRATES.md)); measure it where it matters. A crate that already uses `#[async_trait]` keeps it.

- **`Send` at the spawn boundary.** `tokio::spawn` needs a `Send + 'static` future. When the future is `!Send` (an `Rc`, a raw pointer, a `MutexGuard` or a `&RefCell<T>` held across an `.await`, a `dyn Trait` without `+ Send`), fix the type, not the spawn: `Arc` for `Rc`, an atomic or a `parking_lot::Mutex` for a shared `RefCell` or `Cell`, `dyn Trait + Send + Sync` in a shared box, the guard dropped before the `.await`. `spawn_local` on a `LocalSet` is for a value that must stay on one thread. A `Send` assertion on the stored type proves nothing about the future its methods return: a guard or an `Rc` across an `.await` inside `run` makes the future of `run()` `!Send` while `Worker` stays `Send`. When the future is what `spawn` needs, assert the future, `const _: () = { fn check(worker: &Worker) { fn assert_send<T: Send>(_: T) {} assert_send(worker.run()); } };`; assert the type, `const _: fn() = || { fn assert_send<T: Send>() {} assert_send::<Worker>() };`, only when the type itself crosses threads. Either fails the build on a regression. There is no `unsafe impl Send` or `unsafe impl Sync`.

- **Locks and `.await`.** At the edge, a sync mutex (`std::sync::Mutex` or `parking_lot::Mutex`, the project's choice) guards data for a few lines with no `.await` inside: drop the guard before the `.await`, or move the work into a sync fn. `tokio::sync::Mutex` is for a guard that must live across an `.await`, such as exclusive use of one connection; it is slower and it is not for data. When a tokio lock is in scope, alias the sync one: `use parking_lot::Mutex as SyncMutex;`. On the hot path there is no lock of either kind (see "No lock on the hot path").

- **Use the named combinators, at the edge.** `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered`; `tokio::select!` and `tokio::join!`, not `futures::select!`. These allocate, so they stay off the hot path, and nothing calls `.boxed()` per message. `Framed` with a `Decoder` sits at the wire edge (see [CRATES.md](CRATES.md)).
