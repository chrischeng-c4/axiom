"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "capability-control-plane-operational-efficiency"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-control-plane-operational-efficiency"
DIMENSION = "efficiency"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-operational-efficiency"
)
ASSERTIONS = ("the native Python capability init, report, and sweep scenario completes within 120 seconds without cargo delegation",)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
