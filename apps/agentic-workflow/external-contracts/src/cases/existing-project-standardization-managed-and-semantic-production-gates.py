"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-managed-and-semantic-production-gates"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "managed-and-semantic-production-gates"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_managed_and_semantic_production_gates.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib semantic_coverage_prioritizes_missing_td_before_generator_gap -- --nocapture"
ASSERTIONS = ('semantic coverage prioritizes missing TD before generator gaps',)
