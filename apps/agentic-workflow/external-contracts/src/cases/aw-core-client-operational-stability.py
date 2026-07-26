"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "aw-core-client-operational-stability"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib agent_first_product_contracts_reject_removed_architecture -- --nocapture"
ASSERTIONS = ("aw-core-client-model-workitem-first-artifact-lifecycle reports identical non-zero passed and zero-failed totals across two executions",)
