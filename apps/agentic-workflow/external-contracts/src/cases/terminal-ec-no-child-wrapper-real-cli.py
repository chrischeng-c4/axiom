"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-no-child-wrapper-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_no_child_wrapper_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests test_code_check_bounds_no_child_ec_wrapper_and_preserves_phase -- --nocapture"
ASSERTIONS = ('the real aw binary returns within the configured one-second deadline plus bounded cleanup grace', 'the helper confirms its external child exited before the wrapper timed out', 'the wrapper PID no longer exists after aw returns', 'the envelope has terminal_ec_timeout and exact aw cb check slug next.command', 'the work item remains open in cb_filled and no terminal commit is created')
