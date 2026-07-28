"""Native Python EC for the two-cell semantic-health contract."""

CASE_ID = "project-health-total-observation"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "health-total-observation-and-readiness-gate"
DIMENSION = "behavior"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-health-total-observation"
ASSERTIONS = (
    "matching TD-applicable EC evidence makes ec_accepts_td pass",
    "explicit failing EC evidence rejects TD with exact case counts",
    "matching executable public behaviors make ec_td_alignment pass",
    "internal TD artifacts do not require EC coverage",
    "same-artifact behavior drift is reported in both directions",
    "EC-only and public-TD-only behaviors are reported in opposite directions",
    "missing TD-stage evidence is indeterminate rather than false-green",
    "stdout and durable payload expose exactly the same two semantic cells",
)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
