"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "manual-evidence-artifacts-operational-efficiency"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory -- --nocapture"
ASSERTIONS = ("manual-evidence-artifacts completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
