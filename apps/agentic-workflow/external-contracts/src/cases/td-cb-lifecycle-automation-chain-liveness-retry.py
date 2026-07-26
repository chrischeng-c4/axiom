"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-chain-liveness-retry"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "chain-liveness-proof"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_chain_liveness_retry.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests chain_liveness_test::chain_liveness_code_check_retry_recovers_stranded_terminal_within_tick_budget -- --exact --nocapture"
ASSERTIONS = ('a stranded terminal retry emits the exact `aw cb check <slug>` command, remains within its tick budget, and preserves the lifecycle state until successful completion',)
