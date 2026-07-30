"""Cross-process single-flight contract for the terminal EC lease."""

CASE_ID = "terminal-ec-cross-process-single-flight-python-contract"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-ec-process-liveness"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case terminal-ec-cross-process-single-flight-python-contract"
)
ASSERTIONS = (
    'two real AW processes contend on one project EC lease',
    'the duplicate returns promptly and exactly one Python EC process launches',
)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
