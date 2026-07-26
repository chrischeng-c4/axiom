"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-retry-transition-lease-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_retry_transition_lease_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests test_code_check_retry_contends_while_terminal_transition_holds_lease -- --nocapture"
ASSERTIONS = ('a bounded debug-only barrier pauses the owner after td_merged is written while its lease remains held', 'the second process reads retry phase and promptly receives terminal_ec_single_flight', 'the refusal points to the exact same-slug aw cb check retry', 'after releasing the owner there is one EC launch and one Cb-CodeCheck terminal commit')
