"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-stability"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case existing-project-standardization-operational-stability"
ASSERTIONS = ("existing-project-standardization reports identical non-zero passed and zero-failed totals across two executions",)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
