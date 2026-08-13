use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;

use server_lifecycle::{HookStage, LifecycleController, ShutdownDeadline};

#[tokio::test]
async fn concurrent_callers_share_one_owned_run() {
    let controller = LifecycleController::serving();
    let calls = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(tokio::sync::Notify::new());
    for stage in [
        HookStage::AdmissionStop,
        HookStage::TransportDrain,
        HookStage::DomainQuiesce,
        HookStage::BackgroundStop,
        HookStage::FinalFlush,
    ] {
        let calls = calls.clone();
        let started = started.clone();
        controller
            .register_hook(stage, "hook", move |_context| {
                let calls = calls.clone();
                let started = started.clone();
                async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    started.notify_one();
                    Ok(())
                }
            })
            .unwrap();
    }
    let deadline =
        ShutdownDeadline::from_now(Duration::from_secs(1), Duration::from_millis(20)).unwrap();
    let first = controller.clone();
    let abandoned = tokio::spawn(async move { first.shutdown(deadline, "signal", "first").await });
    started.notified().await;
    abandoned.abort();
    let report = controller.shutdown(deadline, "operator", "second").await;
    assert_eq!(calls.load(Ordering::SeqCst), 5);
    assert_eq!(report.initiating_reason_code, "signal");
    assert!(controller
        .register_hook(HookStage::FinalFlush, "late", |_context| async { Ok(()) })
        .is_err());
}
