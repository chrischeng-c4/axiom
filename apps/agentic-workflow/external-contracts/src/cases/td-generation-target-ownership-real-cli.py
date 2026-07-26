"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-ownership-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "ambiguous-multi-target-generation-preflight"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_ownership_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_gen_ambiguous_schema_plan_fails_before_any_lifecycle_mutation -- --nocapture"
ASSERTIONS = ('the public binary emits exactly one stdout JSON error envelope and no second stderr error', 'error_kind, section, sorted targets, completion, and executable next.command are stable', 'HEAD, symbolic branch, index tree, porcelain-z status, issue bytes, and TD branch ref are unchanged', 'the prepared spec and every target blob remain byte-identical')
