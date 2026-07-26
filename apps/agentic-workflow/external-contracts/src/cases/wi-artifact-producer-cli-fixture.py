"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-artifact-producer-cli-fixture"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "shared-artifact-producer-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_artifact_producer_cli_fixture.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-artifact-producer-cli-fixture"
ASSERTIONS = ('aw wi create creates the durable work-item and payload skeletons before dispatch', 'stdout carries aw.artifact-producer.v1 and a bounded markdown_fragment slot', 'the slot names its schema, payload path, apply command, validation, evidence, and next transition')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
