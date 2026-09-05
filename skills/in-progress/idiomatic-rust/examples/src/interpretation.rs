//! A shared interpretation: one policy over a status, defined once on the type that owns it.
//!
//! 1. `JobStatus::is_terminal` is the one place that says which statuses end a job.
//! 2. The `match` is exhaustive. A new variant fails the build until the policy decides it.
//! 3. Consumers call the method. A `matches!` in a consumer would drift from this one.

/// Where a job is in its life.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum JobStatus {
    /// Waiting for a worker.
    Queued,
    /// On a worker.
    Running,
    /// Finished with a result.
    Succeeded,
    /// Finished with an error.
    Failed {
        /// Whether the scheduler queues the job again.
        retryable: bool,
    },
    /// Stopped by the owner.
    Cancelled,
}

impl JobStatus {
    /// Whether the job is finished for good. Every consumer that counts finished jobs asks here.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        match self {
            Self::Succeeded | Self::Cancelled | Self::Failed { retryable: false } => true,
            // A retryable failure is not the end: the scheduler queues the job again.
            Self::Queued | Self::Running | Self::Failed { retryable: true } => false,
        }
    }
}

/// How many of `statuses` are finished. A metrics consumer.
#[must_use]
pub fn finished(statuses: &[JobStatus]) -> usize {
    statuses
        .iter()
        .filter(|status| status.is_terminal())
        .count()
}

/// Whether every job is finished, so the batch can be reported. A scheduler consumer.
#[must_use]
pub fn batch_complete(statuses: &[JobStatus]) -> bool {
    statuses.iter().all(|status| status.is_terminal())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_retryable_failure_is_not_terminal() {
        let statuses = [
            JobStatus::Succeeded,
            JobStatus::Failed { retryable: true },
            JobStatus::Failed { retryable: false },
            JobStatus::Cancelled,
            JobStatus::Running,
        ];
        // succeeded, non-retryable failure, cancelled = 3 of 5
        assert_eq!(finished(&statuses), 3);
        assert!(!batch_complete(&statuses));
    }
}
