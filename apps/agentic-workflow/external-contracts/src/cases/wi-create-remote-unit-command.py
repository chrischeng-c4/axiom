"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-remote-unit-command"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_remote_unit_command.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib wi_create_remote -- --nocapture"
ASSERTIONS = ('help hides deprecated remote flag', 'compatibility flag parses', 'backend decision is config-driven')
