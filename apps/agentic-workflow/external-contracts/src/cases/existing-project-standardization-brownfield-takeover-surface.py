"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-brownfield-takeover-surface"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_brownfield_takeover_surface.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests standardize_subcommands_registered -- --nocapture"
ASSERTIONS = ('standardize command surface is registered for brownfield takeover',)
