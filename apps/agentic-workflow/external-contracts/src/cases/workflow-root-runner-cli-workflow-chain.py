"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-cli-workflow-chain"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "cli-workflow-chain"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_cli_workflow_chain.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib emit_registry_entries_are_all_chain_valid -- --nocapture"
ASSERTIONS = ('CLI workflow chain emit sites resolve through the real clap tree for capability and work-item roots',)
