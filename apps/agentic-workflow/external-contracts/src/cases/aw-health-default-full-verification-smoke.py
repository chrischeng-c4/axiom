"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-health-default-full-verification-smoke"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "aw-health-default-full-verification-smoke"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_health_default_full_verification_smoke.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-health-default-full-verification-smoke"
ASSERTIONS = ('stdout includes progress JSONL events before the final result when long gates run', 'the final result includes payload_path', 'the payload file contains complete blocker and command evidence')


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
