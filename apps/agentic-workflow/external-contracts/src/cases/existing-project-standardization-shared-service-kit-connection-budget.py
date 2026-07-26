"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-shared-service-kit-connection-budget"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_shared_service_kit_connection_budget.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case existing-project-standardization-shared-service-kit-connection-budget"
ASSERTIONS = ('connection admission consumes the configured budget and releases the permit after the handler finishes',)


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
