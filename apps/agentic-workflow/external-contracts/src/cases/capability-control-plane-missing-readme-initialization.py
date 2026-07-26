"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-missing-readme-initialization"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "missing-readme-initialization"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_missing_readme_initialization.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib capability_init -- --nocapture"
ASSERTIONS = ('capability init renders a canonical README shell',)
