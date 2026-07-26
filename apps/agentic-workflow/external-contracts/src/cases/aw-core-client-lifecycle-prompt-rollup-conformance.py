"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-lifecycle-prompt-rollup-conformance"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "lifecycle-prompt-migration-and-conformance"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_lifecycle_prompt_rollup_conformance.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::prompt_contract_distinguishes_child_parked_and_root_terminal -- --exact --nocapture"
ASSERTIONS = ('child dispatch, parked backlog work, and root terminal completion are distinct prompt states',)
