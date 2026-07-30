"""Black-box generated-manual EC evidence-schema contract."""

CASE_ID = "manual-evidence-schema-python-contract"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "generated-manual-ec-evidence-schema"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case manual-evidence-schema-python-contract"
)
ASSERTIONS = (
    "the generated manual EC inventory entry equals its exact declared field set",
    "the evidence path and gate command derive from the case id while the oracle stays an explicit fill marker",
)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
