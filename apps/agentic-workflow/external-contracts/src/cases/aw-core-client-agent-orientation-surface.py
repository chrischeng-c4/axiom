"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-agent-orientation-surface"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-orientation-surface"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_agent_orientation_surface.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-agent-orientation-surface"
ASSERTIONS = ('agent-facing llm outline lists the registered command surface',)


def verify() -> list[str]:
    from migration_clusters.core_model import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
