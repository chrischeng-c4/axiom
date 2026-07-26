"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-remote-flag-tests"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-create-remote-flag-tests"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_remote_flag_tests.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib wi_create_remote -- --nocapture"
ASSERTIONS = ('create help hides remote flag', 'hidden remote compatibility flag parses', 'create behavior is config-driven')
