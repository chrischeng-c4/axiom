"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-project-local-td-root-resolver"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "project-local-td-root-resolver"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_project_local_td_root_resolver.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib falls_back_to_project_tech_design -- --nocapture"
ASSERTIONS = ('project-local TD root resolution falls back to the project tech-design directory',)
