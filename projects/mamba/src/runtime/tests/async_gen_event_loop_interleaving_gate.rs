//! Inline migration of tests/async_gen_event_loop_interleaving_gate_fixture_1255.rs (#1255).
//!
//! Locks the shape of the C5 async-generator real-event-loop interleaving fixture
//! pinned by tests/cpython/core/async/async_gen_event_loop_interleaving_gate/manifest.toml.

#![cfg(test)]

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        format!("{}/core/async/async_gen_event_loop_interleaving_gate/manifest.toml", crate::conformance::FIXTURES_ROOT),
    )
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(
        m["fixture"].as_str(),
        Some("async_gen_event_loop_interleaving_gate")
    );
    assert_eq!(m["issue"].as_integer(), Some(1255));
    assert_eq!(m["profile"].as_str(), Some("conformance"));
    assert_eq!(
        m["family"].as_str(),
        Some("async_gen_event_loop_interleaving_gate")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
    let related: Vec<_> = m["related_issues"]
        .as_array()
        .expect("related_issues")
        .iter()
        .map(|v| v.as_integer().expect("int"))
        .collect();
    assert_eq!(related, vec![850, 1265]);
}

#[test]
fn isolation_pins_no_global_state() {
    let iso = &manifest()["isolation"];
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(iso[key].as_bool(), Some(true), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(12));
    assert_eq!(py["must_be_python_3_12"].as_bool(), Some(true));
}

#[test]
fn surface_pins_distinct_entity_and_interleave() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_distinct_mb_async_generator_entity",
        "must_cover_anext_returns_coroutine_awaitable",
        "must_cover_gather_interleaves_n_async_gens",
        "must_cover_single_async_gen_conformance_stays_green",
        "must_cover_new_concurrent_fixture_under_async_dir",
        "must_be_offline",
        "must_be_deterministic_modulo_timing",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn r1_mb_async_generator_is_distinct_runtime_entity() {
    let c = &manifest()["r1_distinct_mb_async_generator_entity_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("mb_async_generator_is_distinct_runtime_entity_from_mb_generator")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R1"));
    for key in [
        "must_define_mb_async_generator_runtime_entity",
        "must_keep_mb_async_generator_distinct_from_mb_generator",
        "must_yield_suspended_frame_event_loop_can_resume",
        "forbid_aliasing_async_gen_to_sync_gen",
        "forbid_routing_async_gen_through_synchronous_iteration_for_multi_gen",
        "must_distinguish_aliased_from_routed_through_sync_iteration",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["async_gen_runtime_entity_field_name"].as_str(),
        Some("async_gen_runtime_entity")
    );
    assert_eq!(
        c["expected_async_gen_runtime_entity"].as_str(),
        Some("MbAsyncGenerator")
    );
    assert_eq!(
        c["generator_runtime_entity_field_name"].as_str(),
        Some("generator_runtime_entity")
    );
    assert_eq!(
        c["expected_generator_runtime_entity"].as_str(),
        Some("MbGenerator")
    );
    assert_eq!(
        c["async_gen_aliased_to_sync_gen_failure_kind"].as_str(),
        Some("async_gen_runtime_aliased_to_sync_generator")
    );
    assert_eq!(
        c["async_gen_aliased_to_sync_gen_exit_code"].as_integer(),
        Some(303)
    );
    assert_eq!(
        c["async_gen_routed_through_sync_iteration_failure_kind"].as_str(),
        Some("async_gen_routed_through_sync_iteration_for_multi_gen")
    );
    assert_eq!(
        c["async_gen_routed_through_sync_iteration_exit_code"].as_integer(),
        Some(304)
    );
}

#[test]
fn r2_anext_returns_coroutine_awaitable() {
    let c = &manifest()["r2_anext_returns_coroutine_awaitable_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("dunder_anext_returns_coroutine_awaitable_not_synchronous_result")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R2"));
    for key in [
        "must_return_coroutine_awaitable_from_anext",
        "must_allow_asyncio_to_schedule_across_multiple_async_gens",
        "forbid_anext_returning_synchronous_result",
        "forbid_anext_returning_plain_value_object",
        "must_distinguish_anext_sync_result_from_plain_value",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["anext_return_kind_field_name"].as_str(),
        Some("anext_return_kind")
    );
    let allowed: Vec<_> = c["allowed_anext_return_kinds"]
        .as_array()
        .expect("allowed_anext_return_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(allowed, vec!["coroutine", "awaitable"]);
    let disallowed: Vec<_> = c["disallowed_anext_return_kinds"]
        .as_array()
        .expect("disallowed_anext_return_kinds")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(disallowed, vec!["sync_result", "plain_value"]);
    assert_eq!(
        c["anext_returned_sync_result_failure_kind"].as_str(),
        Some("async_gen_anext_returned_sync_result")
    );
    assert_eq!(
        c["anext_returned_sync_result_exit_code"].as_integer(),
        Some(305)
    );
    assert_eq!(
        c["anext_returned_plain_value_failure_kind"].as_str(),
        Some("async_gen_anext_returned_plain_value_object")
    );
    assert_eq!(
        c["anext_returned_plain_value_exit_code"].as_integer(),
        Some(306)
    );
}

#[test]
fn r3_gather_interleaves_n_async_gens() {
    let c = &manifest()["r3_gather_interleaves_n_async_gens_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("asyncio_gather_of_n_async_gens_interleaves_yields")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R3"));
    for key in [
        "must_use_asyncio_gather_to_collect_n_async_gens",
        "must_observe_interleaved_yield_sequence",
        "must_use_timing_based_verification",
        "forbid_drain_a_before_b_starts",
        "forbid_silently_serializing_async_gens",
        "must_distinguish_drain_before_other_from_serialized_silently",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["n_async_gens_field_name"].as_str(),
        Some("n_async_gens")
    );
    assert_eq!(c["expected_min_n_async_gens"].as_integer(), Some(2));
    assert_eq!(
        c["yield_sequence_field_name"].as_str(),
        Some("yield_sequence")
    );
    assert_eq!(
        c["interleave_observed_field_name"].as_str(),
        Some("interleave_observed")
    );
    assert_eq!(
        c["drain_before_other_failure_kind"].as_str(),
        Some("async_gen_gather_drained_a_before_b_started")
    );
    assert_eq!(
        c["drain_before_other_exit_code"].as_integer(),
        Some(307)
    );
    assert_eq!(
        c["serialized_silently_failure_kind"].as_str(),
        Some("async_gen_gather_serialized_silently")
    );
    assert_eq!(
        c["serialized_silently_exit_code"].as_integer(),
        Some(308)
    );
}

#[test]
fn r4_single_async_gen_conformance_stays_green() {
    let c = &manifest()["r4_single_async_gen_conformance_stays_green_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("single_async_gen_conformance_stays_green_at_583_of_583")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R4"));
    for key in [
        "must_keep_single_async_gen_conformance_passing",
        "forbid_silently_relaxing_single_async_gen_baseline",
        "must_distinguish_regression_from_baseline_relaxed",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["required_single_async_gen_pass_count"].as_integer(),
        Some(583)
    );
    assert_eq!(
        c["required_single_async_gen_total_count"].as_integer(),
        Some(583)
    );
    assert_eq!(
        c["single_async_gen_pass_count_field_name"].as_str(),
        Some("single_async_gen_pass_count")
    );
    assert_eq!(
        c["single_async_gen_total_count_field_name"].as_str(),
        Some("single_async_gen_total_count")
    );
    assert_eq!(
        c["single_async_gen_regression_failure_kind"].as_str(),
        Some("async_gen_single_gen_conformance_regression")
    );
    assert_eq!(
        c["single_async_gen_regression_exit_code"].as_integer(),
        Some(309)
    );
    assert_eq!(
        c["single_async_gen_baseline_relaxed_failure_kind"].as_str(),
        Some("async_gen_single_gen_conformance_baseline_relaxed")
    );
    assert_eq!(
        c["single_async_gen_baseline_relaxed_exit_code"].as_integer(),
        Some(310)
    );
}

#[test]
fn r5_concurrent_fixture_pins_interleave_behavior() {
    let c = &manifest()["r5_concurrent_fixture_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("concurrent_async_gen_fixture_pins_interleave_behavior")
    );
    assert_eq!(c["requirement_id"].as_str(), Some("R5"));
    for key in [
        "must_provide_concurrent_async_gen_fixture",
        "must_pin_concurrent_async_gen_fixture_relative_path",
        "forbid_concurrent_async_gen_fixture_being_skipped",
        "forbid_concurrent_async_gen_fixture_silently_classified_as_single_gen",
        "must_distinguish_concurrent_fixture_missing_from_classified_as_single_gen",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["concurrent_fixture_relative_path"].as_str(),
        Some("projects/mamba/tests/cpython/core/async/async_gen_concurrent")
    );
    assert_eq!(
        c["concurrent_fixture_relative_path_field_name"].as_str(),
        Some("concurrent_fixture_relative_path")
    );
    assert_eq!(
        c["concurrent_fixture_missing_failure_kind"].as_str(),
        Some("async_gen_concurrent_fixture_missing")
    );
    assert_eq!(
        c["concurrent_fixture_missing_exit_code"].as_integer(),
        Some(311)
    );
    assert_eq!(
        c["concurrent_fixture_classified_as_single_gen_failure_kind"].as_str(),
        Some("async_gen_concurrent_fixture_classified_as_single_gen")
    );
    assert_eq!(
        c["concurrent_fixture_classified_as_single_gen_exit_code"].as_integer(),
        Some(312)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let r = &manifest()["runner_contract"];
    let keys: Vec<_> = r["keys"]
        .as_array()
        .expect("keys")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "outcome",
            "case",
            "requirement_id",
            "async_gen_runtime_entity",
            "generator_runtime_entity",
            "anext_return_kind",
            "n_async_gens",
            "yield_sequence",
            "interleave_observed",
            "single_async_gen_pass_count",
            "single_async_gen_total_count",
            "concurrent_fixture_relative_path",
            "failure_kind",
            "exit_code",
        ]
    );
    let outcomes: Vec<_> = r["outcome_values"]
        .as_array()
        .expect("outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "missing", "skip"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "mb_async_generator_is_distinct_runtime_entity_from_mb_generator",
            "dunder_anext_returns_coroutine_awaitable_not_synchronous_result",
            "asyncio_gather_of_n_async_gens_interleaves_yields",
            "single_async_gen_conformance_stays_green_at_583_of_583",
            "concurrent_async_gen_fixture_pins_interleave_behavior",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    for key in [
        "rewriting_mb_generator",
        "async_io_backends_beyond_asyncio",
        "performance_gates",
        "c_extension_fast_paths",
        "runtime_implementation_of_mb_async_generator",
        "runtime_implementation_of_asyncio_event_loop",
    ] {
        assert_eq!(o[key].as_bool(), Some(true), "out_of_scope.{key}");
    }
}
