"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-capability-readiness-reporting"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-readiness-reporting"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_capability_readiness_reporting.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib fixture_reference_can_verify_required_claim -- --nocapture"
ASSERTIONS = ('required claims can be verified by fixture references',)
