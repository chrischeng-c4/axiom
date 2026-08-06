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
