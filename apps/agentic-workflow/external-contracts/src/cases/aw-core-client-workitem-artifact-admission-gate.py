"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-workitem-artifact-admission-gate"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-artifact-admission-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_workitem_artifact_admission_gate.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-core-client-workitem-artifact-admission-gate"
ASSERTIONS = ('the real compiled CB generator rejects an unsupported or unadmitted artifact before issue, Git, or source mutation and accepts a valid admitted WorkItem with exact generated ownership',)


def verify() -> list[str]:
    from migration_clusters.core_model import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
