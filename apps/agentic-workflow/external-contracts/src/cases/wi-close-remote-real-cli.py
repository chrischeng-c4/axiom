"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-close-remote-real-cli"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-close-remote-rehydration"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_close_remote_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-close-remote-real-cli"
ASSERTIONS = ('the repo-built aw binary resolves a tracker-only numeric issue through the configured GitHub backend', '--repo selects every remote read and mutation', 'the optional reason and close mutation each occur exactly once across a retry', 'a missing remote names its backend and repository and emits an executable recovery command', 'a local-only issue still moves from open to closed')


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
