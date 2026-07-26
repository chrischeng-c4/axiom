"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "capability-control-plane-operational-stability"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "markdown-capability-schema"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib markdown_capability_tables -- --nocapture"
ASSERTIONS = ("capability-control-plane reports identical non-zero passed and zero-failed totals across two executions",)
