"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-prompt-vocabulary-and-grammar"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "prompt-vocabulary-and-grammar"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_prompt_vocabulary_and_grammar.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-prompt-vocabulary-and-grammar"
ASSERTIONS = ('the registered public Markdown and JSON renderers expose identical prompt content with the exact closed vocabulary, seven ASCII operators, complete EC-first Python Spec transition table, sole workflow-authority boundary, and no Unicode operator lookalikes',)


def verify() -> list[str]:
    from migration_clusters.prompt_artifacts import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
