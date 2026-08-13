use server_lifecycle::{LifecycleController, ShutdownDeadline};
use std::time::Duration;

#[tokio::test]
async fn publication_precedes_draining_and_late_subscribers_see_it() {
    let controller = LifecycleController::serving();
    let mut subscriber = controller.subscribe();
    let deadline = ShutdownDeadline::from_now(Duration::from_secs(1), Duration::ZERO).unwrap();
    let run = controller.clone();
    let task = tokio::spawn(async move { run.shutdown(deadline, "test", "first").await });
    let observation = subscriber.changed().await;
    assert!(observation.phase.is_draining_or_later());
    assert_eq!(subscriber.shutdown_deadline(), Some(deadline));
    let late = controller.subscribe();
    assert_eq!(late.shutdown_deadline(), Some(deadline));
    let _ = task.await.unwrap();
}

#[tokio::test]
async fn first_shutdown_deadline_wins() {
    let controller = LifecycleController::serving();
    let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(8));
    let mut callers = Vec::new();
    for index in 0..8 {
        let controller = controller.clone();
        let barrier = barrier.clone();
        callers.push(tokio::spawn(async move {
            barrier.wait().await;
            let deadline = ShutdownDeadline::from_now(
                Duration::from_secs(index + 1),
                Duration::from_millis(index),
            )
            .unwrap();
            let report = controller
                .shutdown(deadline, format!("caller-{index}"), "concurrent")
                .await;
            (deadline, report)
        }));
    }
    let mut reports = Vec::new();
    for caller in callers {
        reports.push(caller.await);
    }
    let first_reason = reports[0]
        .as_ref()
        .unwrap()
        .1
        .initiating_reason_code
        .clone();
    assert!(reports
        .iter()
        .all(|report| { report.as_ref().unwrap().1.initiating_reason_code == first_reason }));
    let winner_index: u64 = first_reason
        .strip_prefix("caller-")
        .unwrap()
        .parse()
        .unwrap();
    let winning_deadline = reports
        .iter()
        .find(|report| report.as_ref().unwrap().0.total.as_secs() == winner_index + 1)
        .unwrap()
        .as_ref()
        .unwrap()
        .0;
    let published = controller.subscribe().shutdown_deadline().unwrap();
    assert_eq!(published, winning_deadline);
}

#[tokio::test]
async fn serving_waiter_receives_published_deadline() {
    let controller = LifecycleController::serving();
    let mut subscriber = controller.subscribe();
    let deadline = ShutdownDeadline::from_now(Duration::from_secs(1), Duration::ZERO).unwrap();
    let run = controller.clone();
    let task = tokio::spawn(async move { run.shutdown(deadline, "test", "test").await });
    assert_eq!(subscriber.wait_shutdown_deadline().await.unwrap(), deadline);
    let _ = task.await;
}
