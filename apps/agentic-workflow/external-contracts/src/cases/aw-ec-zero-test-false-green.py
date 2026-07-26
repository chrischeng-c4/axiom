"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-ec-zero-test-false-green"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-false-green-guard"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_ec_zero_test_false_green.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-ec-zero-test-false-green"
ASSERTIONS = ('aw ec verify marks a cargo test command failed when the command exits 0 after running zero tests', 'generated Rust EC wrappers capture stdout and reject the same zero-test false green', 'ec gen keeps precise cargo test target selectors instead of relying on crate-wide filters when the source contract provides one')


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
