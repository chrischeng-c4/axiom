"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-remove-td-merge-command"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "remove-td-merge-command"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_remove_td_merge_command.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests legacy_cli_removal_test::test_td_merge_subcommand_is_removed -- --exact --nocapture"
ASSERTIONS = ('the retired `aw td merge` command is absent from the Clap tree and parsing it returns the literal unrecognized-subcommand failure (#914, refs #851)',)
