"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "runtime-envelope-backward-compatibility"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "runtime-envelope-backward-compatibility"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_runtime_envelope_backward_compatibility.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib envelope_profile -- --nocapture"
ASSERTIONS = ('legacy Dispatch JSON without artifact_quality_profile remains accepted', 'Dispatch JSON with artifact_quality_profile roundtrips')
