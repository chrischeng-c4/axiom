"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-stability"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "existing-project-standardization-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-operational-stability"
)
ASSERTIONS = ("two fresh native Python existing-project health scenarios preserve stable readiness axes and gate identities",)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
