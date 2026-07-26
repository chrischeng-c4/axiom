"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-self-ec-fixture-loop-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "self-ec-fixture-loop-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_self_ec_fixture_loop_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_required_ec_refuses_red_then_records_green_terminal_completion -- --exact --nocapture"
ASSERTIONS = ('a configured required EC case refuses the unchanged CB-filled WorkItem while red without phase or close mutation, then permits terminal close only when green and records the consulted case in the success envelope',)
