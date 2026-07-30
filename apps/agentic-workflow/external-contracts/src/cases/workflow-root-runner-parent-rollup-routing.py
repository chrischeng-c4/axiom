"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-parent-rollup-routing"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "parent-rollup-routing"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_parent_rollup_routing.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-parent-rollup-routing"
)
ASSERTIONS = ('closed change roots route agents to parent inspection',)


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
