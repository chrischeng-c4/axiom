"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-capability-readiness-reporting"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-readiness-reporting"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_capability_readiness_reporting.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case capability-control-plane-capability-readiness-reporting"
ASSERTIONS = ('required claims can be verified by fixture references',)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
