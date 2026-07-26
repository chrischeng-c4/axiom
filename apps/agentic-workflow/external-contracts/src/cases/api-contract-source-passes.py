"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "api-contract-source-passes"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "api-contract-source-passes"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_api_contract_source_passes.rs"
TARGET_COMMAND = "cargo test -p agentic-workflow --lib source_reference_api_contract_source_backed -- --nocapture"
ASSERTIONS = ('source_backed is true', 'findings is empty')
