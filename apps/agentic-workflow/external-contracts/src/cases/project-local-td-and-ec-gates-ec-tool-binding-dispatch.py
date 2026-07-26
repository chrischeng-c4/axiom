"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-ec-tool-binding-dispatch"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-tool-binding-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_ec_tool_binding_dispatch.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_binding_command -- --nocapture"
ASSERTIONS = ('EC tool binding commands resolve the configured runner dispatch',)
