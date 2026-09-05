use inference_case::{Request, Scheduler};
fn request(id: u64, tokens: u32) -> Request { Request { id, tokens, children: vec![id + 10, id + 20] } }
#[test]
fn historical_admission_survives_policy_changes() {
    let mut scheduler = Scheduler::default();
    scheduler.set_max_new_tokens(10);
    scheduler.admit(request(1, 8)).unwrap();
    scheduler.set_max_new_tokens(4);
    assert_eq!(scheduler.admit(request(2, 8)), Err(request(2, 8)));
    scheduler.admit(request(3, 3)).unwrap();
    scheduler.set_max_new_tokens(0);
    assert_eq!(scheduler.next_accepted(), Some(request(1, 8)));
    assert_eq!(scheduler.next_accepted(), Some(request(3, 3)));
    assert_eq!(scheduler.next_accepted(), None);
}
#[test]
fn removal_owns_authoritative_children_and_snapshot_count() {
    let mut scheduler = Scheduler::default();
    scheduler.set_max_new_tokens(10);
    scheduler.admit(request(1, 1)).unwrap();
    scheduler.admit(request(2, 2)).unwrap();
    assert_eq!(scheduler.remove(99), None);
    let removed = scheduler.remove(1).unwrap();
    scheduler.admit(request(3, 3)).unwrap();
    assert_eq!(removed.children, vec![11, 21]);
    assert_eq!(removed.remaining, 1);
    assert_eq!(scheduler.next_accepted(), Some(request(2, 2)));
}
#[test]
fn rejected_input_is_preserved() {
    let mut scheduler = Scheduler::default();
    assert_eq!(scheduler.admit(request(1, 0)), Err(request(1, 0)));
    assert_eq!(scheduler.next_accepted(), None);
}
