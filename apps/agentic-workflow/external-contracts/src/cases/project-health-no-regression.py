"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-health-no-regression"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "project-health-no-regression"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_health_no_regression.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-health-no-regression"
ASSERTIONS = ('unrelated workflow envelope change does not regress project health reporting',)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
