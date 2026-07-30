"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "authoritative-fixture-blocks-on-regenerability-gap"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "authoritative-fixture-blocks-on-regenerability-gap"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_authoritative_fixture_blocks_on_regenerability_gap.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case authoritative-fixture-blocks-on-regenerability-gap"
)
ASSERTIONS = ('a non-self fixture configured generator_authoritative reports production_ready false for a tracked regenerability gap', 'the health payload exposes the regenerability production blocker and a runnable remediation command')


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
