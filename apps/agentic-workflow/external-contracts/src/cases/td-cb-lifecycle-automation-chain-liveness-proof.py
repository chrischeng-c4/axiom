"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-cb-lifecycle-automation-chain-liveness-proof"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "chain-liveness-proof"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_cb_lifecycle_automation_chain_liveness_proof.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests chain_liveness_test::chain_liveness_claim_never_lands_on_deadlock_phase -- --exact --nocapture"
ASSERTIONS = ('the exact driven chain reaches a terminal action within its bounded hop budget without landing on a deadlock phase (#914, refs #921)',)
