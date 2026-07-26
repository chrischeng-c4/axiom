"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-self-ec-fixture-loop-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "self-ec-fixture-loop-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_self_ec_fixture_loop_gate.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-self-ec-fixture-loop-gate"
ASSERTIONS = ('a configured required EC case refuses the unchanged CB-filled WorkItem while red without phase or close mutation, then permits terminal close only when green and records the consulted case in the success envelope',)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
