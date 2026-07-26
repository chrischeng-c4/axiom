"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-shared-service-kit-service-http-delegation"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_shared_service_kit_service_http_delegation.rs"
TARGET_COMMAND = "cargo test -p service-http --lib transport::delegation_tests::serve_delegates_listener_to_shared_http_runtime -- --exact --nocapture"
ASSERTIONS = ('the service-http policy shell delegates listener ownership and request dispatch to server-http while preserving the service router response',)
