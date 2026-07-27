"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "external-fixture-reports-advisory-gap"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "external-fixture-reports-advisory-gap"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_external_fixture_reports_advisory_gap.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case external-fixture-reports-advisory-gap"
ASSERTIONS = ('the external-advisory fixture reports the regenerability gap as an advisory warning', 'the advisory gap does not add a production blocker when all required gates are clean')


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
