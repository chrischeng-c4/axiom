"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "project-local-td-and-ec-gates-operational-efficiency"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "project-local-td-and-ec-gates-operational-efficiency"
DIMENSION = "efficiency"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case project-local-td-and-ec-gates-operational-efficiency"
)
ASSERTIONS = ("the native Python project-local TD and EC producer scenario completes within 120 seconds without cargo delegation",)


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
