"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "placeholder-completeness-unit-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "placeholder-completeness-unit-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_placeholder_completeness_unit_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib completeness_placeholder -- --nocapture"
ASSERTIONS = ('placeholder code rejected', 'omitted prose rejected', 'explicit future TODO allowed')
