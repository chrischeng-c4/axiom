"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-shared-service-kit-substrate"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_shared_service_kit_substrate.rs"
TARGET_COMMAND = "cargo test -p server-tcp --lib tests::serve_accepts_closure_handler_without_async_trait_boxing -- --exact --nocapture"
ASSERTIONS = ('the shared TCP accept loop binds a real listener, admits a connection, invokes the closure handler, and completes without an async-trait box (#1241)',)
