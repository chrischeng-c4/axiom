"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "completeness-placeholder-unit-command"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "cb-lifecycle-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_completeness_placeholder_unit_command.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib completeness_placeholder_scanner_contract -- --nocapture"
ASSERTIONS = ('placeholder code is rejected', 'omitted prose is rejected', 'future_work_allowed TODO is accepted')
