"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-exact-partition-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "exact-generated-unit-target-ownership"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_exact_partition_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-generation-target-exact-partition-real-cli"
ASSERTIONS = ('a cold public TD generation accepts two exact Schema owners', 'Alpha and Beta appear only in their declared target files', 'the admitted lifecycle advances to cb_genned')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
