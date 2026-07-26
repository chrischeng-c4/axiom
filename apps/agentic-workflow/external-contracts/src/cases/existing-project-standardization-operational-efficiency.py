"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "existing-project-standardization-operational-efficiency"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "efficiency"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered -- --nocapture"
ASSERTIONS = ("existing-project-standardization completes its representative gate within 120 seconds with at least one passed and zero failed tests",)
