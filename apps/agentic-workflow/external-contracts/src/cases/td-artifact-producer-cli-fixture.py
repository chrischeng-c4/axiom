"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-artifact-producer-cli-fixture"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "shared-artifact-producer-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_artifact_producer_cli_fixture.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case td-artifact-producer-cli-fixture"
)
ASSERTIONS = ('aw td create initializes one WI-bound Python module below tech-design/src', 'the exact apply handoff contains no Markdown or JSON section payload')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
