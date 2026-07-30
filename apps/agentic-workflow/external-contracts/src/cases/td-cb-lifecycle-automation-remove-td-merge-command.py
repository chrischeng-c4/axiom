"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-remove-td-merge-command"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "remove-td-merge-command"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_remove_td_merge_command.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case td-cb-lifecycle-automation-remove-td-merge-command"
)
ASSERTIONS = ('the retired `aw td merge` command is absent from the Clap tree and parsing it returns the literal unrecognized-subcommand failure (#914, refs #851)',)


def verify() -> list[str]:
    from migration_clusters.td_lifecycle import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
