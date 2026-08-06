"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-efficiency"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "existing-project-standardization-operational-efficiency"
DIMENSION = "efficiency"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-operational-efficiency"
)
ASSERTIONS = ("the native Python existing-project health scenario completes within 120 seconds without cargo delegation",)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
