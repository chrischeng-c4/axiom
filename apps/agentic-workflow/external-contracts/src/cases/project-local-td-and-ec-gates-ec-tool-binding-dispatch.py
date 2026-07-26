"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-ec-tool-binding-dispatch"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-tool-binding-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_ec_tool_binding_dispatch.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-local-td-and-ec-gates-ec-tool-binding-dispatch"
ASSERTIONS = ('EC tool binding commands resolve the configured runner dispatch',)


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
