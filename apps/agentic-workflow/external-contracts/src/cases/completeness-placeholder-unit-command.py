"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "completeness-placeholder-unit-command"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "cb-lifecycle-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_completeness_placeholder_unit_command.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case completeness-placeholder-unit-command"
ASSERTIONS = ('placeholder code is rejected', 'omitted prose is rejected', 'future_work_allowed TODO is accepted')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
