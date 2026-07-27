"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-bounded-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "efficiency"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/efficiency_self_hosting_bounded_admission.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-bounded-admission"
ASSERTIONS = ('backlog admission reaches reviewed-graph preflight instead of a self-hosting rejection', 'repeat WI admission is byte-stable and does not mutate lifecycle state')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
