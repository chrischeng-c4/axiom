"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "td-generation-target-generator-gap-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "exact-generated-unit-target-ownership"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_td_generation_target_generator_gap_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case td-generation-target-generator-gap-real-cli"
ASSERTIONS = ('the public binary emits a typed owned_generated_unit_unsupported HITL envelope', 'the stable unit ID, target, remediation command, and generator_gap reason are explicit', 'HEAD, branch, index, status, issue, and target bytes remain unchanged')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
