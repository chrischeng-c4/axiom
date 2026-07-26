"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-ownership-inferred-single-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "ambiguous-multi-target-generation-preflight"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_ownership_inferred_single_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_gen_no_changes_single_inferred_schema_target_remains_compatible -- --nocapture"
ASSERTIONS = ('a no-Changes Schema TD with one exact managed spec ref passes caller admission', 'the executor selects the same inferred target and generates Widget', 'the lifecycle advances to cb_genned on the persistent project branch')
