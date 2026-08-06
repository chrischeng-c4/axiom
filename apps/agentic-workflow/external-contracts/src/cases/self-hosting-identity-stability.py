"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-identity-stability"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "stability"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/stability_self_hosting_identity_stability.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case self-hosting-identity-stability"
)
ASSERTIONS = ('a malformed self-hosting WI identity returns a normal blocked envelope', 'the failed resolution leaves repository and runtime state byte-identical')


def verify() -> list[str]:
    from migration_clusters.self_hosting_admission import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
