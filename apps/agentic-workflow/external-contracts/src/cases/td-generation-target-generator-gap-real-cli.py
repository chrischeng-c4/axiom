"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-generator-gap-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "exact-generated-unit-target-ownership"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_generator_gap_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_gen_unsupported_owned_unit_fails_before_lifecycle_mutation -- --nocapture"
ASSERTIONS = ('the public binary emits a typed owned_generated_unit_unsupported HITL envelope', 'the stable unit ID, target, remediation command, and generator_gap reason are explicit', 'HEAD, branch, index, status, issue, and target bytes remain unchanged')
