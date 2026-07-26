"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-stability"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-lifecycle-dispatch"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-operational-stability"
ASSERTIONS = ("td-cb-lifecycle-automation reports identical non-zero passed and zero-failed totals across two executions",)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
