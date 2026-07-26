"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-traceability-closure-gate"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "traceability-closure-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_traceability_closure_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib traceability -- --nocapture"
ASSERTIONS = ('traceability tests cover command, TD, source, and CB closure behavior',)
