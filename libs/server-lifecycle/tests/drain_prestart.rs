// <HANDWRITE gap="missing-generator:unit-test:e1b34274" tracker="#1884" reason="scaffold for libs/server-lifecycle/tests/drain_prestart.rs — fill in by hand and update tracker when codegen is ready">
use std::time::Duration;

use server_lifecycle::DrainController;

/// A drain that arrives before a plane subscribes is durable process state,
/// not a lossy watch notification.
#[tokio::test]
async fn receiverless_drain_persists_for_late_subscriber() {
    let drain = DrainController::new();
    drain.start_drain();

    let mut late_subscriber = drain.signal();
    let state = tokio::time::timeout(Duration::from_millis(50), late_subscriber.changed())
        .await
        .expect("late subscriber observes already-published drain");
    assert!(state.is_draining());
}

/// The TCP and admin shutdown futures are constructed before serving begins.
/// If draining flips during startup, both must complete without waiting for a
/// second signal.
#[tokio::test]
async fn both_plane_signals_observe_prestart_drain() {
    let drain = DrainController::new();
    let mut tcp_shutdown = drain.signal();
    let mut admin_shutdown = drain.signal();
    drain.start_drain();

    let (tcp, admin) = tokio::join!(
        tokio::time::timeout(Duration::from_millis(50), tcp_shutdown.changed()),
        tokio::time::timeout(Duration::from_millis(50), admin_shutdown.changed()),
    );
    assert!(tcp
        .expect("TCP shutdown observes pre-start drain")
        .is_draining());
    assert!(admin
        .expect("admin shutdown observes pre-start drain")
        .is_draining());
}
// </HANDWRITE>
