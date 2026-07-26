"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-project-dirty-scope-protection"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "project-dirty-scope-protection"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_project_dirty_scope_protection.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib semantic_coverage_excludes_aw_ec_generated_wrappers -- --nocapture"
ASSERTIONS = ('semantic coverage excludes generated EC wrappers from dirty source scope',)
