"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-lifecycle-prompt-stage-conformance"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "lifecycle-prompt-migration-and-conformance"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_lifecycle_prompt_stage_conformance.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-lifecycle-prompt-stage-conformance"
ASSERTIONS = ('every Python EC, TD, and CB phase-table row, including EC review and change close, projects exact writable and read-only scopes, verifier predicate, terminal level, and lifecycle guard', 'a frontend CB transition projects the complete concrete artifact-quality guard id set')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
