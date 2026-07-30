"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-create-remote-flag-tests"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-create-remote-flag-tests"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_create_remote_flag_tests.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case wi-create-remote-flag-tests"
)
ASSERTIONS = ('create help hides remote flag', 'hidden remote compatibility flag parses', 'create behavior is config-driven')


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
