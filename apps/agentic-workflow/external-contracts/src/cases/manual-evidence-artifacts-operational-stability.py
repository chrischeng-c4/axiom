"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "manual-evidence-artifacts-operational-stability"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-evidence-artifacts-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case manual-evidence-artifacts-operational-stability"
ASSERTIONS = ("two fresh native Python EC scaffolds preserve case, evidence, and structural-check identities",)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
