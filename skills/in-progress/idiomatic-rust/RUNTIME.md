# Runtime

Choose concurrency from the operations and their requirements. These are contextual defaults except where a correctness or hot-path constraint is explicit.

## Find the hot path

Identify hot paths from latency/throughput requirements and measurements, not from the application's category. A web service, CLI decoder, batch job, or inference scheduler can each have one. Measure allocations, blocking, I/O, queueing, and dispatch where they affect that budget. Cold paths have no universal thread, channel, or allocation prescription.

**Strong preference: avoid locks and mutexes on hot paths.** First consider exclusive ownership, partitioned state, immutable snapshots, or appropriate atomic/lock-free operations. These alternatives must preserve the whole operation's correctness. A collection called concurrent, or a channel, may use internal locks; inspect the actual implementation and operation.

Allow a hot-path lock only when no viable alternative preserves correctness. Hold it for only a few lines of necessary shared-state access. Keep preparation, allocation, I/O, and callbacks outside the critical section; never hold the guard across an await. Explain locally why locking is necessary, and assess contention and latency. A short critical section does not bound time spent waiting to acquire the lock. This exception also governs a lock shared with a cold path.

Choose a suitable mutex from project requirements, including poisoning, fairness, runtime blocking, and dependency policy. `std::sync::Mutex` and `parking_lot::Mutex` are both legitimate choices. Outside hot paths, an async mutex can be appropriate when an operation must retain exclusive access over I/O; account for serialization, cancellation, and deadlock. Prefer a lexical scope or a synchronous helper to release ordinary guards before awaiting.

## Ownership and communication

- Prefer a single owner when that naturally serializes state changes. A decoder library can borrow a buffer; an inference scheduler can own admission state; a small CLI may need no concurrency at all.
- An `mpsc` is a useful default for an async work queue. `oneshot` suits one reply, `watch` suits current state, and `broadcast` suits fan-out if consumers handle lag. Establish capacity, backpressure, closure, and draining semantics. Existing channels or concurrent structures are valid when they meet these requirements.
- Use snapshots when readers need a consistent version while a writer replaces it. Use a bounded SPSC ring for two dedicated threads only when that ownership and capacity fit. Multiple producers require a suitable protocol. Wait-free operations bound their own steps; lock-free operations guarantee system-wide progress, not each caller's latency. Neither alone proves a wall-clock deadline.
- A dedicated thread, core affinity, batching, and preallocation can help measured workloads. They are options, not the required boundary for every async application. Own thread and task handles so shutdown accounts for completion, failure, and deliberately detached work. Keep blocking operations off runtime workers when they would stall other tasks.

## Cancellation

Define whether shutdown aborts, drains, or completes already admitted work. A `CancellationToken` is convenient for a task tree; channel closure or an existing shutdown mechanism can suffice. A `biased;` Tokio `select!` with cancellation first gives it polling priority when ready. Use that ordering when cancellation priority is intended; account for fairness among the remaining arms.

Check cancellation safety for the exact API before placing a future in `select!`. Losing an arm drops its future. Tokio `mpsc::Receiver::recv`, `watch::Receiver::changed`, and sleeps are cancellation-safe for their documented purposes. `read_exact`, `write_all`, or a bounded send may lose partial progress, queue position, or an owned message. Complete the operation separately or retain a pinned future across iterations; retaining it does not protect it from eventual task cancellation. Define recovery for that case. See [Tokio's cancellation-safety table](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).

A cancellation token can use internal mutexes. Check the pinned dependency before polling it per message. Where this would put a lock on the hot path, a watcher can bridge cancellation to an atomic stop flag. Own the watcher handle, account for notification delay, and choose batch size from shutdown latency. `Relaxed` is enough only if the flag carries no other data or completion guarantee.

Use `JoinSet` when task results matter, or `TaskTracker` to wait for tracked tasks. With a tracker, close spawning and then await completion. Neither the tracker nor a token defines how to preserve queued work; that is the subsystem's shutdown contract.

## Async traits and dispatch

A pluggable store or transport can reasonably expose async behavior. Native `async fn` or a method returning `impl Future` suits static dispatch. Spell out the returned future's `Send` bound when callers require it; `trait-variant` can generate variants if useful. Native async methods alone do not make a trait dyn-compatible on the evaluation toolchain.

For dynamic dispatch, use an object-safe boxed-future method, `async-trait`, or another adapter that fits the project's conventions. `dynosaur` is one option: static calls avoid returned-future boxing, while dynamic calls still box the returned future. This does not mean all static method bodies are allocation-free. Weigh per-call allocation and dispatch against plugin flexibility, code size, and implementation complexity; no particular crate is required. See [dynosaur's dispatch description](https://docs.rs/dynosaur/0.3.1/dynosaur/).

A synchronous seam suits synchronous operations. Do not force a sync seam, generics, or a dedicated thread merely because the code is latency-sensitive. Check the generated runtime code and measure the path that matters.

## Send belongs to the value being moved

`tokio::spawn` requires a `Send + 'static` future and a compatible output. `Cell<T>` and `RefCell<T>` are `Send` when `T: Send`, but are not `Sync`. Owning one in a task can be fine. Sharing `&RefCell<T>` across an await, or using `Arc<RefCell<T>>` across threads, does not satisfy the required bounds. `Arc` does not make its contents thread-safe. `Rc`, non-Send guards, or non-Send trait objects retained across an await can make the future non-Send.

A bound assertion on a stored type proves only that type's bound. Its methods can create non-Send locals or borrow a non-Sync field. Assert the actual returned future when that is the guarantee needed:

```rust
use std::future::Future;

struct Worker;
impl Worker {
    async fn run(&self) {}
}

fn assert_send<T: Send>(_: T) {}
fn assert_spawnable<F>(_: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{}

let worker = Worker;
assert_send(worker.run()); // Checks this method's future, not just Worker.
assert_spawnable(async move { worker.run().await });
```

The following regression probe must fail: the owned worker is Send, but its method retains a shared borrow of a non-Sync field across suspension.

```compile_fail,E0277
use std::{cell::Cell, future::pending};
struct Worker(Cell<u32>);
impl Worker {
    async fn run(&self) -> u32 {
        pending::<()>().await;
        self.0.get()
    }
}
fn assert_type_send<T: Send>() {}
fn assert_send<T: Send>(_: T) {}
assert_type_send::<Worker>();
let worker = Worker(Cell::new(0));
assert_send(worker.run());
```

Use a local executor when thread confinement is intended. Choose exclusive ownership, atomics, or a suitable mutex for actual shared mutation. Do not add `unsafe impl Send` or `Sync` to silence a spawn error; an unsafe implementation requires an independently justified safety contract.

For atomics, establish the synchronization requirement first. `Relaxed` suits independent counters and flags with no publication obligation. Release/acquire can publish other writes when the load observes the corresponding store. Stronger orderings may simplify a correct algorithm; weaker ordering is not automatically an optimization worth making.
