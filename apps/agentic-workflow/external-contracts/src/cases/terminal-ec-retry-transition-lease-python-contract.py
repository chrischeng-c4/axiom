"""Retry contention contract for the in-flight terminal EC lease."""

CASE_ID = "terminal-ec-retry-transition-lease-python-contract"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case terminal-ec-retry-transition-lease-python-contract"
)
ASSERTIONS = (
    'a retry contends on the same in-flight Python EC lease',
    'the owner alone reaches the exact WI close continuation',
)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
