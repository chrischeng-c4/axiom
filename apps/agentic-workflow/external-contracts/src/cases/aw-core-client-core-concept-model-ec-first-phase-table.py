"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-core-concept-model-ec-first-phase-table"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "core-concept-model-and-invariants"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_core_concept_model_ec_first_phase_table.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-core-concept-model-ec-first-phase-table"
ASSERTIONS = ('the Python Spec lifecycle has one explicit EC review, TD behavior/security, CB generation/check, and terminal all-dimension EC routing table with no phase gaps',)


def verify() -> list[str]:
    from migration_clusters.core_model import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
