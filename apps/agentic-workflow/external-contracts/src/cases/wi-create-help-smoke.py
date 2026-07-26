"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-help-smoke"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-create-help-smoke"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_help_smoke.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib wi_create_remote_help_hides_deprecated_remote_flag -- --nocapture"
ASSERTIONS = ('stdout does not contain --remote',)
