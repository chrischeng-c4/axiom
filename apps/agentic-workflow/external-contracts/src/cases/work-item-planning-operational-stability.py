"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "work-item-planning-operational-stability"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-to-change-atomization"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now -- --nocapture"
ASSERTIONS = ("work-item-planning reports identical non-zero passed and zero-failed totals across two executions",)
