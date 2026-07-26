"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-hand-written-complete-evidence-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "hand-written-implementation-evidence-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_hand_written_complete_evidence_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_no_merge_test::test_code_check_accepts_complete_hand_written_lifecycle_diff -- --exact --nocapture"
ASSERTIONS = ('terminal CB check accepts complete evidence only after every declared hand-written create and modify target has a committed implementation diff',)
