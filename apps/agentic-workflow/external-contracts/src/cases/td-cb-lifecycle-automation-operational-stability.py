"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-stability"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-lifecycle-dispatch"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib td_branch_activation_only_uses_main -- --nocapture"
ASSERTIONS = ("td-cb-lifecycle-automation reports identical non-zero passed and zero-failed totals across two executions",)
