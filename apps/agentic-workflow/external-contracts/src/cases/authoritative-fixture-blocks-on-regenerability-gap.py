"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "authoritative-fixture-blocks-on-regenerability-gap"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "authoritative-fixture-blocks-on-regenerability-gap"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_authoritative_fixture_blocks_on_regenerability_gap.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests authoritative_regenerability_gaps_block_project_health -- --nocapture"
ASSERTIONS = ('a non-self fixture configured generator_authoritative reports production_ready false for a tracked regenerability gap', 'the health payload exposes the regenerability production blocker and a runnable remediation command')
