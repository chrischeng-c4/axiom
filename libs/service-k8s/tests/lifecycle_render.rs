use service_k8s::crd::normalize_unsigned_integer_formats;
use service_k8s::lifecycle::{
    LifecyclePolicy, LifecyclePolicyError, ProbeTiming, TerminationBudget,
};

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
