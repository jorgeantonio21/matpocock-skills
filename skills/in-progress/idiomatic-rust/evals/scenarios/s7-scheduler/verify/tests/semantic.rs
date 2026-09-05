//! Independent tests for s7-scheduler. They check meaning, not style: a policy change keeps the
//! work the old policy admitted, and a retirement's outcome is the one record of what it removed.
//! The scorer copies this file into `tests/` of each result tree.

use infersched::{
    ModelId, Policy, RejectReason, Request, RequestId, RetireOutcome, RetireReason, Scheduler,
    SubmitOutcome,
};

const WIDE: Policy = Policy {
    max_prompt_tokens: 4096,
    max_queue_len: 8,
};
const NARROW: Policy = Policy {
    max_prompt_tokens: 2048,
    max_queue_len: 2,
};

fn request(id: u64, prompt_tokens: u32) -> Request {
    Request {
        id: RequestId(id),
        model: ModelId::new("small"),
        prompt_tokens,
    }
}

fn ids(batch: &[Request]) -> Vec<RequestId> {
    batch.iter().map(|request| request.id).collect()
}

#[test]
fn test_a_queued_request_survives_a_stricter_prompt_limit() {
    let mut scheduler = Scheduler::new(WIDE);
    assert_eq!(
        scheduler.submit(request(1, 3000)),
        SubmitOutcome::Queued { position: 0 }
    );
    scheduler.set_policy(NARROW);
    // admitted under WIDE, so still served, whatever NARROW says
    assert_eq!(
        ids(&scheduler.next_batch(&ModelId::new("small"), 4096)),
        vec![RequestId(1)]
    );
}

#[test]
fn test_a_new_submission_is_checked_against_the_new_policy() {
    let mut scheduler = Scheduler::new(WIDE);
    scheduler.set_policy(NARROW);
    assert_eq!(
        scheduler.submit(request(1, 3000)),
        SubmitOutcome::Rejected(RejectReason::TooManyTokens {
            tokens: 3000,
            max: 2048
        })
    );
}

#[test]
fn test_a_queue_longer_than_the_new_limit_is_kept_and_served() {
    let mut scheduler = Scheduler::new(WIDE);
    for id in 1..=4 {
        assert_eq!(
            scheduler.submit(request(id, 100)),
            SubmitOutcome::Queued {
                position: id as usize - 1
            }
        );
    }
    scheduler.set_policy(NARROW);
    assert_eq!(
        scheduler.queued_len(&ModelId::new("small")),
        4,
        "no queued request is evicted"
    );
    // a fifth submission finds the queue over NARROW's limit of 2
    assert_eq!(
        scheduler.submit(request(5, 100)),
        SubmitOutcome::Rejected(RejectReason::QueueFull {
            model: ModelId::new("small"),
            max: 2
        })
    );
    assert_eq!(
        ids(&scheduler.next_batch(&ModelId::new("small"), 4096)),
        vec![RequestId(1), RequestId(2), RequestId(3), RequestId(4)]
    );
}

#[test]
fn test_retire_returns_the_queued_requests_in_submission_order() {
    let mut scheduler = Scheduler::new(WIDE);
    let _ = scheduler.submit(request(1, 100));
    let _ = scheduler.submit(request(2, 200));
    let outcome = scheduler.retire_model(&ModelId::new("small"));
    assert_eq!(
        outcome,
        RetireOutcome::Retired {
            drained: vec![request(1, 100), request(2, 200)]
        }
    );
    // the outcome is the only record: the scheduler holds nothing for the model now
    assert_eq!(scheduler.queued_len(&ModelId::new("small")), 0);
    assert!(
        scheduler
            .next_batch(&ModelId::new("small"), 4096)
            .is_empty()
    );
}

#[test]
fn test_a_submission_to_a_retired_model_is_rejected() {
    let mut scheduler = Scheduler::new(WIDE);
    let _ = scheduler.submit(request(1, 100));
    let _ = scheduler.retire_model(&ModelId::new("small"));
    assert_eq!(
        scheduler.submit(request(2, 100)),
        SubmitOutcome::Rejected(RejectReason::ModelRetired)
    );
}

#[test]
fn test_retiring_twice_and_retiring_an_unknown_model_are_told_apart() {
    let mut scheduler = Scheduler::new(WIDE);
    let _ = scheduler.submit(request(1, 100));
    let _ = scheduler.retire_model(&ModelId::new("small"));
    assert_eq!(
        scheduler.retire_model(&ModelId::new("small")),
        RetireOutcome::Rejected(RetireReason::AlreadyRetired)
    );
    assert_eq!(
        scheduler.retire_model(&ModelId::new("never-seen")),
        RetireOutcome::Rejected(RetireReason::UnknownModel)
    );
}

#[test]
fn test_retiring_a_seen_model_with_an_empty_queue_drains_nothing() {
    let mut scheduler = Scheduler::new(WIDE);
    let _ = scheduler.submit(request(1, 100));
    let _ = scheduler.next_batch(&ModelId::new("small"), 4096);
    assert_eq!(
        scheduler.retire_model(&ModelId::new("small")),
        RetireOutcome::Retired {
            drained: Vec::new()
        }
    );
}

#[test]
fn test_a_batch_takes_a_request_that_exactly_fills_the_budget() {
    let mut scheduler = Scheduler::new(WIDE);
    let _ = scheduler.submit(request(1, 1024));
    let _ = scheduler.submit(request(2, 1024));
    // 1024 + 1024 = 2048, which fits a budget of 2048 exactly
    assert_eq!(
        ids(&scheduler.next_batch(&ModelId::new("small"), 2048)),
        vec![RequestId(1), RequestId(2)]
    );
}
