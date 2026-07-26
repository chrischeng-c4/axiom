"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "work-item-planning-operational-efficiency"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-to-change-atomization"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case work-item-planning-operational-efficiency"
ASSERTIONS = ("work-item-planning completes its representative gate within 120 seconds with at least one passed and zero failed tests",)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
