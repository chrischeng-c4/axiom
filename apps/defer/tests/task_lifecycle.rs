// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-task-lifecycle" tracker="#766" reason="Conformance tests for Defer's ETA-first, priority-aware push queue scheduler core."
use chrono::{Duration, Utc};
use defer::{
    CreateTask, DeferScheduler, NackOutcome, QueuePolicy, Target, TaskStatus, DEFAULT_PRIORITY,
};

fn target() -> Target {
    Target {
        url: "http://worker.local/task".into(),
        method: "POST".into(),
        headers: Default::default(),
    }
}

fn task(id: &str, due_offset_ms: i64, priority: u8, max_attempts: u32) -> CreateTask {
    CreateTask {
        task_id: id.into(),
        target: target(),
        payload: serde_json::json!({ "id": id }),
        schedule_at: Utc::now() + Duration::milliseconds(due_offset_ms),
        priority,
        max_attempts,
    }
}

fn scheduler(policy: QueuePolicy) -> DeferScheduler {
    let mut s = DeferScheduler::new();
    s.configure_queue("q", policy);
    s
}

#[test]
fn eta_is_checked_before_priority() {
    let now = Utc::now();
    let mut s = scheduler(QueuePolicy::default());
    let mut future_high = task("future-high", 0, 250, 3);
    future_high.schedule_at = now + Duration::seconds(30);
    let mut due_low = task("due-low", 0, 1, 3);
    due_low.schedule_at = now;

    s.create_task("q", future_high).unwrap();
    s.create_task("q", due_low).unwrap();

    let leases = s.lease_due("q", now, 10).unwrap();
    assert_eq!(leases.len(), 1);
    assert_eq!(leases[0].task_id, "due-low");
}

#[test]
fn priority_orders_due_tasks_and_same_priority_uses_creation_fifo() {
    let now = Utc::now();
    let mut s = scheduler(QueuePolicy::default());
    for (id, priority) in [
        ("normal-a", DEFAULT_PRIORITY),
        ("high", 200),
        ("normal-b", DEFAULT_PRIORITY),
        ("low", 1),
    ] {
        let mut t = task(id, 0, priority, 3);
        t.schedule_at = now;
        s.create_task("q", t).unwrap();
    }

    let ids: Vec<_> = s
        .lease_due("q", now, 10)
        .unwrap()
        .into_iter()
        .map(|l| l.task_id)
        .collect();
    assert_eq!(ids, vec!["high", "normal-a", "normal-b", "low"]);
}

#[test]
fn defer_owns_dispatch_budget_and_concurrency() {
    let now = Utc::now();
    let mut s = scheduler(QueuePolicy {
        max_in_flight: 2,
        max_dispatch_per_tick: 5,
        ..QueuePolicy::default()
    });
    for id in ["a", "b", "c"] {
        let mut t = task(id, 0, DEFAULT_PRIORITY, 3);
        t.schedule_at = now;
        s.create_task("q", t).unwrap();
    }

    let first = s.lease_due("q", now, 10).unwrap();
    assert_eq!(first.len(), 2, "concurrency cap limits dispatch");
    assert!(s.lease_due("q", now, 10).unwrap().is_empty());

    assert!(s.ack("q", &first[0].attempt_id).unwrap());
    let second = s.lease_due("q", now, 10).unwrap();
    assert_eq!(second.len(), 1, "ack frees one dispatch slot");
    assert_eq!(second[0].task_id, "c");
}

#[test]
fn nack_reschedules_then_dead_letters_after_max_attempts() {
    let now = Utc::now();
    let mut s = scheduler(QueuePolicy {
        retry_backoff_ms: 1_000,
        ..QueuePolicy::default()
    });
    let mut t = task("retry", 0, DEFAULT_PRIORITY, 2);
    t.schedule_at = now;
    s.create_task("q", t).unwrap();

    let first = s.lease_due("q", now, 1).unwrap().remove(0);
    let outcome = s.nack("q", &first.attempt_id, now).unwrap().unwrap();
    let next_at = match outcome {
        NackOutcome::Retried { next_at } => next_at,
        NackOutcome::DeadLettered => panic!("first failure should retry"),
    };
    assert!(s
        .lease_due("q", next_at - Duration::milliseconds(1), 1)
        .unwrap()
        .is_empty());

    let second = s.lease_due("q", next_at, 1).unwrap().remove(0);
    assert_eq!(second.attempt, 2);
    assert_eq!(
        s.nack("q", &second.attempt_id, next_at).unwrap(),
        Some(NackOutcome::DeadLettered)
    );
    assert_eq!(
        s.status("q", "retry").unwrap(),
        Some(TaskStatus::DeadLettered)
    );
}

#[test]
fn cancel_prevents_dispatch() {
    let now = Utc::now();
    let mut s = scheduler(QueuePolicy::default());
    let mut t = task("cancel-me", 0, DEFAULT_PRIORITY, 3);
    t.schedule_at = now;
    s.create_task("q", t).unwrap();
    assert!(s.cancel("q", "cancel-me").unwrap());
    assert!(s.lease_due("q", now, 1).unwrap().is_empty());
    assert_eq!(
        s.status("q", "cancel-me").unwrap(),
        Some(TaskStatus::Canceled)
    );
}
// HANDWRITE-END
