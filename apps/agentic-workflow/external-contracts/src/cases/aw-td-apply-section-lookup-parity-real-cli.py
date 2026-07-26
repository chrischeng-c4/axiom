"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "aw-td-apply-section-lookup-parity-real-cli"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "td-apply-section-lookup-parity"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_aw_td_apply_section_lookup_parity_real_cli.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case aw-td-apply-section-lookup-parity-real-cli"
ASSERTIONS = ('the already-valid fixture passes aw td check with zero findings', 'missing and malformed payload attempts leave the spec byte-identical', 'body-only Logic applies with exactly one typed Logic wrapper', 'the next initialized payload is applicability/unit-test.json', 'structured Unit Test applies and dispatches contract Logic')


def verify() -> list[str]:
    from migration_clusters.td_source_target import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
