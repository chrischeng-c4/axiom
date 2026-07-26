"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-epic-project-label-dispatch-focused"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-epic-project-label-dispatch"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_epic_project_label_dispatch_focused.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib epic_project_label_dispatch_ -- --nocapture"
ASSERTIONS = ('the #1511 project:pgpool fixture emits exactly aw wi atomize --project pgpool', 'app:mamba and lib:pg retain their existing atomize commands', 'missing, empty, and whitespace-only project labels return blocked/HITL remediation', 'a valid unregistered app:workbench identity emits aw conf init --project-label app:workbench', 'no tested envelope contains --project PROJECT')
