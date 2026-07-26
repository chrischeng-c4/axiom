"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "self-hosting-identity-stability"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "self-hosting-root-runner-policy"
DIMENSION = "stability"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/stability_self_hosting_identity_stability.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_wi_identity_resolution_errors_fail_closed_without_mutation -- --nocapture"
ASSERTIONS = ('a malformed self-hosting WI identity returns a process error instead of entering the root runner', 'the failed resolution creates no loop state and leaves both the repository tree and resolved runtime workspace byte-identical')
