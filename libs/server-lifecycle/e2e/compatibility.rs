use std::time::Duration;

use server_lifecycle::{DrainController, DrainState, LifecyclePhase, Readiness};

#[tokio::test]
async fn legacy_drain_is_a_lifecycle_projection_and_skips_intermediate_phases() {
    let drain = DrainController::new();
    assert_eq!(
        drain.lifecycle().observation().phase,
        LifecyclePhase::Serving
    );
    let mut signal = drain.signal();
    assert_eq!(drain.state(), DrainState::Ready);
    assert!(!Readiness::is_draining(&drain));
    let lifecycle = drain.lifecycle();
    lifecycle
        .transition(LifecyclePhase::Degraded, "dependency", "safe mode")
        .unwrap();
    lifecycle
        .transition(LifecyclePhase::Serving, "recovered", "ready")
        .unwrap();
    drain.start_drain();
    assert_eq!(
        tokio::time::timeout(Duration::from_millis(100), signal.changed())
            .await
            .unwrap(),
        DrainState::Draining
    );
    assert!(drain.is_draining());
    assert!(Readiness::is_draining(&drain));
}
