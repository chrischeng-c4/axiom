"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-wi-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_wi_admission.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-wi-admission"
ASSERTIONS = ('the self-hosted WI root enters the normal EC-first lifecycle', 'the envelope dispatches the exact project-and-WI-scoped EC check')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
