"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-remove-agent-estimate-build"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_remove_agent_estimate_build.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cli::issues::tests::wi_remove_agent_estimate_prioritize_output_omits_estimate_fields -- --exact --nocapture"
ASSERTIONS = ('prioritization output contains no estimate field while retaining the bounded capability-to-epic planning result',)
