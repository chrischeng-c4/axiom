"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "jet-health-verification-dedup-smoke"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "project-health-no-regression"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_jet_health_verification_dedup_smoke.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case jet-health-verification-dedup-smoke"
ASSERTIONS = ('health command succeeds or reports only real project blockers', 'duplicate README gate commands do not multiply command execution inside one AW verification pass')


def verify() -> list[str]:
    from migration_clusters.existing_service_kit import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
