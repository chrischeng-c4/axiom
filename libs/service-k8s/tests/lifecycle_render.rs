use serde_json::json;
use service_k8s::crd::normalize_unsigned_integer_formats;
use service_k8s::lifecycle::{
    LifecyclePolicy, LifecyclePolicyError, ProbeTiming, TerminationBudget, HEALTH_ENDPOINT_PATH,
    READY_ENDPOINT_PATH,
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
        "Rendered container: {}",
        serde_json::to_string_pretty(container).unwrap()
    );

    // Path assertions
    assert_eq!(liveness["httpGet"]["path"], HEALTH_ENDPOINT_PATH);
    assert_eq!(readiness["httpGet"]["path"], READY_ENDPOINT_PATH);
    assert_eq!(startup["httpGet"]["path"], READY_ENDPOINT_PATH);

    // Port and timing assertions
    for probe in [liveness, readiness, startup] {
        assert_eq!(probe["httpGet"]["port"], 9080);
        assert_eq!(probe["periodSeconds"], 7);
        assert_eq!(probe["timeoutSeconds"], 3);
        assert_eq!(probe["failureThreshold"], 4);
    }

    // Pod grace assertion
    assert_eq!(grace, Some(60));

    // Measurement 2: success_threshold 2 -> startup & liveness are 1, readiness is 2
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
    let pod2 = test_pod_template(&cx)
        .with_termination_budget(&m2_budget, 9080)
        .render();
    let c2 = &pod2["spec"]["containers"][0];
    assert_eq!(c2["livenessProbe"]["successThreshold"], 1);
    assert_eq!(c2["startupProbe"]["successThreshold"], 1);
    assert_eq!(c2["readinessProbe"]["successThreshold"], 2);

    // Measurement 3: none of the three rendered probes contains initialDelaySeconds
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

    // Measurement 4: probe timing period 0 / timeout 0 / failure 0 / success 0 -> validation fails with distinct reason
    let m4_policy = LifecyclePolicy {
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
    let m4_err = m4_policy
        .validate()
        .expect_err("zero probe timing must fail");
    let m4_expected = LifecyclePolicyError::InvalidProbeTiming {
        period_seconds: 0,
        timeout_seconds: 0,
        failure_threshold: 0,
        success_threshold: 0,
    };
    assert_eq!(m4_err, m4_expected);
    println!("Row 4 probe timing error reason: {}", m4_err);

    // Ensure m4_err is distinct from zero total grace, budget-exceeds-total, runtime-below-min-hook, overflow
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

    assert_ne!(m4_err, zero_grace_err);
    assert_ne!(m4_err, overflow_err);
    assert_ne!(m4_err, budget_exceeds_err);
    assert_ne!(m4_err, min_hook_err);

    assert_ne!(m4_err.to_string(), zero_grace_err.to_string());
    assert_ne!(m4_err.to_string(), overflow_err.to_string());
    assert_ne!(m4_err.to_string(), budget_exceeds_err.to_string());
    assert_ne!(m4_err.to_string(), min_hook_err.to_string());

    // Measurement 5: total 60 / runtime 50 / reserve 10 / min hook 5 (runtime + reserve == total) -> succeeds
    let m5_policy = LifecyclePolicy {
        total_grace_period_seconds: 60,
        runtime_deadline_seconds: 50,
        sigkill_reserve_seconds: 10,
        min_hook_duration_seconds: 5,
        probe_timing: ProbeTiming::default(),
    };
    let m5_budget = m5_policy
        .validate()
        .expect("runtime + reserve == total must succeed on boundary");
    assert_eq!(m5_budget.runtime_deadline_seconds(), 50);
    assert_eq!(m5_budget.sigkill_reserve_seconds(), 10);
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
