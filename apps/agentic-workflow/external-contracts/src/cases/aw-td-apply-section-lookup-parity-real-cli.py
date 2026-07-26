"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-td-apply-section-lookup-parity-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-apply-section-lookup-parity"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_td_apply_section_lookup_parity_real_cli.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --test cli_tests td_create_apply_normalizes_body_only_logic_then_advances_structured_unit_test -- --nocapture"
ASSERTIONS = ('the already-valid fixture passes aw td check with zero findings', 'missing and malformed payload attempts leave the spec byte-identical', 'body-only Logic applies with exactly one typed Logic wrapper', 'the next initialized payload is applicability/unit-test.json', 'structured Unit Test applies and dispatches contract Logic')
