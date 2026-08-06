"""Black-box contract for the traceability closure gate (#3309)."""

CASE_ID = "existing-project-standardization-traceability-closure-gate"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "traceability-closure-gate"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case existing-project-standardization-traceability-closure-gate"
)
ASSERTIONS = (
    "health payload closes source and TD traceability through explicit percentages and gaps",
    "command traceability is reported as an independent axis",
)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
