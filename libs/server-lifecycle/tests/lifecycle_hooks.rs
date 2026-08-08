use std::sync::{Arc, Mutex};
use std::time::Duration;

use server_lifecycle::{HookStage, HookStatus, LifecycleController, ShutdownDeadline};

struct DropMarker(Arc<std::sync::atomic::AtomicBool>);

impl Drop for DropMarker {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

#[tokio::test]
async fn hooks_run_in_stage_then_registration_order_and_keep_failures() {
    let controller = LifecycleController::serving();
    let seen = Arc::new(Mutex::new(Vec::new()));
    for (stage, name) in [
        (HookStage::FinalFlush, "flush"),
        (HookStage::AdmissionStop, "admission"),
        (HookStage::TransportDrain, "transport"),
        (HookStage::DomainQuiesce, "domain"),
        (HookStage::BackgroundStop, "background"),
    ] {
        let seen = seen.clone();
        controller
            .register_hook(stage, name, move |_context| {
                let seen = seen.clone();
                async move {
                    seen.lock().unwrap().push(name);
                    if name == "domain" {
                        Err("domain failed".into())
                    } else {
                        Ok(())
                    }
                }
            })
            .unwrap();
    }
    let report = controller
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_secs(1), Duration::from_millis(20)).unwrap(),
            "signal",
            "test",
        )
        .await;
    assert_eq!(
        *seen.lock().unwrap(),
        vec!["admission", "transport", "domain", "background", "flush"]
    );
    assert!(matches!(report.outcomes[2].status, HookStatus::Failed));
    assert_eq!(
        report.terminal_phase,
        server_lifecycle::LifecyclePhase::Stopped
    );
}

#[tokio::test(start_paused = true)]
async fn exhausted_usable_budget_does_not_invoke_later_hooks() {
    let controller = LifecycleController::serving();
    let ran = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let cancelled = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let marker = cancelled.clone();
    controller
        .register_hook(HookStage::AdmissionStop, "slow", move |_context| {
            let marker = marker.clone();
            async move {
                let _marker = DropMarker(marker);
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            }
        })
        .unwrap();
    let later = ran.clone();
    controller
        .register_hook(HookStage::FinalFlush, "later", move |_context| {
            let later = later.clone();
            async move {
                later.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        })
        .unwrap();
    let report = controller
        .shutdown(
            ShutdownDeadline::from_now(Duration::from_millis(100), Duration::from_millis(20))
                .unwrap(),
            "test",
            "budget",
        )
        .await;
    assert!(report
        .outcomes
        .iter()
        .all(|outcome| matches!(outcome.status, HookStatus::TimedOut)));
    assert_eq!(ran.load(std::sync::atomic::Ordering::SeqCst), 0);
    assert!(cancelled.load(std::sync::atomic::Ordering::SeqCst));
    assert!(report.remaining_reserve >= Duration::from_millis(20));
}

#[tokio::test]
async fn panicking_hook_is_failed_and_later_hooks_and_callers_complete() {
    let controller = LifecycleController::serving();
    let later_ran = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    controller
        .register_hook(HookStage::AdmissionStop, "panic", |_context| async {
            panic!("user hook exploded");
        })
        .unwrap();
    let later = later_ran.clone();
    controller
        .register_hook(HookStage::FinalFlush, "later", move |_context| {
            let later = later.clone();
            async move {
                later.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            }
        })
        .unwrap();

    let deadline =
        ShutdownDeadline::from_now(Duration::from_secs(1), Duration::from_millis(20)).unwrap();
    let first_controller = controller.clone();
    let (first, second) = tokio::join!(
        first_controller.shutdown(deadline, "signal", "panic test"),
        controller.shutdown(deadline, "operator", "same run"),
    );
    assert!(Arc::ptr_eq(&first, &second));
    assert!(matches!(first.outcomes[0].status, HookStatus::Failed));
    assert!(first.outcomes[0]
        .error
        .as_deref()
        .is_some_and(|error| !error.is_empty()));
    assert!(matches!(first.outcomes[1].status, HookStatus::Completed));
    assert_eq!(later_ran.load(std::sync::atomic::Ordering::SeqCst), 1);
}
