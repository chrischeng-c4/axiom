"""Canonical Python EC declaration migrated from the reviewed legacy inventory."""

CASE_ID = "terminal-touched-codegen-red-repair-green"
CAPABILITY_ID = "td-cb-lifecycle-automation"
USE_CASE_ID = "terminal-touched-codegen-drift-gate"
DIMENSION = "behavior"
LEGACY_TEST_PATH = "apps/agentic-workflow/tests/behavior_terminal_touched_codegen_red_repair_green.rs"
TARGET_COMMAND = "python3 apps/agentic-workflow/external-contracts/src/runner.py --case terminal-touched-codegen-red-repair-green"
ASSERTIONS = ('committed accepted CODEGEN drift refuses before EC and leaves phase, state, issue bytes, HEAD, index tree, cached diff, status, and target bytes unchanged', 'the finding names only the accepted target and exact spec section while a second unaccepted generated target remains drifted', 'the emitted aw cb gen slug command regenerates and commits only the accepted target, preserves terminal phase, and emits the exact retry command', 'restored parity runs EC once, closes the WI, and a td_merged retry neither reruns EC nor duplicates the terminal commit')


def verify() -> list[str]:
    from migration_clusters.td_terminal import verify as verify_cluster_case

    return verify_cluster_case(CASE_ID)
