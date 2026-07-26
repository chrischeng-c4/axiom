"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "aw-core-client-operational-efficiency"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-first-cli-product-model"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib agent_first_product_contracts_reject_removed_architecture -- --nocapture"
ASSERTIONS = ("aw-core-client-model-workitem-first-artifact-lifecycle completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
