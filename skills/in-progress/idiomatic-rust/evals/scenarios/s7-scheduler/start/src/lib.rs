//! An inference request scheduler.
//!
//! 1. A request is admitted against the policy when it is submitted, and queued per model.
//! 2. `next_batch` drains a model's queue in submission order while the prompt tokens fit the budget.
//! 3. Order is submission order. Time is not an input.

use std::collections::{HashMap, VecDeque};

/// A client-assigned request identifier.
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct RequestId(pub u64);

/// The model a request runs on.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ModelId(String);

impl ModelId {
    /// A model identified by `name`.
    #[must_use]
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// The model's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

/// A request to run a prompt on a model.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Request {
    /// The client's id for the request.
    pub id: RequestId,
    /// The model to run on.
    pub model: ModelId,
    /// The prompt length.
    pub prompt_tokens: u32,
}

/// The admission limits.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Policy {
    /// The longest prompt a submission may carry.
    pub max_prompt_tokens: u32,
    /// The most requests one model's queue holds.
    pub max_queue_len: usize,
}

/// Why a submission was not queued.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RejectReason {
    /// The prompt is longer than the policy allows.
    TooManyTokens {
        /// The prompt length.
        tokens: u32,
        /// The policy's limit.
        max: u32,
    },
    /// The model's queue is at the policy's limit.
    QueueFull {
        /// The model.
        model: ModelId,
        /// The policy's limit.
        max: usize,
    },
}

/// What `submit` did.
#[must_use = "a rejection must reach the client"]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SubmitOutcome {
    /// The request is queued at `position` (zero is the front).
    Queued {
        /// The request's place in its model's queue.
        position: usize,
    },
    /// The request was not queued.
    Rejected(RejectReason),
}

/// The scheduler: one queue per model, and the policy that admits new requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scheduler {
    policy: Policy,
    queues: HashMap<ModelId, VecDeque<Request>>,
}

impl Scheduler {
    /// An empty scheduler under `policy`.
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            queues: HashMap::new(),
        }
    }

    /// The policy in force.
    #[must_use]
    pub const fn policy(&self) -> Policy {
        self.policy
    }

    /// Admits `request` under the policy and queues it behind its model's earlier requests.
    pub fn submit(&mut self, request: Request) -> SubmitOutcome {
        // The prompt limit is a property of the request alone.
        if request.prompt_tokens > self.policy.max_prompt_tokens {
            return SubmitOutcome::Rejected(RejectReason::TooManyTokens {
                tokens: request.prompt_tokens,
                max: self.policy.max_prompt_tokens,
            });
        }
        let queue = self.queues.entry(request.model.clone()).or_default();
        // The queue limit is a property of the model's queue at this moment.
        if queue.len() >= self.policy.max_queue_len {
            return SubmitOutcome::Rejected(RejectReason::QueueFull {
                model: request.model,
                max: self.policy.max_queue_len,
            });
        }
        queue.push_back(request);
        SubmitOutcome::Queued {
            position: queue.len() - 1,
        }
    }

    /// Takes requests from the front of `model`'s queue while their prompt tokens fit in `budget`.
    /// Returns them in submission order. An empty or unknown model gives an empty batch.
    pub fn next_batch(&mut self, model: &ModelId, budget: u32) -> Vec<Request> {
        let Some(queue) = self.queues.get_mut(model) else {
            return Vec::new();
        };
        let mut batch = Vec::new();
        let mut used = 0;
        while let Some(front) = queue.front()
            && used + front.prompt_tokens < budget
        {
            used += front.prompt_tokens;
            batch.extend(queue.pop_front());
        }
        batch
    }

    /// How many requests wait in `model`'s queue.
    #[must_use]
    pub fn queued_len(&self, model: &ModelId) -> usize {
        self.queues.get(model).map_or(0, VecDeque::len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const POLICY: Policy = Policy {
        max_prompt_tokens: 4096,
        max_queue_len: 2,
    };

    fn request(id: u64, prompt_tokens: u32) -> Request {
        Request {
            id: RequestId(id),
            model: ModelId::new("small"),
            prompt_tokens,
        }
    }

    #[test]
    fn test_submit_queues_behind_earlier_requests() {
        let mut scheduler = Scheduler::new(POLICY);
        assert_eq!(
            scheduler.submit(request(1, 100)),
            SubmitOutcome::Queued { position: 0 }
        );
        assert_eq!(
            scheduler.submit(request(2, 100)),
            SubmitOutcome::Queued { position: 1 }
        );
        assert_eq!(scheduler.queued_len(&ModelId::new("small")), 2);
    }

    #[test]
    fn test_submit_rejects_a_prompt_above_the_limit() {
        let mut scheduler = Scheduler::new(POLICY);
        assert_eq!(
            scheduler.submit(request(1, 4097)),
            SubmitOutcome::Rejected(RejectReason::TooManyTokens {
                tokens: 4097,
                max: 4096
            })
        );
    }

    #[test]
    fn test_submit_rejects_when_the_queue_is_full() {
        let mut scheduler = Scheduler::new(POLICY);
        let _ = scheduler.submit(request(1, 100));
        let _ = scheduler.submit(request(2, 100));
        // max_queue_len is 2, so the third request finds the queue full
        assert_eq!(
            scheduler.submit(request(3, 100)),
            SubmitOutcome::Rejected(RejectReason::QueueFull {
                model: ModelId::new("small"),
                max: 2
            })
        );
    }

    #[test]
    fn test_next_batch_stops_before_the_budget_is_exceeded() {
        let mut scheduler = Scheduler::new(POLICY);
        let _ = scheduler.submit(request(1, 1000));
        let _ = scheduler.submit(request(2, 1500));
        // 1000 fits in 2000; 1000 + 1500 = 2500 does not
        let batch = scheduler.next_batch(&ModelId::new("small"), 2000);
        assert_eq!(
            batch.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![RequestId(1)]
        );
        assert_eq!(scheduler.queued_len(&ModelId::new("small")), 1);
    }

    #[test]
    fn test_next_batch_of_an_unknown_model_is_empty() {
        let mut scheduler = Scheduler::new(POLICY);
        assert!(
            scheduler
                .next_batch(&ModelId::new("other"), 1000)
                .is_empty()
        );
    }
}
