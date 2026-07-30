"""Red-to-green self-EC terminal gating contract."""

CASE_ID = "self-ec-fixture-loop-gate-python-contract"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "self-ec-fixture-loop-gate"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case self-ec-fixture-loop-gate-python-contract"
)
ASSERTIONS = (
    'a required Python EC refuses terminal progress while red',
    'green verification records the consulted case and emits the exact WI close command',
)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
