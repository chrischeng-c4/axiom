"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-default-section-queue-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-default-section-queue-preservation"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_default_section_queue_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_create_replay_does_not_clobber_authored_logic_section -- --nocapture"
ASSERTIONS = ('the fresh skeleton contains logic followed by unit-test', 'logic applicability emits an applicability unit-test dispatch', 'contract authoring does not start before unit-test applicability')
