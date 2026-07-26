"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-health-policy"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_health_policy.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-health-policy"
ASSERTIONS = ('health pins policy_mode, required_trailer, root_runner_allowed, direct_repair_default, and exact complete ordered hard_gates and advisory_axes arrays', 'health never emits any aw goal command as remediation')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
