"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-stability"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "stability"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered -- --nocapture"
ASSERTIONS = ("existing-project-standardization reports identical non-zero passed and zero-failed totals across two executions",)
