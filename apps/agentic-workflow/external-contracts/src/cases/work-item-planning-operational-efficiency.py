"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "work-item-planning-operational-efficiency"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-to-change-atomization"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now -- --nocapture"
ASSERTIONS = ("work-item-planning completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
