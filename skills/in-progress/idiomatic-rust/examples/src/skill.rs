//! The Rust blocks in `SKILL.md`, each inside the scaffolding it needs to compile.
//!
//! 1. Each block is a verbatim excerpt of this file. `evals/snippets.py` checks that.
//! 2. The items around a block are stubs: the smallest types and functions that let it compile
//!    and pass the check in `LINTS.md`.
//! 3. `#[rustfmt::skip]` keeps a block's one-line forms as `SKILL.md` prints them.
//! 4. The excerpts carry no doc comments of their own; the prose beside them in `SKILL.md` does.
#![expect(
    clippy::missing_errors_doc,
    reason = "the excerpts omit doc comments; SKILL.md's prose carries the rule beside each one"
)]

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

/// A system-generated job identifier. Every `u64` is valid, so the derives build it directly.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)] // std
#[derive(Serialize, Deserialize)] // serde
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable)] // zerocopy
#[repr(transparent)] // wire layout is the inner u64
pub struct JobId(u64);

/// A worker in the pool.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct WorkerId(pub u32);

/// How many times a job was retried.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Retries(pub u32);

/// The priority a queue is keyed by.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Priority(pub u8);

/// No job has the id.
#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
#[error("job {0:?} is not known")]
pub struct JobNotFound(pub JobId);

#[derive(Debug, Error, Copy, Clone, PartialEq, Eq)]
pub enum SchedulerError {
    #[error(transparent)]
    JobNotFound(#[from] JobNotFound),
    #[error("worker {0:?} is offline")]
    WorkerOffline(WorkerId),
}

/// Why a cancel request was not honoured.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// No job has the id.
    UnknownJob,
}

/// A queued job: the worker it runs on and its retry count.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Job {
    /// The worker the job is assigned to.
    pub worker: WorkerId,
    /// How many times the job was retried.
    pub attempts: Retries,
}

/// A worker and its queues by priority.
#[derive(Debug, Default)]
pub struct Worker {
    queues: HashMap<Priority, HashMap<JobId, Job>>,
}

/// The scheduler the `cancel` and `remove_queued` excerpts belong to.
#[derive(Debug, Default)]
pub struct Scheduler {
    jobs: HashMap<JobId, Job>,
    workers: HashMap<WorkerId, Worker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
#[must_use = "a rejection must reach the caller"]
pub enum CancelOutcome { Cancelled { attempts: Retries }, Rejected(RejectReason) }

impl Scheduler {
    pub fn cancel(&mut self, id: JobId) -> Result<CancelOutcome, SchedulerError> {
        let Some(job) = self.jobs.get_mut(&id) else {
            return Ok(CancelOutcome::Rejected(RejectReason::UnknownJob));
        };
        // An unknown worker is a broken invariant, not an expected "no".
        if !self.workers.contains_key(&job.worker) {
            return Err(SchedulerError::WorkerOffline(job.worker));
        }
        let attempts = job.attempts;
        self.jobs.remove(&id);
        Ok(CancelOutcome::Cancelled { attempts })
    }

    /// Registers `worker` with no queues.
    pub fn add_worker(&mut self, worker: WorkerId) {
        self.workers.entry(worker).or_default();
    }

    /// Queues `id` on `job.worker` at `priority`. The worker must be registered.
    pub fn enqueue(&mut self, id: JobId, job: Job, priority: Priority) {
        self.jobs.insert(id, job);
        if let Some(worker) = self.workers.get_mut(&job.worker) {
            worker.queues.entry(priority).or_default().insert(id, job);
        }
    }

    /// Removes `job_id` from `worker_id`'s queue at `priority`. False when either is unknown.
    #[rustfmt::skip]
    pub fn remove_queued(&mut self, worker_id: WorkerId, priority: Priority, job_id: JobId) -> bool {
        let removed = 'queue: {
            let Some(worker) = self.workers.get_mut(&worker_id) else { break 'queue false };
            let Some(queue) = worker.queues.get_mut(&priority) else { break 'queue false };
            queue.remove(&job_id).is_some()
        };
        if removed {
            self.jobs.remove(&job_id);
        }
        removed
    }
}

/// Builds the tick callback and the sorted job list. Each rebinding freezes what it names.
#[rustfmt::skip]
pub fn prepare(fetch_jobs: impl FnOnce() -> Vec<JobId>) -> (impl Fn() -> usize, Vec<JobId>) {
    let counter = Arc::new(AtomicUsize::new(0));
    let on_tick = {
        let counter = Arc::clone(&counter);
        move || counter.fetch_add(1, Ordering::Relaxed)
    };
    let jobs = { let mut jobs = fetch_jobs(); jobs.sort_unstable(); jobs };
    (on_tick, jobs)
}

/// The module-local `Result` alias excerpt. Its own module, so the alias does not shadow the
/// two-parameter `Result` the rest of this file uses.
pub mod decode {
    use thiserror::Error;

    /// A decoded header.
    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub struct Header {
        /// The protocol version.
        pub version: u8,
    }

    impl Header {
        /// The one version this decoder reads.
        pub const VERSION: u8 = 1;
    }

    #[derive(Debug, Error)]
    pub enum DecodeError {
        #[error("invalid header")]
        InvalidHeader,
        #[error("truncated input")]
        TruncatedInput,
    }

    pub type Result<T> = std::result::Result<T, DecodeError>;

    pub fn header(bytes: &[u8]) -> Result<Header> {
        let Some(&version) = bytes.first() else {
            return Err(DecodeError::TruncatedInput);
        };
        if version != Header::VERSION {
            return Err(DecodeError::InvalidHeader);
        }
        Ok(Header { version })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKER: WorkerId = WorkerId(1);
    const HIGH: Priority = Priority(2);

    fn scheduler_with_one_job() -> Scheduler {
        let mut scheduler = Scheduler::default();
        scheduler.add_worker(WORKER);
        scheduler.enqueue(
            JobId(7),
            Job {
                worker: WORKER,
                attempts: Retries(3),
            },
            HIGH,
        );
        scheduler
    }

    #[test]
    fn test_cancel_of_an_unknown_job_is_a_rejection_not_an_error() -> anyhow::Result<()> {
        let mut scheduler = scheduler_with_one_job();
        assert_eq!(
            scheduler.cancel(JobId(8))?,
            CancelOutcome::Rejected(RejectReason::UnknownJob)
        );
        // the known job cancels with its retry count
        assert_eq!(
            scheduler.cancel(JobId(7))?,
            CancelOutcome::Cancelled {
                attempts: Retries(3)
            }
        );
        Ok(())
    }

    #[test]
    fn test_cancel_with_an_offline_worker_is_an_error() {
        let mut scheduler = Scheduler::default();
        scheduler.enqueue(
            JobId(7),
            Job {
                worker: WORKER,
                attempts: Retries(0),
            },
            HIGH,
        );
        assert_eq!(
            scheduler.cancel(JobId(7)),
            Err(SchedulerError::WorkerOffline(WORKER))
        );
    }

    #[test]
    fn test_remove_queued_exits_the_labeled_block_on_each_miss() {
        let mut scheduler = scheduler_with_one_job();
        assert!(
            !scheduler.remove_queued(WorkerId(9), HIGH, JobId(7)),
            "unknown worker"
        );
        assert!(
            !scheduler.remove_queued(WORKER, Priority(0), JobId(7)),
            "unknown priority"
        );
        assert!(
            scheduler.remove_queued(WORKER, HIGH, JobId(7)),
            "the queued job"
        );
        assert!(
            !scheduler.remove_queued(WORKER, HIGH, JobId(7)),
            "already removed"
        );
    }

    #[test]
    fn test_prepare_sorts_the_jobs_and_counts_ticks() {
        let (on_tick, jobs) = prepare(|| vec![JobId(3), JobId(1), JobId(2)]);
        assert_eq!(jobs, vec![JobId(1), JobId(2), JobId(3)]);
        // fetch_add returns the value before the increment: 0, then 1
        assert_eq!(on_tick(), 0);
        assert_eq!(on_tick(), 1);
    }

    #[test]
    fn test_header_uses_the_module_alias() {
        assert!(matches!(
            decode::header(&[]),
            Err(decode::DecodeError::TruncatedInput)
        ));
        assert!(matches!(
            decode::header(&[9]),
            Err(decode::DecodeError::InvalidHeader)
        ));
        assert_eq!(
            decode::header(&[1]).ok(),
            Some(decode::Header { version: 1 })
        );
    }

    #[test]
    fn test_job_id_is_the_inner_u64_on_the_wire() {
        // zerocopy reads the eight bytes as the u64, little-endian on this target
        let id = JobId::read_from_bytes(&7_u64.to_ne_bytes()).expect("eight bytes are a JobId");
        assert_eq!(id, JobId(7));
    }
}
