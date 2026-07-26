"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-ec-zero-test-false-green"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-false-green-guard"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_ec_zero_test_false_green.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_verify_rejects_zero_test_false_green -- --nocapture"
ASSERTIONS = ('aw ec verify marks a cargo test command failed when the command exits 0 after running zero tests', 'generated Rust EC wrappers capture stdout and reject the same zero-test false green', 'ec gen keeps precise cargo test target selectors instead of relying on crate-wide filters when the source contract provides one')
