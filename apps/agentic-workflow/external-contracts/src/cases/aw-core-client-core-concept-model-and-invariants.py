"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-core-concept-model-and-invariants"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "core-concept-model-and-invariants"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_core_concept_model_and_invariants.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_drives_wi_run_to_workflow_complete -- --exact --nocapture"
ASSERTIONS = ('from an admitted CB-generated child, the real compiled goal runner follows emitted CB commands, closes the child, rolls up its epic and capability, and terminates with completion.workflow_complete=true',)
