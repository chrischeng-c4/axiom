"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "artifact-preflight-health-rollup"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "artifact-preflight-health-rollup"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_artifact_preflight_health_rollup.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests preflight_ -- --nocapture"
ASSERTIONS = ('missing hard evidence appears in production_blockers', 'missing advisory evidence appears in quality warnings', 'passing evidence keeps production_ready true when no other blockers exist')
