"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-cb-generation-and-standardize-scan-defaults"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "cb-generation-and-standardize-scan-defaults"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_cb_generation_and_standardize_scan_defaults.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib cb_gen_force_regen_defaults_td_root_to_project_tech_design -- --nocapture"
ASSERTIONS = ('CB force regeneration defaults to project-local tech-design roots',)
