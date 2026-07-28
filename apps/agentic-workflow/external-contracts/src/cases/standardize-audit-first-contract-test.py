"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "standardize-audit-first-contract-test"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "standardize-audit-first-contract-test"
DIMENSION = "behavior"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case standardize-audit-first-contract-test"
ASSERTIONS = (
    "a missing preservation baseline is a successful not-applicable health observation",
    "recording a baseline produces a successful recorded health observation",
    "the durable audit contains exact typed route and command preservation surfaces",
    "health surface and debt counts correspond to the durable audit",
)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
