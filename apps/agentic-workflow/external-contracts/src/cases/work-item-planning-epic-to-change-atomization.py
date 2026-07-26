"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "work-item-planning-epic-to-change-atomization"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-to-change-atomization"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_work_item_planning_epic_to_change_atomization.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case work-item-planning-epic-to-change-atomization"
ASSERTIONS = ('prioritization routes bounded changes into the ready lane',)


def verify() -> list[str]:
    from migration_clusters.work_item_planning import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
