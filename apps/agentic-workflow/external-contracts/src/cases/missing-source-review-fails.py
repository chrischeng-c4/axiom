"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "missing-source-review-fails"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "missing-source-review-fails"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_missing_source_review_fails.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case missing-source-review-fails"
ASSERTIONS = ('source_backed is false', 'finding code is missing_source_reference')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
