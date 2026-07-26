"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-default-section-queue-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-default-section-queue-preservation"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_default_section_queue_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-default-section-queue-real-cli"
ASSERTIONS = ('the fresh skeleton contains logic followed by unit-test', 'logic applicability emits an applicability unit-test dispatch', 'contract authoring does not start before unit-test applicability')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
