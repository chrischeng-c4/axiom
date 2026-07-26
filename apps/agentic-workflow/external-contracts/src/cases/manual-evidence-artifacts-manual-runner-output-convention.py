"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "manual-evidence-artifacts-manual-runner-output-convention"
CAPABILITY_ID = "manual-evidence-artifacts"
USE_CASE_ID = "manual-runner-output-convention"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_manual_evidence_artifacts_manual_runner_output_convention.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case manual-evidence-artifacts-manual-runner-output-convention"
ASSERTIONS = ('EC doc generation writes the manual from inventory',)


def verify() -> list[str]:
    from migration_clusters.capability_manual import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
