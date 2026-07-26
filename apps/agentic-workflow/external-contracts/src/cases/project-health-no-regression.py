"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-health-no-regression"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "project-health-no-regression"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_health_no_regression.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib project_health -- --nocapture"
ASSERTIONS = ('unrelated workflow envelope change does not regress project health reporting',)
