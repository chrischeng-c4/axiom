"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-cb-and-cold-verification-gates"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "cb-and-cold-verification-gates"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_cb_and_cold_verification_gates.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cb_gen_cold_rebuild_targets_include_codegen_changes -- --nocapture"
ASSERTIONS = ('CB cold rebuild targets include codegen changes',)
