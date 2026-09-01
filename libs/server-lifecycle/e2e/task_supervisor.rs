use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use server_lifecycle::{HookStage, HookStatus, TaskSupervisor};

#[tokio::test]
async fn task_errors_and_panics_are_collected_and_final_flush_still_runs() {
    let supervisor =
        TaskSupervisor::new(Duration::from_secs(1), Duration::from_millis(20)).unwrap();
    let (stop_error, stopped_error) = tokio::sync::oneshot::channel();
    let error_task = tokio::spawn(async move {
        let _ = stopped_error.await;
        Err::<(), _>("transport failed")
    });
    supervisor
        .register_oneshot_task(HookStage::TransportDrain, "error", stop_error, error_task)
        .unwrap();

    let (stop_panic, stopped_panic) = tokio::sync::oneshot::channel();
    let panic_task = tokio::spawn(async move {
        let _ = stopped_panic.await;
        panic!("transport panic");
        #[allow(unreachable_code)]
        Ok::<(), &'static str>(())
    });
    supervisor
        .register_oneshot_task(HookStage::TransportDrain, "panic", stop_panic, panic_task)
        .unwrap();

    let flushed = Arc::new(AtomicBool::new(false));
    let flush = flushed.clone();
    supervisor
        .register_hook(HookStage::FinalFlush, "flush", move |_| {
            let flush = flush.clone();
            async move {
                flush.store(true, Ordering::SeqCst);
                Ok(())
            }
        })
        .unwrap();

    let report = supervisor.shutdown("test", "collect failures").await;
    assert!(matches!(report.outcomes[0].status, HookStatus::Failed));
    assert!(matches!(report.outcomes[1].status, HookStatus::Failed));
    assert!(matches!(report.outcomes[2].status, HookStatus::Completed));
    assert!(flushed.load(Ordering::SeqCst));
}

#[tokio::test(start_paused = true)]
async fn deadline_cancels_a_task_instead_of_detaching_it() {
    struct Dropped(Arc<AtomicBool>);
    impl Drop for Dropped {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let supervisor =
        TaskSupervisor::new(Duration::from_millis(100), Duration::from_millis(20)).unwrap();
    let dropped = Arc::new(AtomicBool::new(false));
    let marker = dropped.clone();
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let _guard = Dropped(marker);
        let _ = stopped.await;
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok::<(), &'static str>(())
    });
    supervisor
        .register_oneshot_task(HookStage::BackgroundStop, "hung", stop, task)
        .unwrap();

    let report = supervisor.shutdown("test", "deadline").await;
    assert!(matches!(report.outcomes[0].status, HookStatus::TimedOut));
    tokio::task::yield_now().await;
    assert!(dropped.load(Ordering::SeqCst));
}
