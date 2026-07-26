"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "ec-artifact-producer-cli-fixture"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "shared-artifact-producer-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_ec_artifact_producer_cli_fixture.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case ec-artifact-producer-cli-fixture"
ASSERTIONS = ('aw ec draft creates the canonical Python pyproject, runner, and bounded case module without Markdown fallback', 'draft emits aw.cli.v1 with the exact aw ec check continuation', 'the generated Python inventory preserves the requested capability and passes structural check')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
