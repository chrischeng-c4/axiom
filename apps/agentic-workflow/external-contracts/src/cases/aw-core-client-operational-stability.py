"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "aw-core-client-operational-stability"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-operational-stability"
ASSERTIONS = ("aw-core-client-model-workitem-first-artifact-lifecycle reports identical non-zero passed and zero-failed totals across two executions",)


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
