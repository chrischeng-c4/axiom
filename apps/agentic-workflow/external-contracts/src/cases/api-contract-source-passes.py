"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "api-contract-source-passes"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "api-contract-source-passes"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_api_contract_source_passes.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case api-contract-source-passes"
ASSERTIONS = ('source_backed is true', 'findings is empty')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
