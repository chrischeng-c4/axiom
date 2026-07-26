"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-managed-and-semantic-production-gates"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "managed-and-semantic-production-gates"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_managed_and_semantic_production_gates.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case existing-project-standardization-managed-and-semantic-production-gates"
ASSERTIONS = ('semantic coverage prioritizes missing TD before generator gaps',)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
