//! Consolidated umbrella binary for 15 MVP-profile / release-gate runner
//! contracts (Phase 6 of the tests/ DDD refactor). Each former binary
//! pinned the contract of one helper script under `scripts/` — e.g.
//! `gate0_list_tests.py`, `perf_floor_check.py`, `release_baseline.py`,
//! `inventory_summary.py` — by spawning `python3` against the script and
//! a manifest fixture and asserting the script's behavior.
//!
//! They are honest integration binaries (each spawns `python3` or
//! `cargo`) so they already pass the Phase 5 orphan-lint predicate.
//! Consolidating them is a link-time optimization: one binary instead
//! of 15, ~188 `#[test]` functions in one umbrella.
//!
//! Each former `tests/<runner>.rs` now lives in `tests/mvp_runners/`
//! and is registered below with `#[path]`.
//!
//! Selector: `cargo test -p mamba --test mvp_runners`.

#[path = "common.rs"]
mod common;

#[path = "mvp_gates/gate0_list_wrapper_2534.rs"]
mod gate0_list_wrapper_2534;

#[path = "mvp_gates/inventory_summary_2537.rs"]
mod inventory_summary_2537;

#[path = "mvp_gates/mvp_baseline_tier_metadata_2566.rs"]
mod mvp_baseline_tier_metadata_2566;

#[path = "mvp_gates/mvp_cpython_identity_2572.rs"]
mod mvp_cpython_identity_2572;

#[path = "mvp_gates/mvp_perf_bench_header_2571.rs"]
mod mvp_perf_bench_header_2571;

#[path = "mvp_gates/mvp_perf_benchmark_manifest_2567.rs"]
mod mvp_perf_benchmark_manifest_2567;

#[path = "mvp_gates/mvp_perf_floor_checker_2565.rs"]
mod mvp_perf_floor_checker_2565;

#[path = "mvp_gates/mvp_perf_gate_summary_2573.rs"]
mod mvp_perf_gate_summary_2573;

#[path = "mvp_gates/mvp_perf_internal_time_2570.rs"]
mod mvp_perf_internal_time_2570;

#[path = "mvp_gates/mvp_perf_suite_geomean_2569.rs"]
mod mvp_perf_suite_geomean_2569;

#[path = "mvp_gates/mvp_release_baseline_policy_2823.rs"]
mod mvp_release_baseline_policy_2823;

#[path = "mvp_gates/mvp_release_blocker_budget_2822.rs"]
mod mvp_release_blocker_budget_2822;

#[path = "mvp_gates/mvp_release_flaky_quarantine_2824.rs"]
mod mvp_release_flaky_quarantine_2824;

#[path = "mvp_gates/mvp_release_gate_runner_2821.rs"]
mod mvp_release_gate_runner_2821;

#[path = "mvp_gates/mvp_release_skip_policy_2825.rs"]
mod mvp_release_skip_policy_2825;
