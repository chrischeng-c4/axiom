"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-project-dirty-scope-protection"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "project-dirty-scope-protection"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_project_dirty_scope_protection.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-local-td-and-ec-gates-project-dirty-scope-protection"
ASSERTIONS = ('semantic coverage excludes generated EC wrappers from dirty source scope',)


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
