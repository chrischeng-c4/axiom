"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-help-smoke"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-create-help-smoke"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_help_smoke.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-create-help-smoke"
ASSERTIONS = ('stdout does not contain --remote',)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
