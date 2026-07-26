"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-artifact-producer-cli-fixture"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "shared-artifact-producer-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_artifact_producer_cli_fixture.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-artifact-producer-cli-fixture"
ASSERTIONS = ('aw td create creates the durable TD skeleton and one JSON payload for the current queued section', 'the TD contract exposes validation, generation, evidence, and a runnable next transition', 'CODEGEN-BEGIN/END and HANDWRITE-BEGIN/END ownership outputs are explicit', 'HANDWRITE requires gap, tracker, and reason fields')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
