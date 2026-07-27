"""Black-box contract for canonical typed priority-label authoring."""

CASE_ID = "wi-typed-priority-label"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "typed-priority-label-authoring"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case wi-typed-priority-label"
)
ASSERTIONS = (
    "create help documents priority:<value>",
    "typed priority emits priority:p2",
    "typed priority never emits priority::p2",
)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
