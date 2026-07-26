"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-root-envelope-completion-contract"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "root-envelope-completion-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_root_envelope_completion_contract.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case workflow-root-runner-root-envelope-completion-contract"
ASSERTIONS = ('root envelope blocks on pending epicize artifacts before creating WIs',)


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
