"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-efficiency"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-cb-lifecycle-automation-operational-efficiency"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-operational-efficiency"
ASSERTIONS = ("the native Python red-to-green terminal EC scenario completes within 120 seconds without cargo delegation",)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
