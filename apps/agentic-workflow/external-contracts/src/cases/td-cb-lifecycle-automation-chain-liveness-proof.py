"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-chain-liveness-proof"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "chain-liveness-proof"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_chain_liveness_proof.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-chain-liveness-proof"
ASSERTIONS = ('the exact driven chain reaches a terminal action within its bounded hop budget without landing on a deadlock phase (#914, refs #921)',)


def verify() -> list[str]:
    from migration_clusters.td_lifecycle import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
