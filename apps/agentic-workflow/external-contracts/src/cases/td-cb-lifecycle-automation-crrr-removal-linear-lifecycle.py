"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-crrr-removal-linear-lifecycle"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "crrr-removal-linear-lifecycle"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_crrr_removal_linear_lifecycle.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-crrr-removal-linear-lifecycle"
ASSERTIONS = ('TD created phase dispatches directly to generation in the linear lifecycle',)


def verify() -> list[str]:
    from migration_clusters.td_lifecycle import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
