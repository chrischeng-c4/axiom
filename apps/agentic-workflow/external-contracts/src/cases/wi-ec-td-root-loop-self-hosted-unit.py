"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-ec-td-root-loop-self-hosted-unit"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "wi-ec-td-root-loop"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_ec_td_root_loop_self_hosted_unit.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_red_and_green_loop_states_route_to_adaptation_or_terminal_check -- --nocapture"
ASSERTIONS = ('red and green EC loop states expose exact bounded TD commands', 'a tracker-backed root has a local lifecycle ledger before EC transitions write next_action')
