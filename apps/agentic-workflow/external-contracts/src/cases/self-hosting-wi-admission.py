"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-wi-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_wi_admission.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_work_item_root_is_rejected_before_loop_state_or_dispatch -- --nocapture"
ASSERTIONS = ('the WI root emits action self_hosting_policy before loop state or dispatch', 'the envelope exposes no invoke command and both the repository tree and resolved runtime workspace remain byte-identical')
