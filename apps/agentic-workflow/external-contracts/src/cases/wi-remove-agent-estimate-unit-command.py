"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-remove-agent-estimate-unit-command"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-remove-agent-estimate-unit-command"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_remove_agent_estimate_unit_command.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib wi_remove_agent_estimate -- --nocapture"
ASSERTIONS = ('bounded non-epic validates without estimate section', 'legacy estimate section remains parseable', 'generated body and planning output omit estimate fields')
