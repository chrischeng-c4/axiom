"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "jet-health-verification-dedup-smoke"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "jet-health-verification-dedup-smoke"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_jet_health_verification_dedup_smoke.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib duplicate_claim_gate_commands_run_once_per_report_pass -- --nocapture"
ASSERTIONS = ('health command succeeds or reports only real project blockers', 'duplicate README gate commands do not multiply command execution inside one AW verification pass')
