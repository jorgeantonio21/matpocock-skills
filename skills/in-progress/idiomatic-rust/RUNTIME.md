# Runtime

Rules for async tasks, OS threads, and the boundary between them. Follow them together with [SKILL.md](SKILL.md).

- **Two regimes.** Identify the hot path from latency and throughput requirements and measurements. A library, web service, CLI tool, batch job, or inference server can have one. Avoid locks and mutexes there, subject only to the correctness exception in "Share without a lock". Measure allocation, I/O, and queueing costs against the same budget. The edge is the surrounding work; its concurrency choices depend on the project. The edge rules below are defaults for the operations they describe, not a required architecture.

- **Cancel first, at the edge.** When long-lived Tokio tasks require prompt cancellation, pass a `CancellationToken` and prefer `biased;` with cancellation as the first `select!` arm. Tokio then polls arms in written order, so cancellation is checked first. Account for fairness among the remaining arms and whether shutdown drains or aborts admitted work. Use child tokens for subsystem cancellation when that hierarchy fits. Keep an existing shutdown protocol, including channel closure, when it meets the contract.

  ```rust
  tokio::select! {
      biased;
      _ = cancel.cancelled() => return,
      _ = tokio::time::sleep(backoff) => {}
  }
  ```

- **Only cancel-safe futures in `select!`.** When one arm completes, `select!` drops the futures of the other arms. A future that has consumed input and not yet returned it loses that input. `recv`, `sleep`, `cancelled`, and `changed` are cancel safe. `read_exact`, `write_all`, and `send` on a bounded channel are not. Run those to completion outside the `select!`, or create the future once, pin it, and poll `&mut fut` across iterations.

- **Cancellation on the hot path.** Cancellation tokens can use internal mutexes. Check the pinned implementation before creating, cloning, dropping, or polling a token per message. Where that would introduce hot-path locking, bridge the token to an atomic in one watcher task and retain its handle. Read the flag in the loop. `Relaxed` is enough when the flag carries no other data or completion guarantee. Choose a batch size that meets shutdown latency.

  ```rust
  let stop = Arc::new(AtomicBool::new(false));
  let watcher = tokio::spawn({
      let stop = Arc::clone(&stop);
      async move {
          cancel.cancelled().await;
          stop.store(true, Ordering::Relaxed);
      }
  });
  // Keep the watcher handle for shutdown.

  // In the hot loop, once per drained batch:
  if stop.load(Ordering::Relaxed) {
      break;
  }
  ```

- **Share without a lock.** On hot paths, first consider exclusive ownership, partitioned state, immutable snapshots, or appropriate atomic/lock-free operations. A wait-free operation bounds its own steps; a lock-free operation guarantees system-wide progress, not each caller's latency. An atomic can hold a flag or counter. An SPSC ring such as `rtrb` can connect two dedicated threads; `ArrayQueue` can serve several producers. `ArcSwap` can publish snapshots. Choose channels such as `crossbeam-channel` or `flume` from their capacity and blocking contracts. A channel or concurrent collection is not necessarily lock-free; `dashmap` uses sharded locks. Allow a hot-path lock only when no viable alternative preserves correctness. Hold it for only a few lines of necessary shared-state access. Keep preparation, allocation, I/O, and callbacks outside; never hold the guard across an await. Explain locally why locking is necessary, and assess contention and latency. A short critical section does not bound acquisition time. Choose a suitable mutex, including std or `parking_lot`, from the project's requirements. Outside hot paths, use the synchronization that fits the operation.

- **Atomic orderings.** Use `Relaxed` for a flag or a counter that carries no other data. When the atomic publishes data written before the store, use `Release` on the store and `Acquire` on the load. Use `SeqCst` only with a comment that names the ordering between two atomics that the code needs. Do not write `SeqCst` by default.

- **The spawn shape.** Write `fn spawn_x(deps, cancel: CancellationToken, log: Logger) -> JoinHandle<()>`. State the exit conditions in the doc comment. Keep each `select!` arm to one line. Move a longer arm body into a named `async fn`.

- **Own every handle.** Store a `JoinHandle` or await it. Do not drop it. Use `JoinSet` when you read the results. Use `TaskTracker` when you do not. At shutdown, call `tracker.close()` and then `tracker.wait().await`. Both operate at spawn granularity, so they cost nothing per message.

- **One channel per role, at the edge.** Prefer `mpsc` for an async work queue, `oneshot` for one reply, `watch` for current state, and `broadcast` for fan-out. Handle `Lagged` explicitly. Define capacity, backpressure, closure, and draining behavior. These channels can synchronize internally, and a `oneshot` allocates per request. Check the actual operations before using them on a hot path. Apply "Share without a lock" to internal locking too. Keep an existing queue when it satisfies the same requirements.

- **A queue and a thread at the boundary.** When measured CPU work needs a dedicated thread, producers can enqueue work for that thread to drain in batches. Use `thread::Builder::new().name(..)` and consider core affinity only when measurements justify it. Correlate replies and preallocate storage when the latency budget requires it. Store the thread's handle and join it at shutdown. This pattern is optional; a library, synchronous CLI, or async service may need a different boundary. Do not call blocking work directly on runtime workers when it would stall other tasks. A mutex shared with a hot path must satisfy the same narrow exception in "Share without a lock".

  ```rust
  let drainer = thread::Builder::new()
      .name(format!("drain-{queue_name}"))
      .spawn(move || {
          if let Some(core) = config.core_id {
              core_affinity::set_for_current(core);
          }

          drain(queue, stop)
      })?;
  ```

- **Async in a trait is a dispatch decision.** Define sync methods such as `PollEvents` or `TryDequeue<T>` when the seam is synchronous. A pluggable store or transport can reasonably need an async boundary. Native `async fn` or `impl Future` suits static dispatch; require the returned future to be `Send` when callers need it. `trait-variant` can generate that variant. Dynamic dispatch can use an object-safe boxed-future method, `async-trait`, or an adapter such as `dynosaur` (see [CRATES.md](CRATES.md)). With `dynosaur`, static dispatch avoids returned-future boxing, while dynamic dispatch still boxes. Choose the needed dispatch and bounds before the crate; no particular adapter is required.

- **`Send` at the spawn boundary.** `tokio::spawn` needs a `Send + 'static` future and a suitable output. `Sync` means a shared `&T` is `Send`. An `Rc`, non-Send guard, raw pointer, or trait object without the needed bounds can make a future non-Send when retained across an await. `RefCell<T>` and `Cell<T>` are `Send` when `T: Send`, but are not `Sync`. Owning one in a task can be valid; sharing one through `Arc` does not make it thread-safe. Choose ownership, atomics, or a suitable mutex for actual shared mutation, and drop ordinary guards before awaiting. Use `spawn_local` on a `LocalSet` when thread confinement is intended. A bound assertion on `Worker` proves only the stored type, not the future returned by its methods. Assert the relevant future as well when callers require that guarantee. Do not write `unsafe impl Send` or `unsafe impl Sync` to silence a spawn error.

  ```rust
  struct Worker;

  impl Worker {
      async fn run(&self) {}
  }

  fn assert_send<T: Send>(_: T) {}

  let worker = Worker;
  assert_send(worker.run());
  ```

- **Locks and `.await`.** Release ordinary mutex guards before an await, preferably with a lexical scope or a synchronous helper. On hot paths, every lock must satisfy the correctness exception and critical-section requirements in "Share without a lock". Outside hot paths, choose std, `parking_lot`, or an async mutex from blocking, poisoning, and ownership requirements. An async mutex can suit an operation that deliberately owns a resource across I/O; account for cancellation, serialization, and deadlock.

- **Use the named combinators, at the edge.** Use `buffer_unordered(n)`, `ready_chunks(n)`, `try_next`, `take_until(cancel.cancelled())`, and `FuturesUnordered` where they express the operation. Use the runtime's `select!` and `join!` variants that meet its cancellation and fairness requirements. Check allocation and dispatch costs for the chosen combinator, rather than assuming every combinator allocates. Measure before adding per-message boxing on a hot path. Use `Framed` with a `Decoder` when it fits the protocol boundary (see [CRATES.md](CRATES.md)).
