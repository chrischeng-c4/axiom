"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-core-concept-model-phase-less-admission"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "core-concept-model-and-invariants"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_core_concept_model_phase_less_admission.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::phase_less_project_wi_enters_ec_before_td -- --exact --nocapture"
ASSERTIONS = ('a phase-less project WorkItem enters EC authoring before any TD authoring command',)
