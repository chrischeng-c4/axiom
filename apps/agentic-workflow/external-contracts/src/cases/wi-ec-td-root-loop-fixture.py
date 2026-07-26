"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-ec-td-root-loop-fixture"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "wi-ec-td-root-loop"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_ec_td_root_loop_fixture.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_drives_wi_run_to_workflow_complete -- --exact --nocapture"
ASSERTIONS = ('fixture root follows emitted commands until completion.workflow_complete=true', 'no retired lifecycle command or hidden agent-only step is required')
