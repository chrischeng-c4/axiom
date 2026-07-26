"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "quality-primitive-metadata-contract-test"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "quality-primitive-metadata-contract-test"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_quality_primitive_metadata_contract_test.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib quality_primitives -- --nocapture"
ASSERTIONS = ('default profiles validate', 'selection citation explains a matching primitive', 'review finding reports an anti-pattern')
