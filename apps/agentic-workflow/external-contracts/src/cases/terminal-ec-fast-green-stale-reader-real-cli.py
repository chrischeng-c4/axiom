"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-fast-green-stale-reader-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_fast_green_stale_reader_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case terminal-ec-fast-green-stale-reader-real-cli"
ASSERTIONS = ('a debug-only bounded barrier proves process B read cb_filled before process A completes', 'process A executes the fast-green inventory and completes the terminal transition', 'process B acquires afterward, re-reads td_merged, and reports terminal retry without EC', 'the EC launch marker contains one line and git contains one Cb-CodeCheck terminal commit')


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
