//! Contextual admission: a request is admitted under a policy, and the policy can change.
//!
//! 1. `Limits::admit` is the authority. Only it builds an `Admitted`, so every `Admitted` records a real check.
//! 2. `Admitted` records the policy it was checked against. It does not prove the request passes a later policy.
//! 3. The consumer decides what a policy change means for earlier admissions, and says so in code.

/// A request to run a prompt of `tokens` tokens.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Request {
    /// The prompt length.
    pub tokens: u32,
}

/// Identifies one version of the limits. A new version gets a new id.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PolicyId(pub u32);

/// The admission policy in force.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Limits {
    id: PolicyId,
    max_tokens: u32,
}

/// Why a request was not admitted.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// The prompt is longer than the policy allows.
    TooManyTokens {
        /// The prompt length.
        tokens: u32,
        /// The policy's limit.
        max: u32,
    },
}

/// A request that `Limits::admit` accepted, and the policy that accepted it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Admitted {
    request: Request,
    admitted_under: PolicyId,
}

impl Limits {
    /// A policy that admits prompts up to `max_tokens`.
    #[must_use]
    pub const fn new(id: PolicyId, max_tokens: u32) -> Self {
        Self { id, max_tokens }
    }

    /// The policy's id.
    #[must_use]
    pub const fn id(self) -> PolicyId {
        self.id
    }

    /// Admits `request` under this policy, or says why not.
    ///
    /// # Errors
    ///
    /// A prompt longer than `max_tokens` is rejected.
    pub const fn admit(self, request: Request) -> Result<Admitted, RejectReason> {
        if request.tokens > self.max_tokens {
            return Err(RejectReason::TooManyTokens {
                tokens: request.tokens,
                max: self.max_tokens,
            });
        }
        Ok(Admitted {
            request,
            admitted_under: self.id,
        })
    }
}

impl Admitted {
    /// The admitted request.
    #[must_use]
    pub const fn request(self) -> Request {
        self.request
    }

    /// Whether this admission was checked against `limits`. False after a policy change,
    /// even when the request would pass the new limits.
    #[must_use]
    pub const fn is_under(self, limits: Limits) -> bool {
        self.admitted_under.0 == limits.id.0
    }
}

/// What a policy change does to requests admitted under the old policy.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum OnPolicyChange {
    /// Earlier admissions stay queued. The old policy admitted them, and that admission stands.
    KeepEarlier,
    /// Earlier admissions are checked again. A request the new policy rejects leaves the queue.
    Readmit,
}

/// A queue of admitted requests and the policy that admits new ones.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Queue {
    limits: Limits,
    queued: Vec<Admitted>,
}

impl Queue {
    /// An empty queue under `limits`.
    #[must_use]
    pub const fn new(limits: Limits) -> Self {
        Self {
            limits,
            queued: Vec::new(),
        }
    }

    /// Admits `request` under the current policy and queues it.
    ///
    /// # Errors
    ///
    /// The policy's rejection is returned unchanged.
    pub fn submit(&mut self, request: Request) -> Result<(), RejectReason> {
        let admitted = self.limits.admit(request)?;
        self.queued.push(admitted);
        Ok(())
    }

    /// Replaces the policy. What happens to earlier admissions is `on_change`, chosen by the caller.
    /// Returns the requests that left the queue.
    pub fn set_limits(&mut self, limits: Limits, on_change: OnPolicyChange) -> Vec<Request> {
        self.limits = limits;
        match on_change {
            OnPolicyChange::KeepEarlier => Vec::new(),
            OnPolicyChange::Readmit => {
                let (kept, dropped): (Vec<_>, Vec<_>) = self
                    .queued
                    .iter()
                    .partition(|admitted| limits.admit(admitted.request()).is_ok());
                self.queued = kept;
                dropped.into_iter().map(Admitted::request).collect()
            }
        }
    }

    /// The queued admissions, in submission order.
    #[must_use]
    pub fn queued(&self) -> &[Admitted] {
        &self.queued
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const V1: Limits = Limits::new(PolicyId(1), 4096);
    const V2: Limits = Limits::new(PolicyId(2), 2048);

    #[test]
    fn test_admission_records_the_policy_that_checked_it() -> anyhow::Result<()> {
        let admitted = V1
            .admit(Request { tokens: 3000 })
            .map_err(|reason| anyhow::anyhow!("{reason:?}"))?;
        assert!(admitted.is_under(V1));
        // the request would fail V2's 2048 limit, and the admission says nothing about V2
        assert!(!admitted.is_under(V2));
        Ok(())
    }

    #[test]
    fn test_keep_earlier_serves_work_admitted_under_the_old_policy() -> anyhow::Result<()> {
        let mut queue = Queue::new(V1);
        queue
            .submit(Request { tokens: 3000 })
            .map_err(|reason| anyhow::anyhow!("{reason:?}"))?;
        let dropped = queue.set_limits(V2, OnPolicyChange::KeepEarlier);
        assert!(
            dropped.is_empty(),
            "nothing leaves the queue, dropped {dropped:?}"
        );
        assert_eq!(queue.queued().len(), 1);
        // a new request of the same size is checked against V2 and rejected
        assert_eq!(
            queue.submit(Request { tokens: 3000 }),
            Err(RejectReason::TooManyTokens {
                tokens: 3000,
                max: 2048
            })
        );
        Ok(())
    }

    #[test]
    fn test_readmit_returns_the_requests_the_new_policy_rejects() -> anyhow::Result<()> {
        let mut queue = Queue::new(V1);
        queue
            .submit(Request { tokens: 3000 })
            .map_err(|reason| anyhow::anyhow!("{reason:?}"))?;
        queue
            .submit(Request { tokens: 100 })
            .map_err(|reason| anyhow::anyhow!("{reason:?}"))?;
        let dropped = queue.set_limits(V2, OnPolicyChange::Readmit);
        assert_eq!(dropped, vec![Request { tokens: 3000 }]);
        assert_eq!(queue.queued().len(), 1);
        Ok(())
    }
}
