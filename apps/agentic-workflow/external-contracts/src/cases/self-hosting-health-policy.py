"""Native Python EC for the public AW self-hosting health policy boundary."""

CASE_ID = "self-hosting-health-policy"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-health-policy"
ASSERTIONS = (
    "public AW health enables Python-first self-hosting roots",
    "bounded direct repair is conditional self-hosting fallback metadata",
)


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
