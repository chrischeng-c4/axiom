"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-capability-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_capability_admission.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-capability-admission"
ASSERTIONS = ('project, capability, and backlog roots emit action self_hosting_policy and policy_mode sanctioned_direct_commit', 'the envelopes expose no invoke command and both the repository tree and resolved runtime workspace remain byte-identical')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
