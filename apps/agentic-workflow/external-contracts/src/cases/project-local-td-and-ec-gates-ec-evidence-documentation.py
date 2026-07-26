"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-ec-evidence-documentation"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-evidence-documentation"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_ec_evidence_documentation.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib ec_doc -- --nocapture"
ASSERTIONS = ('EC documentation generation and drift checks are covered',)
