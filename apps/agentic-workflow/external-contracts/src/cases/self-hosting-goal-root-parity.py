"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-goal-root-parity"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_goal_root_parity.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-goal-root-parity"
ASSERTIONS = ('self-AW WI identity resolution and rollup routing reject before loop-state or dispatch touch the fixture tree',)


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
