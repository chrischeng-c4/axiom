"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "work-item-planning-epic-to-change-atomization"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "epic-to-change-atomization"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_work_item_planning_epic_to_change_atomization.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib prioritize_lanes_put_bounded_bug_in_ready_now -- --nocapture"
ASSERTIONS = ('prioritization routes bounded changes into the ready lane',)
