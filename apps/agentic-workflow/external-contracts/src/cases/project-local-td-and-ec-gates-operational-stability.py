"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "project-local-td-and-ec-gates-operational-stability"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "project-local-td-and-ec-gates-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-local-td-and-ec-gates-operational-stability"
ASSERTIONS = ("two fresh native Python project-local TD and EC producer scenarios preserve artifact identities and routes",)


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
