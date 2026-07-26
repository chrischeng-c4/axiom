"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-core-client-agent-orientation-surface"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "agent-orientation-surface"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_core_client_agent_orientation_surface.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib llm_outline_uses_cli_std_and_standard_commands -- --nocapture"
ASSERTIONS = ('agent-facing llm outline lists the registered command surface',)
