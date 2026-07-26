"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "capability-control-plane-operational-efficiency"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "markdown-capability-schema"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case capability-control-plane-operational-efficiency"
ASSERTIONS = ("capability-control-plane completes its representative gate within 120 seconds with at least one passed and zero failed tests",)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
