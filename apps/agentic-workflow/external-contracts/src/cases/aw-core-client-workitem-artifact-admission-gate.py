"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-workitem-artifact-admission-gate"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-artifact-admission-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_workitem_artifact_admission_gate.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests inplace_mode_test::workitem_artifact_admission_gate_real_cli_positive_and_negative -- --exact --nocapture"
ASSERTIONS = ('the real compiled CB generator rejects an unsupported or unadmitted artifact before issue, Git, or source mutation and accepts a valid admitted WorkItem with exact generated ownership',)
