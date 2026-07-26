"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-hand-written-implementation-evidence-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "hand-written-implementation-evidence-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_hand_written_implementation_evidence_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_no_merge_test::test_code_check_refuses_unchanged_hand_written_modify_paths -- --exact --nocapture"
ASSERTIONS = ('terminal CB check refuses a hand-written modify path with zero committed implementation diff since its Td-Init baseline (#1382)',)
