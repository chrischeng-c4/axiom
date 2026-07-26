"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "existing-project-standardization-shared-service-kit-drain"
CAPABILITY_ID = "existing-project-standardization"
USE_CASE_ID = "shared-service-kit-substrate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_existing_project_standardization_shared_service_kit_drain.rs"
TARGET_COMMAND = "cargo test -p server-lifecycle --test drain_prestart receiverless_drain_persists_for_late_subscriber -- --exact --nocapture"
ASSERTIONS = ('a drain transition published before subscription remains durable and is observed by a late server-plane subscriber',)
