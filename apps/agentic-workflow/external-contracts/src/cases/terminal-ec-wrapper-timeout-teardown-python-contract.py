"""Bounded teardown contract for a stalled childless EC wrapper."""

CASE_ID = "terminal-ec-wrapper-timeout-teardown-python-contract"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case terminal-ec-wrapper-timeout-teardown-python-contract"
)
ASSERTIONS = (
    'a childless stalled Python EC wrapper is bounded by the configured timeout',
    'AW tears down the wrapper process group and reports a typed timeout',
)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
