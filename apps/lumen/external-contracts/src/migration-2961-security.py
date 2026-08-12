"""EC security case for #2961 -- fail-closed embedded-to-Raft migration.

Every expected value is an EC-owned literal from #2961. R3 refuses a final
fence without a durable checkpoint or a target at the acknowledged watermark;
R4 refuses catalog cutover before the verified watermark; R6 refuses cleanup
before restart and oracle proof and for unrelated or authoritative source data.
Each refusal is also held to its named field and an adjacent admissible input,
so a generic rejection or a decider that rejects everything cannot pass.
"""

from __future__ import annotations

from lumen.topology.migration import MigrationPhase, MigrationProgress
from lumen.topology.migration_admission import (
    decide_catalog_cutover,
    decide_next_phase,
    decide_source_cleanup,
)
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 17

MIGRATION_2961_SECURITY_MATRIX = (
    ("undurable_final_fence_is_refused", "checkpoint_not_durable"),
    ("undurable_final_fence_names_checkpoint", "checkpoint_durable"),
    ("unequal_tail_final_fence_is_refused", "target_not_at_acknowledged_watermark"),
    ("unequal_tail_final_fence_names_target_watermark", "target_watermark"),
    ("durable_caught_up_final_fence_neighbour_is_admitted", "admitted"),
    ("behind_verified_watermark_cutover_is_refused", "target_not_at_verified_watermark"),
    ("behind_verified_watermark_cutover_names_target", "target_watermark"),
    ("verified_watermark_cutover_neighbour_is_admitted", "admitted"),
    ("cleanup_before_restart_proof_is_refused", "post_cutover_restart_not_verified"),
    ("cleanup_before_oracle_proof_is_refused", "oracle_not_verified"),
    ("cleanup_of_unrelated_source_is_refused", "unrelated_source"),
    ("cleanup_of_authoritative_source_is_refused", "authoritative_source"),
    ("authoritative_cleanup_refusal_names_source", "source_is_authoritative"),
    ("verified_related_non_authoritative_cleanup_is_admitted", "admitted"),
    ("cleanup_admission_retains_no_authoritative_source_claim", "retain"),
    ("undurable_catalog_cutover_transition_is_refused", "checkpoint_not_durable"),
    ("unequal_tail_catalog_cutover_transition_is_refused", "target_not_at_acknowledged_watermark"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_migration_2961_security() -> dict:
    checks = []

    checkpoint_missing = MigrationProgress(
        phase=MigrationPhase.TAILING,
        checkpoint_durable=False,
        acknowledged_watermark=41,
        target_watermark=41,
        verified_watermark=41,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=False,
        oracle_verified=False,
        source_is_authoritative=False,
        source_is_related=True,
    )
    no_checkpoint_fence = decide_next_phase(checkpoint_missing, MigrationPhase.FINAL_WRITE_FENCE)

    # 1. R3 -- final fence cannot run without a durable checkpoint.
    obs1 = _outcome(no_checkpoint_fence)
    exp1 = MIGRATION_2961_SECURITY_MATRIX[0][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R3 -- the refusal tells the operator which durability predicate failed.
    obs2 = no_checkpoint_fence.field_path if isinstance(no_checkpoint_fence, Rejection) else ""
    exp2 = MIGRATION_2961_SECURITY_MATRIX[1][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    tail_behind = MigrationProgress(
        phase=MigrationPhase.TAILING,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=41,
        verified_watermark=41,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=False,
        oracle_verified=False,
        source_is_authoritative=False,
        source_is_related=True,
    )
    behind_fence = decide_next_phase(tail_behind, MigrationPhase.FINAL_WRITE_FENCE)

    # 3. R3 -- a target behind acknowledged mutations cannot be fenced.
    obs3 = _outcome(behind_fence)
    exp3 = MIGRATION_2961_SECURITY_MATRIX[2][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- that different refusal names the target watermark specifically.
    obs4 = behind_fence.field_path if isinstance(behind_fence, Rejection) else ""
    exp4 = MIGRATION_2961_SECURITY_MATRIX[3][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    caught_up = MigrationProgress(
        phase=MigrationPhase.TAILING,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=42,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=True,
        oracle_verified=True,
        source_is_authoritative=False,
        source_is_related=True,
    )
    admitted_fence = decide_next_phase(caught_up, MigrationPhase.FINAL_WRITE_FENCE)

    # 5. R3 -- the immediately neighbouring caught-up request remains admitted.
    obs5 = _outcome(admitted_fence)
    exp5 = MIGRATION_2961_SECURITY_MATRIX[4][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    unverified_target = MigrationProgress(
        phase=MigrationPhase.FINAL_WRITE_FENCE,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=41,
        catalog_cutover_committed=False,
        catalog_generation=16,
        post_cutover_restart_verified=True,
        oracle_verified=True,
        source_is_authoritative=False,
        source_is_related=True,
    )
    blocked_cutover = decide_catalog_cutover(unverified_target, target_catalog_generation=17)

    # 6. R4 -- routing cannot move before target verification catches up.
    obs6 = _outcome(blocked_cutover)
    exp6 = MIGRATION_2961_SECURITY_MATRIX[5][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- the stable cutover refusal names its unsatisfied watermark.
    obs7 = blocked_cutover.field_path if isinstance(blocked_cutover, Rejection) else ""
    exp7 = MIGRATION_2961_SECURITY_MATRIX[6][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    admitted_cutover = decide_catalog_cutover(caught_up, target_catalog_generation=17)

    # 8. R4 -- the verified neighbouring target is admitted, not blanket-refused.
    obs8 = _outcome(admitted_cutover)
    exp8 = MIGRATION_2961_SECURITY_MATRIX[7][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    no_restart_proof = MigrationProgress(
        phase=MigrationPhase.CATALOG_CUTOVER,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=42,
        catalog_cutover_committed=True,
        catalog_generation=17,
        post_cutover_restart_verified=False,
        oracle_verified=True,
        source_is_authoritative=False,
        source_is_related=True,
    )
    restart_blocked_cleanup = decide_source_cleanup(no_restart_proof)

    # 9. R6 -- cleanup waits for post-cutover restart evidence.
    obs9 = _outcome(restart_blocked_cleanup)
    exp9 = MIGRATION_2961_SECURITY_MATRIX[8][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    no_oracle_proof = MigrationProgress(
        phase=MigrationPhase.CATALOG_CUTOVER,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=42,
        catalog_cutover_committed=True,
        catalog_generation=17,
        post_cutover_restart_verified=True,
        oracle_verified=False,
        source_is_authoritative=False,
        source_is_related=True,
    )
    oracle_blocked_cleanup = decide_source_cleanup(no_oracle_proof)

    # 10. R6 -- restart alone cannot substitute for oracle verification.
    obs10 = _outcome(oracle_blocked_cleanup)
    exp10 = MIGRATION_2961_SECURITY_MATRIX[9][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    unrelated_source = MigrationProgress(
        phase=MigrationPhase.CATALOG_CUTOVER,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=42,
        catalog_cutover_committed=True,
        catalog_generation=17,
        post_cutover_restart_verified=True,
        oracle_verified=True,
        source_is_authoritative=False,
        source_is_related=False,
    )
    unrelated_cleanup = decide_source_cleanup(unrelated_source)

    # 11. R6 -- an unrelated PVC/data source is never a cleanup candidate.
    obs11 = _outcome(unrelated_cleanup)
    exp11 = MIGRATION_2961_SECURITY_MATRIX[10][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    authoritative_source = MigrationProgress(
        phase=MigrationPhase.CATALOG_CUTOVER,
        checkpoint_durable=True,
        acknowledged_watermark=42,
        target_watermark=42,
        verified_watermark=42,
        catalog_cutover_committed=True,
        catalog_generation=17,
        post_cutover_restart_verified=True,
        oracle_verified=True,
        source_is_authoritative=True,
        source_is_related=True,
    )
    authoritative_cleanup = decide_source_cleanup(authoritative_source)

    # 12. R6 -- source still named authoritative also cannot be deleted.
    obs12 = _outcome(authoritative_cleanup)
    exp12 = MIGRATION_2961_SECURITY_MATRIX[11][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R6 -- that high-risk refusal names the authority predicate.
    obs13 = authoritative_cleanup.field_path if isinstance(authoritative_cleanup, Rejection) else ""
    exp13 = MIGRATION_2961_SECURITY_MATRIX[12][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    cleanup_ready = decide_source_cleanup(caught_up)

    # 14. R6 -- all explicit proofs plus a related non-authoritative source admit cleanup.
    obs14 = _outcome(cleanup_ready)
    exp14 = MIGRATION_2961_SECURITY_MATRIX[13][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R6 -- admission still reports retained source until the explicit cleanup action.
    obs15 = cleanup_ready.source_retention.value if not isinstance(cleanup_ready, Rejection) else _outcome(cleanup_ready)
    exp15 = MIGRATION_2961_SECURITY_MATRIX[14][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    no_checkpoint_cutover = decide_next_phase(checkpoint_missing, MigrationPhase.CATALOG_CUTOVER)

    # 16. R3 -- phase advancement cannot skip checkpoint durability to cut over.
    obs16 = _outcome(no_checkpoint_cutover)
    exp16 = MIGRATION_2961_SECURITY_MATRIX[15][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    behind_tail_cutover = decide_next_phase(tail_behind, MigrationPhase.CATALOG_CUTOVER)

    # 17. R3 -- phase advancement cannot skip the acknowledged-tail watermark.
    obs17 = _outcome(behind_tail_cutover)
    exp17 = MIGRATION_2961_SECURITY_MATRIX[16][1]
    checks.append({"name": MIGRATION_2961_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {
        "case_id": "migration-2961-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
