"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-agent-first-cli-product-model"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_agent_first_cli_product_model.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib agent_first_product_contracts_reject_removed_architecture -- --nocapture"
ASSERTIONS = ('binary orientation, README, capability contract, and canonical product TDs share one agent-first CLI model and reject removed architecture prose',)
