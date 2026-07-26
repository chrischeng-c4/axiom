"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-cli-workflow-chain"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "cli-workflow-chain"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_cli_workflow_chain.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case workflow-root-runner-cli-workflow-chain"
ASSERTIONS = ('CLI workflow chain emit sites resolve through the real clap tree for capability and work-item roots',)


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
