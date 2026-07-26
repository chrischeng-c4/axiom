"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-capability-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_self_hosting_capability_admission.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_project_capability_and_backlog_roots_are_rejected_before_mutation -- --nocapture"
ASSERTIONS = ('project, capability, and backlog roots emit action self_hosting_policy and policy_mode sanctioned_direct_commit', 'the envelopes expose no invoke command and both the repository tree and resolved runtime workspace remain byte-identical')
