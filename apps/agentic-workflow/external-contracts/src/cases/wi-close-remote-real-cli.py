"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "wi-close-remote-real-cli"
CAPABILITY_ID = "work-item-planning"
USE_CASE_ID = "wi-close-remote-rehydration"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_wi_close_remote_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests wi_close_remote_ -- --nocapture"
ASSERTIONS = ('the repo-built aw binary resolves a tracker-only numeric issue through the configured GitHub backend', '--repo selects every remote read and mutation', 'the optional reason and close mutation each occur exactly once across a retry', 'a missing remote names its backend and repository and emits an executable recovery command', 'a local-only issue still moves from open to closed')
