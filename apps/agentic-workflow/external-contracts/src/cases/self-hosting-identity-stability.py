"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-identity-stability"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "stability"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/stability_self_hosting_identity_stability.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case self-hosting-identity-stability"
ASSERTIONS = ('a malformed self-hosting WI identity returns a process error instead of entering the root runner', 'the failed resolution creates no loop state and leaves both the repository tree and resolved runtime workspace byte-identical')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
