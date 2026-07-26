"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-remove-agent-estimate-spec-check"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_remove_agent_estimate_spec_check.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::issues::tests::wi_remove_agent_estimate_legacy_section_is_inert -- --exact --nocapture"
ASSERTIONS = ('legacy Agent Estimate input remains parseable but is inert and creates no readiness requirement',)
