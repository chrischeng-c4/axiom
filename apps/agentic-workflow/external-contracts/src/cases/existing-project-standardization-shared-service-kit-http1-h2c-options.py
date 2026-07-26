"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-shared-service-kit-http1-h2c-options"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_shared_service_kit_http1_h2c_options.rs"
TARGET_COMMAND = "cargo test -p server-http --lib tests::serves_http1_and_h2c_on_one_listener_with_tunable_options -- --exact --nocapture"
ASSERTIONS = ('the shared HTTP runtime serves HTTP/1.1 and h2c on one real listener while accepting explicit HTTP/2 stream and drain options',)
