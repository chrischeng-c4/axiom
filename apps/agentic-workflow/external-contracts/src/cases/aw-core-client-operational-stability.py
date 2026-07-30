"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "aw-core-client-operational-stability"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-core-client-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-operational-stability"
)
ASSERTIONS = ("two fresh native Python prompt and runner scenarios preserve identical prompt documentation and typed lifecycle invariants",)


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
