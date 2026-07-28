"""Native Python EC for total health observation and focused isolation."""

CASE_ID = "project-health-total-observation"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "health-total-observation-and-readiness-gate"
DIMENSION = "behavior"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-health-total-observation"
ASSERTIONS = (
    "focused capability does not call a deliberately poisoned mutation evaluator",
    "focused advisory and required unavailability emit matching durable payloads",
    "aggregate advisory unavailability remains degraded and production-ready",
    "aggregate required failure is blocked and nonzero",
    "focused failure is nonzero while not-applicable is successful",
)


def verify() -> list[str]:
    from migration_clusters.existing_health import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
