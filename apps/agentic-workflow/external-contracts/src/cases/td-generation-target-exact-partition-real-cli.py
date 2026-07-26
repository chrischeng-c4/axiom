"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-exact-partition-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "exact-generated-unit-target-ownership"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_exact_partition_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_gen_exact_schema_unit_ownership_partitions_real_targets -- --nocapture"
ASSERTIONS = ('a cold public TD generation accepts two exact Schema owners', 'Alpha and Beta appear only in their declared target files', 'the admitted lifecycle advances to cb_genned')
