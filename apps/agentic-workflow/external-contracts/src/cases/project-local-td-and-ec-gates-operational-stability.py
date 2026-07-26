"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "project-local-td-and-ec-gates-operational-stability"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "cb-generation-and-standardize-scan-defaults"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design -- --nocapture"
ASSERTIONS = ("project-local-td-and-ec-gates reports identical non-zero passed and zero-failed totals across two executions",)
