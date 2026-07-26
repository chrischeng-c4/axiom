"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-markdown-capability-schema"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "markdown-capability-schema"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_markdown_capability_schema.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib markdown_capability_tables -- --nocapture"
ASSERTIONS = ('canonical field-style capability contracts parse from Markdown',)
