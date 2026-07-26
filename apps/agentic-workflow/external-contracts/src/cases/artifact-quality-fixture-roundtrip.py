"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "artifact-quality-fixture-roundtrip"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-lifecycle-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_artifact_quality_fixture_roundtrip.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case artifact-quality-fixture-roundtrip"
ASSERTIONS = ('every serialized fixture decodes, validates, and re-encodes to the same typed profile', 'each profile exposes intent_read, quality_dials, source_policy, and preflight_gate_set in review context')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
