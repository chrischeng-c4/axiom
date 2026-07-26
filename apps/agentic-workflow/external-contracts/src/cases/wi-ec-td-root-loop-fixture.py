"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-ec-td-root-loop-fixture"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "wi-ec-td-root-loop"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_ec_td_root_loop_fixture.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-ec-td-root-loop-fixture"
ASSERTIONS = ('fixture root follows emitted commands until completion.workflow_complete=true', 'no retired lifecycle command or hidden agent-only step is required')


def verify() -> list[str]:
    from migration_clusters.workflow_runner import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
