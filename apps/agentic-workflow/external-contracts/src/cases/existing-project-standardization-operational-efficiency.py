"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-efficiency"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case existing-project-standardization-operational-efficiency"
ASSERTIONS = ("existing-project-standardization completes its representative gate within 120 seconds with at least one passed and zero failed tests",)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
