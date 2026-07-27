"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "work-item-planning-operational-stability"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "work-item-planning-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case work-item-planning-operational-stability"
ASSERTIONS = ("two fresh native Python planning scenarios produce identical non-empty results and pass every representative assertion",)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
