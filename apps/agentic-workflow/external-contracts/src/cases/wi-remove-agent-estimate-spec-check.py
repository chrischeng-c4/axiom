"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-remove-agent-estimate-spec-check"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "capability-to-epic-planning"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_remove_agent_estimate_spec_check.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case wi-remove-agent-estimate-spec-check"
ASSERTIONS = ('legacy Agent Estimate input remains parseable but is inert and creates no readiness requirement',)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
