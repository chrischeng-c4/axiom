"""Black-box manual EC runner and continuation output convention."""

CASE_ID = "manual-runner-output-convention-python-contract"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case manual-runner-output-convention-python-contract"
)
ASSERTIONS = (
    "EC draft writes the runner, the case module, the inventory, and the lock in that exact artifact set",
    "the envelope emits the exact structural-check continuation and the scaffolded runner fails closed until authored",
)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
