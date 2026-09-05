//! The Rust blocks in `RUNTIME.md`, each inside the scaffolding it needs to compile.
//!
//! 1. Each block is a verbatim excerpt of this file. `evals/snippets.py` checks that.
//! 2. The items around a block are stubs: the smallest types and functions that let it compile
//!    and pass the check in `LINTS.md`.
//! 3. `#[rustfmt::skip]` keeps a block's one-line forms as `RUNTIME.md` prints them.

use std::collections::VecDeque;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tokio_util::sync::CancellationToken;

/// A unit of work the drain loop processes.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct JobId(pub u64);

/// Waits `backoff` between attempts until `connected` is set or the token is cancelled.
/// The cancel arm comes first and `biased;` makes it the first arm polled.
pub async fn reconnect(cancel: CancellationToken, backoff: Duration, connected: Arc<AtomicBool>) {
    loop {
        tokio::select! {
            biased;
            () = cancel.cancelled() => return,
            () = tokio::time::sleep(backoff) => {}
        }
        if connected.load(Ordering::Relaxed) {
            return;
        }
    }
}

/// Drains `queue` in batches until the token is cancelled. The token is bridged to an atomic
/// once, in one watcher task, so the loop reads a flag and never touches the token. The loop
/// stands for the body of the drain thread.
///
/// # Panics
///
/// When the watcher task panics, which it cannot: it only awaits the token and stores a flag.
#[rustfmt::skip]
pub async fn drain_until_cancelled(cancel: CancellationToken, mut queue: VecDeque<JobId>) {
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
    watcher.await.expect("the watcher only awaits the token");
}

/// Processes everything that is ready. A stand-in for the batch body.
fn drain_batch(queue: &mut VecDeque<JobId>) {
    queue.clear();
}

/// Drain-thread settings.
#[derive(Copy, Clone, Debug, Default)]
pub struct DrainConfig {
    /// The core to pin the thread to, when the deployment pins.
    pub core_id: Option<core_affinity::CoreId>,
}

/// Starts the named, pinned drain thread. The caller stores the handle and joins it at shutdown.
///
/// # Errors
///
/// The OS refused to create the thread.
#[rustfmt::skip]
pub fn spawn_drainer(
    queue_name: &str,
    config: DrainConfig,
    queue: rtrb::Consumer<JobId>,
    stop: Arc<AtomicBool>,
) -> io::Result<JoinHandle<()>> {
    let drainer = thread::Builder::new()
        .name(format!("drain-{queue_name}"))
        .spawn(move || {
            if let Some(core) = config.core_id { core_affinity::set_for_current(core); }
            drain(queue, stop);
        })?;
    Ok(drainer)
}

/// Pops everything that is ready, then checks the flag, until the flag is set.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the drain thread owns its stop flag for its whole life, as the excerpt passes it"
)]
fn drain(mut queue: rtrb::Consumer<JobId>, stop: Arc<AtomicBool>) {
    loop {
        while queue.pop().is_ok() {}
        if stop.load(Ordering::Relaxed) {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BACKOFF: Duration = Duration::from_secs(1);

    #[tokio::test(start_paused = true)]
    async fn test_reconnect_returns_on_cancel_before_the_backoff_elapses() {
        let cancel = CancellationToken::new();
        let task = tokio::spawn(reconnect(
            cancel.clone(),
            BACKOFF,
            Arc::new(AtomicBool::new(false)),
        ));
        cancel.cancel();
        task.await.expect("reconnect does not panic");
    }

    #[tokio::test(start_paused = true)]
    async fn test_reconnect_returns_once_connected_after_one_backoff() {
        let connected = Arc::new(AtomicBool::new(true));
        let started = tokio::time::Instant::now();
        reconnect(CancellationToken::new(), BACKOFF, connected).await;
        // one sleep of BACKOFF, then the flag is seen
        assert_eq!(started.elapsed(), BACKOFF);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_drain_stops_once_the_watcher_sets_the_flag() {
        let cancel = CancellationToken::new();
        cancel.cancel();
        drain_until_cancelled(cancel, VecDeque::from([JobId(1), JobId(2)])).await;
    }

    #[test]
    fn test_spawn_drainer_names_the_thread_and_joins() -> anyhow::Result<()> {
        let (mut producer, consumer) = rtrb::RingBuffer::new(4);
        producer
            .push(JobId(1))
            .map_err(|error| anyhow::anyhow!("{error}"))?;
        let stop = Arc::new(AtomicBool::new(false));
        let drainer = spawn_drainer(
            "orders",
            DrainConfig::default(),
            consumer,
            Arc::clone(&stop),
        )?;
        assert_eq!(drainer.thread().name(), Some("drain-orders"));
        stop.store(true, Ordering::Relaxed);
        drainer
            .join()
            .map_err(|_| anyhow::anyhow!("the drain thread panicked"))?;
        Ok(())
    }
}
