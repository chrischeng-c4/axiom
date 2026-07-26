"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-ec-no-child-wrapper-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_ec_no_child_wrapper_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case terminal-ec-no-child-wrapper-real-cli"
ASSERTIONS = ('the real aw binary returns within the configured one-second deadline plus bounded cleanup grace', 'the helper confirms its external child exited before the wrapper timed out', 'the wrapper PID no longer exists after aw returns', 'the envelope has terminal_ec_timeout and exact aw cb check slug next.command', 'the work item remains open in cb_filled and no terminal commit is created')


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
