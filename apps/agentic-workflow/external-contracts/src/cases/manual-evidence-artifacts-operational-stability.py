"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "manual-evidence-artifacts-operational-stability"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory -- --nocapture"
ASSERTIONS = ("manual-evidence-artifacts reports identical non-zero passed and zero-failed totals across two executions",)
