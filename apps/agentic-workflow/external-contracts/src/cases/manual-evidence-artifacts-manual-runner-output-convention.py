"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "manual-evidence-artifacts-manual-runner-output-convention"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_manual_evidence_artifacts_manual_runner_output_convention.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_doc_gen_writes_manual_from_inventory -- --nocapture"
ASSERTIONS = ('EC doc generation writes the manual from inventory',)
