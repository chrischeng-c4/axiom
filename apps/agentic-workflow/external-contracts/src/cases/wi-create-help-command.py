"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-help-command"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_help_command.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-create-help-command"
ASSERTIONS = ('help output does not list --remote',)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
