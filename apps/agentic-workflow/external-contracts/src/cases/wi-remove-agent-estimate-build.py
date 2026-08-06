"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-remove-agent-estimate-build"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_remove_agent_estimate_build.rs"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case wi-remove-agent-estimate-build"
)
ASSERTIONS = ('prioritization output contains no estimate field while retaining the bounded capability-to-epic planning result',)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
