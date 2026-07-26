"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_td_surface_convergence_ec_gated_terminal_check_unification_verb_lifecycle_policy_fixture_loop_self_ec.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests fixture_loop_test::fixture_loop_goal_converges_through_cb_to_required_ec_red_green_terminal -- --exact --nocapture"
ASSERTIONS = ('the public goal runner follows CB fill and check, stops at terminal required EC while red, resumes the same CB-filled WorkItem when green, records the consulted case, and closes at the unified terminal check',)
