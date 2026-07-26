"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-typed-prompt-ir-and-envelope-projection"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "typed-prompt-ir-and-envelope-projection"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_typed_prompt_ir_and_envelope_projection.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-typed-prompt-ir-and-envelope-projection"
ASSERTIONS = ('a production WorkflowEnvelope pins every typed prompt JSON field and its rendered agent_prompt from the same decoded IR', 'an invalid typed contract makes WorkflowEnvelope serialization fail instead of falling back to prose')


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
