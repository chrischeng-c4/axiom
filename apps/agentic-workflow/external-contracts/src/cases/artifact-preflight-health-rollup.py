"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "artifact-preflight-health-rollup"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "artifact-preflight-health-rollup"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_artifact_preflight_health_rollup.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case artifact-preflight-health-rollup"
ASSERTIONS = ('missing hard evidence appears in production_blockers', 'missing advisory evidence appears in quality warnings', 'passing evidence keeps production_ready true when no other blockers exist')


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
