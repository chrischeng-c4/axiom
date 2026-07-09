// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#unit-test
// HANDWRITE-BEGIN gap="missing-generator:unit-test:defer-rate-limits" tracker="#766" reason="Focused tests for queue dispatch budget, max-in-flight, and lease expiry reclaim."
use chrono::{Duration, Utc};
use defer::{
    CreateTask, DeferScheduler, QueueControlState, QueuePolicy, SchedulerError, Target,
    DEFAULT_PRIORITY,
};

fn target() -> Target {
    Target {
        url: "http://worker.local/task".into(),
        method: "POST".into(),
        headers: Default::default(),
    }
}

fn due_task(id: &str) -> CreateTask {
    CreateTask {
        task_id: id.into(),
        target: target(),
        payload: serde_json::json!({ "id": id }),
        schedule_at: Utc::now(),
        priority: DEFAULT_PRIORITY,
        max_attempts: 3,
    }
}

#[test]
fn max_dispatch_per_tick_limits_each_scheduler_drain() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue(
        "q",
        QueuePolicy {
            max_in_flight: 10,
            max_dispatch_per_tick: 2,
            ..QueuePolicy::default()
        },
    );
    for id in ["a", "b", "c"] {
        let mut task = due_task(id);
        task.schedule_at = now;
        s.create_task("q", task).unwrap();
    }

    assert_eq!(s.lease_due("q", now, 10).unwrap().len(), 2);
    assert_eq!(s.lease_due("q", now, 10).unwrap().len(), 1);
}

#[test]
fn expired_leases_return_to_scheduler_control() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue(
        "q",
        QueuePolicy {
            max_in_flight: 1,
            max_dispatch_per_tick: 10,
            lease_ttl_ms: 100,
            retry_backoff_ms: 100,
            ..QueuePolicy::default()
        },
    );
    let mut task = due_task("a");
    task.schedule_at = now;
    s.create_task("q", task).unwrap();

    let lease = s.lease_due("q", now, 1).unwrap().remove(0);
    assert!(s.lease_due("q", now, 1).unwrap().is_empty());

    let expired = s
        .reclaim_expired("q", lease.expires_at + Duration::milliseconds(1))
        .unwrap();
    assert_eq!(expired, vec!["a".to_string()]);

    let retry_at = lease.expires_at + Duration::milliseconds(101);
    let retry = s.lease_due("q", retry_at, 1).unwrap();
    assert_eq!(retry.len(), 1);
    assert_eq!(retry[0].task_id, "a");
    assert_eq!(retry[0].attempt, 2);
}

#[test]
fn per_queue_rate_bucket_limits_dispatch_over_time() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue(
        "q",
        QueuePolicy {
            max_in_flight: 10,
            max_dispatch_per_tick: 10,
            max_dispatches_per_second: 1,
            max_burst_size: 1,
            ..QueuePolicy::default()
        },
    );
    for id in ["a", "b", "c"] {
        let mut task = due_task(id);
        task.schedule_at = now;
        s.create_task("q", task).unwrap();
    }

    assert_eq!(s.lease_due("q", now, 10).unwrap().len(), 1);
    assert!(s.lease_due("q", now, 10).unwrap().is_empty());
    assert!(s
        .lease_due("q", now + Duration::milliseconds(999), 10)
        .unwrap()
        .is_empty());
    assert_eq!(
        s.lease_due("q", now + Duration::milliseconds(1_000), 10)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn queue_pause_resume_is_per_queue_control() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue("slow", QueuePolicy::default());
    s.configure_queue("fast", QueuePolicy::default());
    for queue in ["slow", "fast"] {
        let mut task = due_task(queue);
        task.schedule_at = now;
        s.create_task(queue, task).unwrap();
    }

    s.pause_queue("slow").unwrap();
    assert!(s.lease_due("slow", now, 1).unwrap().is_empty());
    assert_eq!(s.lease_due("fast", now, 1).unwrap().len(), 1);
    assert_eq!(
        s.queue_snapshot("slow").unwrap().control_state,
        QueueControlState::Paused
    );

    s.resume_queue("slow").unwrap();
    assert_eq!(s.lease_due("slow", now, 1).unwrap().len(), 1);
}

#[test]
fn disabled_queue_rejects_new_tasks_and_stops_dispatch() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue("q", QueuePolicy::default());
    let mut task = due_task("a");
    task.schedule_at = now;
    s.create_task("q", task).unwrap();

    s.disable_queue("q").unwrap();
    assert!(s.lease_due("q", now, 1).unwrap().is_empty());
    assert_eq!(
        s.create_task("q", due_task("b")).unwrap_err(),
        SchedulerError::QueueDisabled("q".into())
    );
    assert_eq!(
        s.queue_snapshot("q").unwrap().control_state,
        QueueControlState::Disabled
    );
}

#[test]
fn updating_one_queue_policy_does_not_change_other_queues() {
    let now = Utc::now();
    let mut s = DeferScheduler::new();
    s.configure_queue("limited", QueuePolicy::default());
    s.configure_queue("open", QueuePolicy::default());
    s.update_queue_policy(
        "limited",
        QueuePolicy {
            max_in_flight: 1,
            max_dispatch_per_tick: 1,
            ..QueuePolicy::default()
        },
    )
    .unwrap();
    for queue in ["limited", "open"] {
        for id in ["a", "b"] {
            let mut task = due_task(&format!("{queue}-{id}"));
            task.schedule_at = now;
            s.create_task(queue, task).unwrap();
        }
    }

    assert_eq!(s.lease_due("limited", now, 10).unwrap().len(), 1);
    assert_eq!(s.lease_due("open", now, 10).unwrap().len(), 2);
    assert_eq!(
        s.queue_snapshot("limited")
            .unwrap()
            .policy
            .max_dispatch_per_tick,
        1
    );
    assert_eq!(
        s.queue_snapshot("open")
            .unwrap()
            .policy
            .max_dispatch_per_tick,
        QueuePolicy::default().max_dispatch_per_tick
    );
}
// HANDWRITE-END
