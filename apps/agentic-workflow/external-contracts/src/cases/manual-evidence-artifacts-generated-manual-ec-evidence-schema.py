"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "manual-evidence-artifacts-generated-manual-ec-evidence-schema"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "generated-manual-ec-evidence-schema"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_manual_evidence_artifacts_generated_manual_ec_evidence_schema.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_generated_manual_artifact -- --nocapture"
ASSERTIONS = ('generated manual EC evidence metadata validates',)
