"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-lifecycle-prompt-stage-conformance"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "lifecycle-prompt-migration-and-conformance"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_lifecycle_prompt_stage_conformance.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::python_artifact_prompt_contracts_preserve_stage_owner_and_gate -- --exact --nocapture"
ASSERTIONS = ('every Python EC, TD, and CB phase-table row, including EC review and change close, projects exact writable and read-only scopes, verifier predicate, terminal level, and lifecycle guard', 'a frontend CB transition projects the complete concrete artifact-quality guard id set')
