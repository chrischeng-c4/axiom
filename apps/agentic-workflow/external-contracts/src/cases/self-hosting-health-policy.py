"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-health-policy"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_health_policy.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_health_reports_policy_and_never_points_back_to_root_runner -- --nocapture"
ASSERTIONS = ('health pins policy_mode, required_trailer, root_runner_allowed, direct_repair_default, and exact complete ordered hard_gates and advisory_axes arrays', 'health never emits any aw goal command as remediation')
