"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "capability-control-plane-markdown-capability-schema"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "markdown-capability-schema"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_capability_control_plane_markdown_capability_schema.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-markdown-capability-schema"
)
ASSERTIONS = ('canonical field-style capability contracts parse from Markdown',)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
