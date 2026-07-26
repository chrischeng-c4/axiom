"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-agent-first-cli-product-model"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_agent_first_cli_product_model.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-agent-first-cli-product-model"
ASSERTIONS = ('binary orientation, README, capability contract, and canonical product TDs share one agent-first CLI model and reject removed architecture prose',)


def verify() -> list[str]:
    from migration_clusters.core_model import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
