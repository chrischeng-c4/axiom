"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-ec-external-contract-source"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-external-contract-source"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_ec_external_contract_source.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_draft_fill_markdown_drives_inventory -- --nocapture"
ASSERTIONS = ('EC draft and fill Markdown drives inventory generation',)
