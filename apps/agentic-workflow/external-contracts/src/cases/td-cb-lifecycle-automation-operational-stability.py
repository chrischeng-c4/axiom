"""Operational Python EC dimension required by the DeveloperTool profile."""

CASE_ID = "td-cb-lifecycle-automation-operational-stability"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-cb-lifecycle-automation-operational-stability"
DIMENSION = "stability"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-cb-lifecycle-automation-operational-stability"
ASSERTIONS = ("two fresh native Python terminal EC scenarios preserve red and green verdicts and close continuations",)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
