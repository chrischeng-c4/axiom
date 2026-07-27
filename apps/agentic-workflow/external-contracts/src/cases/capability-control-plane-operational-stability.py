"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "capability-control-plane-operational-stability"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-control-plane-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case capability-control-plane-operational-stability"
ASSERTIONS = ("two fresh native Python capability scenarios preserve the same project identity and canonical Markdown contract",)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
