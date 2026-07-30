"""Black-box contract for typed epic ownership authoring."""

CASE_ID = "wi-typed-epic-owner"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "typed-epic-owner-authoring"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case wi-typed-epic-owner"
)
ASSERTIONS = (
    "create help documents --epic",
    "typed owner emits epic:<id>",
    "created change resolves under its epic in the graph",
    "body-only parent compatibility still establishes ownership",
    "unowned create emits an exact actionable update command template",
    "resolving and executing the template assigns typed ownership",
    "invalid, cross-project, non-epic, and conflicting owners are rejected",
)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
