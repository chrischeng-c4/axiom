"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-td-lock-and-external-contract-target-resolution"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "td-lock-and-external-contract-target-resolution"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_td_lock_and_external_contract_target_resolution.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_context_defaults_td_root_to_project_tech_design -- --nocapture"
ASSERTIONS = ('EC context defaults TD roots to the project tech-design directory',)
