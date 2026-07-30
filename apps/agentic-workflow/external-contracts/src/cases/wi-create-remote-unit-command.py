"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-remote-unit-command"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_remote_unit_command.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case wi-create-remote-unit-command"
)
ASSERTIONS = ('help hides deprecated remote flag', 'compatibility flag parses', 'backend decision is config-driven')


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
