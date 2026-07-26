"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-efficiency"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-lifecycle-dispatch"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-operational-efficiency"
ASSERTIONS = ("td-cb-lifecycle-automation completes its representative gate within 120 seconds with at least one passed and zero failed tests",)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
