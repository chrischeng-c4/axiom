"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "goal-backlog-drain"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_goal_backlog_drain.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests goal_backlog -- --nocapture"
ASSERTIONS = ('a reviewed epic graph parks one blocked child and dispatches its ready sibling deterministically', 'the terminal envelope names the still-parked WI and its reason with no spinning or premature completion', 'an already-reviewed epic is never redispatched for atomization')
