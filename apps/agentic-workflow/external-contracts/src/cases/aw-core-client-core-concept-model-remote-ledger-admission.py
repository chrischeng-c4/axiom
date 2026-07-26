"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-core-concept-model-remote-ledger-admission"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "core-concept-model-and-invariants"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_core_concept_model_remote_ledger_admission.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::remote_wi_admission_seeds_the_local_ec_lifecycle_ledger -- --exact --nocapture"
ASSERTIONS = ('admitting a remote WorkItem seeds the local EC-first lifecycle ledger before dispatching artifact work',)
