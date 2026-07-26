"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "workflow-root-runner-parent-rollup-routing"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "parent-rollup-routing"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_workflow_root_runner_parent_rollup_routing.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib closed_change_outputs_parent_inspection -- --nocapture"
ASSERTIONS = ('closed change roots route agents to parent inspection',)
