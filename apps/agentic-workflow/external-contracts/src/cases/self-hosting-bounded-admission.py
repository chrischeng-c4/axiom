"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-bounded-admission"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "efficiency"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/efficiency_self_hosting_bounded_admission.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_backlog_rejects_before_reviewed_graph_or_state_io -- --nocapture"
ASSERTIONS = ('backlog admission succeeds without a reviewed graph and never creates backlog state', 'repeated invocations emit byte-identical envelopes and leave both the repository tree and sentinel-seeded resolved runtime workspace byte-identical')
