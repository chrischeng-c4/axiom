"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-retry-transition-lease-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_retry_transition_lease_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case terminal-ec-retry-transition-lease-real-cli"
ASSERTIONS = ('a bounded debug-only barrier pauses the owner after td_merged is written while its lease remains held', 'the second process reads retry phase and promptly receives terminal_ec_single_flight', 'the refusal points to the exact same-slug aw cb check retry', 'after releasing the owner there is one EC launch and one Cb-CodeCheck terminal commit')


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
