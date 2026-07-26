"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-capability-project-sweep"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-project-sweep"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_capability_project_sweep.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib capability_sweep -- --nocapture"
ASSERTIONS = ('capability sweep groups project readiness and next actions',)
