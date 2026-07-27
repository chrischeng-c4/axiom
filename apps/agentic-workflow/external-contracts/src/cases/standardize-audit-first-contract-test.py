"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "standardize-audit-first-contract-test"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "standardize-audit-first-contract-test"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_standardize_audit_first_contract_test.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case standardize-audit-first-contract-test"
ASSERTIONS = ('audit_required is true without a preservation baseline', 'audit_required is false when a baseline exists', 'route and command surfaces are included in the fixture baseline')


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
