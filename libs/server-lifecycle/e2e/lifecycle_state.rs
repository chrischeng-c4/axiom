use server_lifecycle::{LifecycleController, LifecyclePhase};

#[test]
fn phases_expose_probe_predicates() {
    for (phase, healthy, ready, started) in [
        (LifecyclePhase::Starting, true, false, false),
        (LifecyclePhase::Recovering, true, false, false),
        (LifecyclePhase::Serving, true, true, true),
        (LifecyclePhase::Degraded, true, true, true),
        (LifecyclePhase::Draining, true, false, true),
        (LifecyclePhase::Stopping, true, false, true),
        (LifecyclePhase::Stopped, false, false, true),
        (LifecyclePhase::Fatal, false, false, false),
    ] {
        let controller = LifecycleController::new();
        reach(&controller, phase);
        let observation = controller.observation();
        assert_eq!(observation.is_healthy(), healthy, "{phase:?}");
        assert_eq!(observation.is_ready(), ready, "{phase:?}");
        assert_eq!(observation.startup_succeeded(), started, "{phase:?}");
        assert_eq!(
            observation.admission_open,
            matches!(phase, LifecyclePhase::Serving | LifecyclePhase::Degraded),
            "{phase:?}"
        );
    }
}

#[test]
fn degraded_admission_can_close_and_reopen_with_generations() {
    let controller = LifecycleController::serving();
    let open = controller
        .transition_degraded(true, "dependency", "safe")
        .unwrap();
    assert!(open.admission_open);
    let closed = controller
        .transition_degraded(false, "dependency", "closed")
        .unwrap();
    assert!(!closed.admission_open);
    assert!(closed.generation > open.generation);
    let reopened = controller
        .transition_degraded(true, "dependency", "recovered")
        .unwrap();
    assert!(reopened.admission_open);
    assert!(reopened.generation > closed.generation);
}

#[tokio::test]
async fn transition_events_do_not_coalesce() {
    let controller = LifecycleController::new();
    let mut events = controller.subscribe_transitions();
    controller
        .transition(LifecyclePhase::Recovering, "recover", "retry")
        .unwrap();
    controller
        .transition(LifecyclePhase::Serving, "serve", "ready")
        .unwrap();
    let e0 = events.next().await.unwrap();
    let e1 = events.next().await.unwrap();
    let e2 = events.next().await.unwrap();
    assert_eq!((e0.phase, e0.generation), (LifecyclePhase::Starting, 0));
    assert_eq!((e1.phase, e1.generation), (LifecyclePhase::Recovering, 1));
    assert_eq!((e2.phase, e2.generation), (LifecyclePhase::Serving, 2));
}

#[tokio::test]
async fn transition_events_report_bounded_lag() {
    let controller = LifecycleController::serving();
    let mut events = controller.subscribe_transitions();
    assert_eq!(events.next().await.unwrap().generation, 0);
    controller.transition_degraded(true, "dep", "open").unwrap();
    for index in 0..80 {
        controller
            .transition_degraded(index % 2 == 0, "dep", "toggle")
            .unwrap();
    }
    match events.next().await {
        Err(server_lifecycle::LifecycleEventError::Lagged(skipped)) => assert!(skipped > 0),
        other => panic!("expected bounded lag, got {other:?}"),
    }
}

#[test]
fn valid_edges_and_reverse_transitions_are_checked() {
    let edges = [
        (LifecyclePhase::Starting, LifecyclePhase::Recovering),
        (LifecyclePhase::Starting, LifecyclePhase::Serving),
        (LifecyclePhase::Starting, LifecyclePhase::Degraded),
        (LifecyclePhase::Starting, LifecyclePhase::Draining),
        (LifecyclePhase::Starting, LifecyclePhase::Fatal),
        (LifecyclePhase::Recovering, LifecyclePhase::Serving),
        (LifecyclePhase::Recovering, LifecyclePhase::Degraded),
        (LifecyclePhase::Recovering, LifecyclePhase::Draining),
        (LifecyclePhase::Recovering, LifecyclePhase::Fatal),
        (LifecyclePhase::Serving, LifecyclePhase::Degraded),
        (LifecyclePhase::Serving, LifecyclePhase::Draining),
        (LifecyclePhase::Serving, LifecyclePhase::Fatal),
        (LifecyclePhase::Degraded, LifecyclePhase::Serving),
        (LifecyclePhase::Degraded, LifecyclePhase::Draining),
        (LifecyclePhase::Degraded, LifecyclePhase::Fatal),
        (LifecyclePhase::Draining, LifecyclePhase::Stopping),
        (LifecyclePhase::Draining, LifecyclePhase::Fatal),
        (LifecyclePhase::Stopping, LifecyclePhase::Stopped),
        (LifecyclePhase::Stopping, LifecyclePhase::Fatal),
    ];
    for (from, to) in edges {
        let controller = LifecycleController::new();
        reach(&controller, from);
        assert!(
            controller.transition(to, "edge", "valid").is_ok(),
            "{from:?}->{to:?}"
        );
    }

    let controller = LifecycleController::serving();
    let before = controller.observation();
    assert!(controller
        .transition(LifecyclePhase::Recovering, "reverse", "reject")
        .is_err());
    assert_eq!(controller.observation(), before);
    let same = controller
        .transition(LifecyclePhase::Serving, "ignored", "ignored")
        .unwrap();
    assert_eq!(same.generation, before.generation);
    assert_eq!(same.reason_code, before.reason_code);
}

fn reach(controller: &LifecycleController, phase: LifecyclePhase) {
    match phase {
        LifecyclePhase::Starting => {}
        LifecyclePhase::Recovering
        | LifecyclePhase::Serving
        | LifecyclePhase::Degraded
        | LifecyclePhase::Draining
        | LifecyclePhase::Fatal => {
            controller.transition(phase, "test", "table").unwrap();
        }
        LifecyclePhase::Stopping => {
            controller
                .transition(LifecyclePhase::Draining, "test", "table")
                .unwrap();
            controller
                .transition(LifecyclePhase::Stopping, "test", "table")
                .unwrap();
        }
        LifecyclePhase::Stopped => {
            controller
                .transition(LifecyclePhase::Draining, "test", "table")
                .unwrap();
            controller
                .transition(LifecyclePhase::Stopping, "test", "table")
                .unwrap();
            controller
                .transition(LifecyclePhase::Stopped, "test", "table")
                .unwrap();
        }
    }
}
