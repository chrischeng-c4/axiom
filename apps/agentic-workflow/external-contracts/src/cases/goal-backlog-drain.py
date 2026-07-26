"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "goal-backlog-drain"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_goal_backlog_drain.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case goal-backlog-drain"
ASSERTIONS = ('a reviewed epic graph parks one blocked child and dispatches its ready sibling deterministically', 'the terminal envelope names the still-parked WI and its reason with no spinning or premature completion', 'an already-reviewed epic is never redispatched for atomization')


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
