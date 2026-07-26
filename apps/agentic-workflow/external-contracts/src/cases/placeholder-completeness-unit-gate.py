"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "placeholder-completeness-unit-gate"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "placeholder-completeness-unit-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_placeholder_completeness_unit_gate.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case placeholder-completeness-unit-gate"
ASSERTIONS = ('placeholder code rejected', 'omitted prose rejected', 'explicit future TODO allowed')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
