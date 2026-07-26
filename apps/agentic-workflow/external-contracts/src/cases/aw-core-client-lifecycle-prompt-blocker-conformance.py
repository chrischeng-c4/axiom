"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-lifecycle-prompt-blocker-conformance"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "lifecycle-prompt-migration-and-conformance"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_lifecycle_prompt_blocker_conformance.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::prompt_contract_routes_invalid_oracle_and_typed_blockers -- --exact --nocapture"
ASSERTIONS = ('invalid oracle state routes to EC repair and decision, approval, environment, red-gate, and missing-evidence blockers remain typed with exact resume',)
