"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-root-envelope-completion-contract"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "root-envelope-completion-contract"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_root_envelope_completion_contract.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib create_wi_blocks_on_pending_epicize_artifact -- --nocapture"
ASSERTIONS = ('root envelope blocks on pending epicize artifacts before creating WIs',)
