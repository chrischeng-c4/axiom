"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "missing-source-review-fails"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "missing-source-review-fails"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_missing_source_review_fails.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib source_reference_missing_required_source -- --nocapture"
ASSERTIONS = ('source_backed is false', 'finding code is missing_source_reference')
