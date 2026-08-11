use serde_json::json;
use service_k8s::crd::normalize_unsigned_integer_formats;
use service_k8s::lifecycle::{
    LifecyclePolicy, LifecyclePolicyError, ProbeTiming, TerminationBudget,
};
use service_k8s::render::common::ServicePodTemplate;
use service_k8s::render::guaranteed_resources;
use service_k8s::render::RenderCtx;

fn test_cx() -> RenderCtx<'static> {
    RenderCtx {
        app: "lumen",
        manager: "lumen-operator",
        api_version: "lumen.axiom.dev/v1alpha1",
        kind: "Lumen",
        name: "test-svc",
        ns: "default",
        owner: None,
    }
}

fn test_pod_template<'a>(cx: &'a RenderCtx<'a>) -> ServicePodTemplate<'a> {
    ServicePodTemplate {
        cx,
        component: "server",
        image: "lumen:latest",
        image_pull_policy: "IfNotPresent",
        command: vec!["lumen".into()],
        args: vec!["run".into()],
        ports: vec![json!({ "name": "http", "containerPort": 9080 })],
        env: vec![],
        env_from: vec![],
        resources: guaranteed_resources("100m", "128Mi"),
        readiness_probe: None,
        liveness_probe: None,
        startup_probe: None,
        lifecycle: None,
        container_security_context: None,
        pod_security_context: None,
        service_account_name: Some("lumen"),
        termination_grace_period_seconds: None,
        volumes: vec![],
        volume_mounts: vec![],
        pod_annotations: None,
        topology_spread_constraints: vec![],
    }
}

#[test]
fn policy_shape_and_serde_round_trip() {
    let policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 10,
            timeout_seconds: 2,
            failure_threshold: 3,
            success_threshold: 1,
        },
    };

    let budget = policy.validate().expect("policy should be valid");
    assert_eq!(budget.total_grace_period_seconds(), 60);
    assert_eq!(budget.runtime_deadline_seconds(), 45);
    assert_eq!(budget.sigkill_reserve_seconds(), 10);
    assert_eq!(budget.min_hook_duration_seconds(), 5);
    assert_eq!(
        budget.probe_timing(),
        ProbeTiming {
            period_seconds: 10,
            timeout_seconds: 2,
            failure_threshold: 3,
            success_threshold: 1,
        }
    );

    // Serde round trip preserves every field without loss
    let json_str = serde_json::to_string(&policy).expect("serialization should succeed");
    let deserialized: LifecyclePolicy =
        serde_json::from_str(&json_str).expect("deserialization should succeed");
    assert_eq!(policy, deserialized);

    // Validated budget produces identical raw policy
    assert_eq!(budget.raw_policy(), policy);
}

#[test]
fn budget_validation_measurements() {
    // Measurement 1: total 60, runtime 45, reserve 10, min_hook 5 -> succeeds
    let m1_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m1_budget = m1_policy.validate().expect("Measurement 1 must succeed");
    assert_eq!(m1_budget.runtime_deadline_seconds(), 45);
    assert_eq!(m1_budget.sigkill_reserve_seconds(), 10);

    // Measurement 2: total 60, runtime 55, reserve 10, min_hook 5 -> total budget invariant violation
    let m2_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 55,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m2_err = m2_policy
        .validate()
        .expect_err("Measurement 2 must fail total budget invariant");
    let m2_expected = LifecyclePolicyError::BudgetExceedsTotal {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 55,
        sigkill_reserve_seconds: 10,
    };
    assert_eq!(m2_err, m2_expected);
    println!("Row 2 reason: {}", m2_err);

    // Measurement 3: total 60, runtime 3, reserve 10, min_hook 5 -> app minimum invariant violation
    let m3_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 3,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m3_err = m3_policy
        .validate()
        .expect_err("Measurement 3 must fail app minimum invariant");
    let m3_expected = LifecyclePolicyError::RuntimeBelowMinimumHook {
        runtime_deadline_seconds: 3,
        min_hook_duration_seconds: 5,
    };
    assert_eq!(m3_err, m3_expected);
    println!("Row 3 reason: {}", m3_err);

    // Distinctness check between Row 2 and Row 3 reasons
    assert_ne!(m2_err, m3_err);
    assert_ne!(m2_err.to_string(), m3_err.to_string());

    // Measurement 4: runtime set to u64::MAX -> overflow violation without wrap/clamp/panic
    let m4_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: u64::MAX,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m4_err = m4_policy
        .validate()
        .expect_err("Measurement 4 must fail with overflow reason");
    assert_eq!(m4_err, LifecyclePolicyError::Overflow);
    println!("Row 4 reason: {}", m4_err);

    // Measurement 5: total grace 0 -> zero total grace violation
    let m5_policy = LifecyclePolicy {
        total_grace_period_seconds: 0,
        runtime_deadline_seconds: 0,
        sigkill_reserve_seconds: 0,
        min_hook_duration_seconds: 0,
        probe_timing: ProbeTiming::default(),
    };
    let m5_err = m5_policy
        .validate()
        .expect_err("Measurement 5 must fail with zero total grace reason");
    assert_eq!(m5_err, LifecyclePolicyError::ZeroTotalGrace);
    println!("Row 5 reason: {}", m5_err);
}

#[test]
fn probe_derivation_measurements() {
    let cx = test_cx();

    // Measurement 1: total 60, runtime 45, reserve 10, min hook 5, period 7, timeout 3, failure 4, success 1, port 9080
    let m1_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 7,
            timeout_seconds: 3,
            failure_threshold: 4,
            success_threshold: 1,
        },
    };
    let m1_budget = m1_policy.validate().expect("policy 1 must validate");
    let pod_template = test_pod_template(&cx).with_termination_budget(&m1_budget, 9080);
    let rendered = pod_template.render();

    let container = &rendered["spec"]["containers"][0];
    let liveness = &container["livenessProbe"];
    let readiness = &container["readinessProbe"];
    let startup = &container["startupProbe"];
    let grace = rendered["spec"]["terminationGracePeriodSeconds"].as_u64();

    println!(
        "Row 1 Rendered container: {}",
        serde_json::to_string_pretty(container).unwrap()
    );

    // Path assertions: string literals spelled out at assertion site (Row 1 requirement)
    assert_eq!(liveness["httpGet"]["path"], "/healthz");
    assert_eq!(readiness["httpGet"]["path"], "/readyz");
    assert_eq!(startup["httpGet"]["path"], "/readyz");

    // Port and timing assertions
    for probe in [liveness, readiness, startup] {
        assert_eq!(probe["httpGet"]["port"], 9080);
        assert_eq!(probe["periodSeconds"], 7);
        assert_eq!(probe["timeoutSeconds"], 3);
        assert_eq!(probe["failureThreshold"], 4);
    }

    // Pod grace assertion
    assert_eq!(grace, Some(60));

    // Measurement 2 (Round 3 Row 2): render at probe port 8443 while pod template containerPort is 9080
    let pod2_template = test_pod_template(&cx);
    assert_eq!(pod2_template.ports[0]["containerPort"], 9080);
    let rendered2 = pod2_template
        .with_termination_budget(&m1_budget, 8443)
        .render();
    let container2 = &rendered2["spec"]["containers"][0];

    println!(
        "Row 2 Rendered container (port 8443): {}",
        serde_json::to_string_pretty(container2).unwrap()
    );

    // Ensure template containerPort remains 9080
    assert_eq!(container2["ports"][0]["containerPort"], 9080);

    // All three probes must target 8443 and none target 9080
    for probe in [
        &container2["livenessProbe"],
        &container2["readinessProbe"],
        &container2["startupProbe"],
    ] {
        assert_eq!(probe["httpGet"]["port"], 8443);
        assert_ne!(probe["httpGet"]["port"], 9080);
    }

    // Success threshold check: success_threshold 2 -> startup & liveness are 1, readiness is 2
    let m2_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 7,
            timeout_seconds: 3,
            failure_threshold: 4,
            success_threshold: 2,
        },
    };
    let m2_budget = m2_policy.validate().expect("policy 2 must validate");
    let pod2_threshold = test_pod_template(&cx)
        .with_termination_budget(&m2_budget, 9080)
        .render();
    let c2_thresh = &pod2_threshold["spec"]["containers"][0];
    assert_eq!(c2_thresh["livenessProbe"]["successThreshold"], 1);
    assert_eq!(c2_thresh["startupProbe"]["successThreshold"], 1);
    assert_eq!(c2_thresh["readinessProbe"]["successThreshold"], 2);

    // Initial delay check: none of the three rendered probes contains initialDelaySeconds
    for probe in [
        &container["livenessProbe"],
        &container["readinessProbe"],
        &container["startupProbe"],
    ] {
        assert!(
            probe.get("initialDelaySeconds").is_none(),
            "no probe may contain initialDelaySeconds"
        );
    }

    // Measurement 3 (Round 3 Row 3): period 1 / timeout 0 / failure 3 / success 1 -> fails carrying timeout 0
    let m3_timing_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 1,
            timeout_seconds: 0,
            failure_threshold: 3,
            success_threshold: 1,
        },
    };
    let m3_timing_err = m3_timing_policy
        .validate()
        .expect_err("timeout 0 must fail validation");
    let m3_timing_expected = LifecyclePolicyError::InvalidProbeTiming {
        period_seconds: 1,
        timeout_seconds: 0,
        failure_threshold: 3,
        success_threshold: 1,
    };
    assert_eq!(m3_timing_err, m3_timing_expected);
    if let LifecyclePolicyError::InvalidProbeTiming {
        period_seconds,
        timeout_seconds,
        failure_threshold,
        success_threshold,
    } = m3_timing_err
    {
        assert_eq!(period_seconds, 1);
        assert_eq!(timeout_seconds, 0);
        assert_eq!(failure_threshold, 3);
        assert_eq!(success_threshold, 1);
    } else {
        panic!("expected InvalidProbeTiming");
    }
    println!("Row 3 probe timing error (timeout 0): {}", m3_timing_err);

    // Measurement 4 (Round 3 Row 4): period 1 / timeout 1 / failure 0 / success 1 -> fails carrying failure 0
    let m4_timing_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 1,
            timeout_seconds: 1,
            failure_threshold: 0,
            success_threshold: 1,
        },
    };
    let m4_timing_err = m4_timing_policy
        .validate()
        .expect_err("failure 0 must fail validation");
    let m4_timing_expected = LifecyclePolicyError::InvalidProbeTiming {
        period_seconds: 1,
        timeout_seconds: 1,
        failure_threshold: 0,
        success_threshold: 1,
    };
    assert_eq!(m4_timing_err, m4_timing_expected);
    if let LifecyclePolicyError::InvalidProbeTiming {
        period_seconds,
        timeout_seconds,
        failure_threshold,
        success_threshold,
    } = m4_timing_err
    {
        assert_eq!(period_seconds, 1);
        assert_eq!(timeout_seconds, 1);
        assert_eq!(failure_threshold, 0);
        assert_eq!(success_threshold, 1);
    } else {
        panic!("expected InvalidProbeTiming");
    }
    println!("Row 4 probe timing error (failure 0): {}", m4_timing_err);

    // Measurement 5 (Round 3 Row 5): period 1 / timeout 1 / failure 3 / success 0 -> fails carrying success 0
    let m5_timing_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 1,
            timeout_seconds: 1,
            failure_threshold: 3,
            success_threshold: 0,
        },
    };
    let m5_timing_err = m5_timing_policy
        .validate()
        .expect_err("success 0 must fail validation");
    let m5_timing_expected = LifecyclePolicyError::InvalidProbeTiming {
        period_seconds: 1,
        timeout_seconds: 1,
        failure_threshold: 3,
        success_threshold: 0,
    };
    assert_eq!(m5_timing_err, m5_timing_expected);
    if let LifecyclePolicyError::InvalidProbeTiming {
        period_seconds,
        timeout_seconds,
        failure_threshold,
        success_threshold,
    } = m5_timing_err
    {
        assert_eq!(period_seconds, 1);
        assert_eq!(timeout_seconds, 1);
        assert_eq!(failure_threshold, 3);
        assert_eq!(success_threshold, 0);
    } else {
        panic!("expected InvalidProbeTiming");
    }
    println!("Row 5 probe timing error (success 0): {}", m5_timing_err);

    // Zero-all timing test: period 0 / timeout 0 / failure 0 / success 0
    let m4_zero_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 0,
            timeout_seconds: 0,
            failure_threshold: 0,
            success_threshold: 0,
        },
    };
    let m4_zero_err = m4_zero_policy
        .validate()
        .expect_err("zero probe timing must fail");
    let m4_zero_expected = LifecyclePolicyError::InvalidProbeTiming {
        period_seconds: 0,
        timeout_seconds: 0,
        failure_threshold: 0,
        success_threshold: 0,
    };
    assert_eq!(m4_zero_err, m4_zero_expected);

    // Ensure m4_zero_err is distinct from other policy error types
    let zero_grace_err = LifecyclePolicyError::ZeroTotalGrace;
    let overflow_err = LifecyclePolicyError::Overflow;
    let budget_exceeds_err = LifecyclePolicyError::BudgetExceedsTotal {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 55,
        sigkill_reserve_seconds: 10,
    };
    let min_hook_err = LifecyclePolicyError::RuntimeBelowMinimumHook {
        runtime_deadline_seconds: 3,
        min_hook_duration_seconds: 5,
    };

    assert_ne!(m4_zero_err, zero_grace_err);
    assert_ne!(m4_zero_err, overflow_err);
    assert_ne!(m4_zero_err, budget_exceeds_err);
    assert_ne!(m4_zero_err, min_hook_err);

    assert_ne!(m4_zero_err.to_string(), zero_grace_err.to_string());
    assert_ne!(m4_zero_err.to_string(), overflow_err.to_string());
    assert_ne!(m4_zero_err.to_string(), budget_exceeds_err.to_string());
    assert_ne!(m4_zero_err.to_string(), min_hook_err.to_string());

    // Measurement 6 (Round 3 Row 6): period 1 / timeout 1 / failure 1 / success 1 -> succeeds at K8s minimum
    let m6_min_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 45,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming {
            period_seconds: 1,
            timeout_seconds: 1,
            failure_threshold: 1,
            success_threshold: 1,
        },
    };
    let m6_min_budget = m6_min_policy
        .validate()
        .expect("minimum probe timing 1 must validate");
    let pod6 = test_pod_template(&cx)
        .with_termination_budget(&m6_min_budget, 9080)
        .render();
    let container6 = &pod6["spec"]["containers"][0];

    println!(
        "Row 6 Rendered container (minimum timing 1): {}",
        serde_json::to_string_pretty(container6).unwrap()
    );

    for probe in [
        &container6["livenessProbe"],
        &container6["readinessProbe"],
        &container6["startupProbe"],
    ] {
        assert_eq!(probe["periodSeconds"], 1);
        assert_eq!(probe["timeoutSeconds"], 1);
        assert_eq!(probe["failureThreshold"], 1);
    }

    // Boundary total check: total 60 / runtime 50 / reserve 10 / min hook 5 (runtime + reserve == total) -> succeeds
    let m5_boundary_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 50,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m5_boundary_budget = m5_boundary_policy
        .validate()
        .expect("runtime + reserve == total must succeed on boundary");
    assert_eq!(m5_boundary_budget.runtime_deadline_seconds(), 50);
    assert_eq!(m5_boundary_budget.sigkill_reserve_seconds(), 10);
}

#[test]
fn try_from_conversions() {
    let policy = LifecyclePolicy::default();
    let budget_ref = TerminationBudget::try_from(&policy).expect("valid ref conversion");
    let budget_val = TerminationBudget::try_from(policy).expect("valid val conversion");
    assert_eq!(budget_ref, budget_val);
}

#[test]
fn crd_schema_derivation_is_crd_safe() {
    let mut schema = serde_json::to_value(schemars::schema_for!(LifecyclePolicy))
        .expect("schema derivation should succeed");
    normalize_unsigned_integer_formats(&mut schema);

    let schema_str = serde_json::to_string(&schema).expect("json stringify");
    assert!(
        !schema_str.contains("\"uint64\""),
        "uint64 format should be stripped"
    );
    assert!(
        !schema_str.contains("\"uint32\""),
        "uint32 format should be stripped"
    );
}
