"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-hand-written-complete-evidence-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "hand-written-implementation-evidence-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_hand_written_complete_evidence_gate.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-hand-written-complete-evidence-gate"
ASSERTIONS = ('terminal CB check accepts complete evidence only after every declared hand-written create and modify target has a committed implementation diff',)


def verify() -> list[str]:
    from migration_clusters.td_lifecycle import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
