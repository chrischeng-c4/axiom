"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-epic-project-label-dispatch-chain"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-epic-project-label-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_epic_project_label_dispatch_chain.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-epic-project-label-dispatch-chain"
ASSERTIONS = ('run.rs:open_epic_envelope is present in EMIT_REGISTRY', 'aw wi atomize --project pgpool parses through the real CLI tree', 'aw conf init --project-label app:workbench parses through the real CLI tree')


def verify() -> list[str]:
    from migration_clusters.core_model import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
