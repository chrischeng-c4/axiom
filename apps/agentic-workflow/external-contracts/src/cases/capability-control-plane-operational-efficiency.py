"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "capability-control-plane-operational-efficiency"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "markdown-capability-schema"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib markdown_capability_tables -- --nocapture"
ASSERTIONS = ("capability-control-plane completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
