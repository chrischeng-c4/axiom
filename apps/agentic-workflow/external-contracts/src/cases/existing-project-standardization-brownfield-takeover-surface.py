"""Black-box contract for the brownfield takeover CLI surface (#3309)."""

CASE_ID = "existing-project-standardization-brownfield-takeover-surface"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "brownfield-takeover-surface"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-brownfield-takeover-surface"
)
ASSERTIONS = (
    "retired standardize namespace is absent from the real CLI",
    "brownfield audit recording is available under aw td audit-record",
)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
