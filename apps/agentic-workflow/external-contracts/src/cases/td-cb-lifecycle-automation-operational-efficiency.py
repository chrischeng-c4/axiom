"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-efficiency"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-lifecycle-dispatch"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib td_branch_activation_only_uses_main -- --nocapture"
ASSERTIONS = ("td-cb-lifecycle-automation completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
