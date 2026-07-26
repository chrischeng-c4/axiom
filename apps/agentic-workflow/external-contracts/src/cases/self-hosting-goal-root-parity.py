"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-goal-root-parity"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_goal_root_parity.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::run::tests::self_hosting_wi_identity_and_rollup_never_reenter_root_runner -- --nocapture"
ASSERTIONS = ('self-AW WI identity resolution and rollup routing reject before loop-state or dispatch touch the fixture tree',)
