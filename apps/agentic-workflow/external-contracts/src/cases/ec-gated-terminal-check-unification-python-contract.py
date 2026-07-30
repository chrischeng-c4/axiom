"""EC-gated terminal check unification contract."""

CASE_ID = "ec-gated-terminal-check-unification-python-contract"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-surface-convergence-ec-gated-terminal-check-unification-verb-lifecycle-policy-fixture-loop-self-ec"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case ec-gated-terminal-check-unification-python-contract"
)
ASSERTIONS = (
    'the public Python EC seam routes red back to bounded CB regeneration',
    'the same WI converges to one exact terminal close continuation when green',
)


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
