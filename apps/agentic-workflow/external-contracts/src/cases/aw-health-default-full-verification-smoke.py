"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-health-default-full-verification-smoke"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "aw-health-default-full-verification-smoke"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_health_default_full_verification_smoke.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests health_verbose_stdout_streams_progress_before_result -- --nocapture"
ASSERTIONS = ('stdout includes progress JSONL events before the final result when long gates run', 'the final result includes payload_path', 'the payload file contains complete blocker and command evidence')
