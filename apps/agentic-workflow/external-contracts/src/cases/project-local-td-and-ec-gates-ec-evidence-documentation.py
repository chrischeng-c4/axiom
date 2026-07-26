"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "project-local-td-and-ec-gates-ec-evidence-documentation"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "ec-evidence-documentation"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_project_local_td_and_ec_gates_ec_evidence_documentation.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case project-local-td-and-ec-gates-ec-evidence-documentation"
ASSERTIONS = ('EC documentation generation and drift checks are covered',)


def verify() -> list[str]:
    from migration_clusters.project_local_gates import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
