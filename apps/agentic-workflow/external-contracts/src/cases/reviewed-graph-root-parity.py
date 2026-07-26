"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "reviewed-graph-root-parity"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "goal-unified-loop-verb"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_reviewed_graph_root_parity.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case reviewed-graph-root-parity"
ASSERTIONS = ('epic and backlog roots select the same ready leaf after epic-first priority ordering', 'terminal child rollup closes the epic without re-atomizing', 'stale or invalid graph metadata fails closed with issue-specific remediation')


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
