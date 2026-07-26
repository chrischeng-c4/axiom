"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "manual-evidence-artifacts-operational-efficiency"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "efficiency"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case manual-evidence-artifacts-operational-efficiency"
ASSERTIONS = ("manual-evidence-artifacts completes its representative gate within 120 seconds with at least one passed and zero failed tests",)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
