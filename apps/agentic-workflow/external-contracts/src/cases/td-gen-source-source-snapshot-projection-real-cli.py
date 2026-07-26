"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-gen-source-source-snapshot-projection-real-cli"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "authoritative-source-snapshot-projection"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_gen_source_source_snapshot_projection_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests test_gen_source_projects_legacy_snapshot_and_runs_generated_test -- --nocapture"
ASSERTIONS = ('a const changes from before to after in the exact requested target', 'a uniquely named generated Rust test is present in exact target bytes', 'cargo test with that unique filter reports running 1 test and 1 passed', 'siblings and an unmatched existing target remain byte-identical', 'a second replay reports summary.wrote_files=false', 'the unmatched target error names the snapshot target and runnable --target remediation')
