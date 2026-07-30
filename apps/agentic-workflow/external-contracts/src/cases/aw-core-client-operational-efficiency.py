"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "aw-core-client-operational-efficiency"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-core-client-operational-efficiency"
DIMENSION = "efficiency"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-operational-efficiency"
)
ASSERTIONS = ("the native Python prompt and runner scenario completes within 120 seconds with representative assertions and no cargo delegation",)


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
