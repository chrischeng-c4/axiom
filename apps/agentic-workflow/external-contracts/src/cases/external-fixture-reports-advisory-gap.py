"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "external-fixture-reports-advisory-gap"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "external-fixture-reports-advisory-gap"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_external_fixture_reports_advisory_gap.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests regenerability_gaps_are_advisory_when_production_gates_clean -- --nocapture"
ASSERTIONS = ('the external-advisory fixture reports the regenerability gap as an advisory warning', 'the advisory gap does not add a production blocker when all required gates are clean')
