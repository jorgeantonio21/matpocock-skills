//! An inference request scheduler.
//!
//! 1. A request is admitted against the policy when it is submitted, and queued per model.
//! 2. `next_batch` drains a model's queue in submission order while the prompt tokens fit the budget.
//! 3. A policy change applies to later submissions. An admission, once given, stands.
//! 4. A retired model rejects submissions. The retirement outcome carries the requests it drained.
//! 5. Order is submission order. Time is not an input.

use std::collections::{HashMap, HashSet, VecDeque};

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
    /// The model was retired. No later submission for it is queued.
    ModelRetired,
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

/// Why a model was not retired.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RetireReason {
    /// The model was retired before.
    AlreadyRetired,
    /// No request ever named the model, so there is nothing to retire.
    UnknownModel,
}

/// What `retire_model` did.
#[must_use = "the drained requests must be answered"]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RetireOutcome {
    /// The model's queue is gone. `drained` holds what was in it, in submission order.
    Retired {
        /// The requests that were queued. Each client is owed a reply.
        drained: Vec<Request>,
    },
    /// Nothing changed.
    Rejected(RetireReason),
}

/// The scheduler: one queue per model, the models retired so far, and the policy that admits
/// new requests.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Scheduler {
    policy: Policy,
    queues: HashMap<ModelId, VecDeque<Request>>,
    retired: HashSet<ModelId>,
}

impl Scheduler {
    /// An empty scheduler under `policy`.
    #[must_use]
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            queues: HashMap::new(),
            retired: HashSet::new(),
        }
    }

    /// The policy in force.
    #[must_use]
    pub const fn policy(&self) -> Policy {
        self.policy
    }

    /// Replaces the policy for submissions from now on. A request already queued was admitted
    /// under the policy of its time, and that admission stands: nothing is evicted or re-checked.
    pub fn set_policy(&mut self, policy: Policy) {
        self.policy = policy;
    }

    /// Admits `request` under the policy and queues it behind its model's earlier requests.
    pub fn submit(&mut self, request: Request) -> SubmitOutcome {
        // A retired model takes no work, whatever the request looks like.
        if self.retired.contains(&request.model) {
            return SubmitOutcome::Rejected(RejectReason::ModelRetired);
        }
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
            && used + front.prompt_tokens <= budget
        {
            used += front.prompt_tokens;
            batch.extend(queue.pop_front());
        }
        batch
    }

    /// Retires `model`: its queue is removed and later submissions for it are rejected.
    /// The outcome carries the drained requests. It is the only record of them.
    pub fn retire_model(&mut self, model: &ModelId) -> RetireOutcome {
        // A model is retired once. The second call has nothing to drain and says so.
        if self.retired.contains(model) {
            return RetireOutcome::Rejected(RetireReason::AlreadyRetired);
        }
        // A model with no queue was never submitted to. There is nothing to retire.
        let Some(queue) = self.queues.remove(model) else {
            return RetireOutcome::Rejected(RetireReason::UnknownModel);
        };
        self.retired.insert(model.clone());
        RetireOutcome::Retired {
            drained: queue.into(),
        }
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
    fn test_next_batch_takes_what_fits_including_an_exact_fit() {
        let mut scheduler = Scheduler::new(POLICY);
        let _ = scheduler.submit(request(1, 1000));
        let _ = scheduler.submit(request(2, 1000));
        // 1000 + 1000 = 2000 fits a budget of 2000 exactly
        let batch = scheduler.next_batch(&ModelId::new("small"), 2000);
        assert_eq!(
            batch.iter().map(|r| r.id).collect::<Vec<_>>(),
            vec![RequestId(1), RequestId(2)]
        );
        assert_eq!(scheduler.queued_len(&ModelId::new("small")), 0);
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

    #[test]
    fn test_set_policy_keeps_the_requests_the_old_policy_admitted() {
        let mut scheduler = Scheduler::new(POLICY);
        let _ = scheduler.submit(request(1, 3000));
        scheduler.set_policy(Policy {
            max_prompt_tokens: 2048,
            max_queue_len: 1,
        });
        // the queued 3000-token request stays; a new one of the same size is rejected
        assert_eq!(scheduler.queued_len(&ModelId::new("small")), 1);
        assert_eq!(
            scheduler.submit(request(2, 3000)),
            SubmitOutcome::Rejected(RejectReason::TooManyTokens {
                tokens: 3000,
                max: 2048
            })
        );
        assert_eq!(scheduler.next_batch(&ModelId::new("small"), 4096).len(), 1);
    }

    #[test]
    fn test_retire_drains_the_queue_and_closes_the_model() {
        let mut scheduler = Scheduler::new(POLICY);
        let _ = scheduler.submit(request(1, 100));
        let _ = scheduler.submit(request(2, 200));
        assert_eq!(
            scheduler.retire_model(&ModelId::new("small")),
            RetireOutcome::Retired {
                drained: vec![request(1, 100), request(2, 200)]
            }
        );
        assert_eq!(
            scheduler.submit(request(3, 100)),
            SubmitOutcome::Rejected(RejectReason::ModelRetired)
        );
        assert_eq!(
            scheduler.retire_model(&ModelId::new("small")),
            RetireOutcome::Rejected(RetireReason::AlreadyRetired)
        );
        assert_eq!(
            scheduler.retire_model(&ModelId::new("other")),
            RetireOutcome::Rejected(RetireReason::UnknownModel)
        );
    }
}
