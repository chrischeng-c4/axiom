"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "standardize-audit-first-contract-test"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "standardize-audit-first-contract-test"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_standardize_audit_first_contract_test.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib standardize_audit -- --nocapture"
ASSERTIONS = ('audit_required is true without a preservation baseline', 'audit_required is false when a baseline exists', 'route and command surfaces are included in the fixture baseline')
