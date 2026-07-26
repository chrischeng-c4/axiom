"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-cross-process-single-flight-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_cross_process_single_flight_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests test_code_check_cross_process_single_flight_prevents_duplicate_ec_launch -- --nocapture"
ASSERTIONS = ('the first aw process owns the project lock while its EC command runs', 'the second same-slug aw process returns terminal_ec_single_flight promptly', 'both refusal envelopes point to exact aw cb check slug retry commands', 'the append-only EC launch marker contains exactly one line', 'the work item remains open in cb_filled and no terminal commit is created')
