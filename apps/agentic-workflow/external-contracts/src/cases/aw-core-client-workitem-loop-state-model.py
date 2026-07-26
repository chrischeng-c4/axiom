"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-workitem-loop-state-model"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-loop-state-model"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_workitem_loop_state_model.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib loop_state_round_trips -- --nocapture"
ASSERTIONS = ('work-item loop state serializes and parses losslessly',)
