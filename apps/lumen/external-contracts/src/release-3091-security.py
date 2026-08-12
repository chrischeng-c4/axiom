"""EC security case for #3091 -- release finalization fails closed.

Expected values are EC-owned literals from #3091 R1/R2/R5-R9/R11.  Each
refusal supplies the forbidden input explicitly, checks its typed reason and
the field it names where the issue requires an actionable refusal, and keeps a
neighbouring valid request admitted.  Runtime durability and traffic evidence
are intentionally outside this pure design contract.
"""

from __future__ import annotations

from lumen.release.admission import (
    decide_activation,
    decide_evolution,
    decide_finalize,
    decide_member_admission,
    decide_reader_version,
    select_write_format,
)
from lumen.release.spec import BackupSetAttestation, CompatibilityDescriptor, ReleaseSpec
from lumen.release.status import ReleaseStatus
from lumen.release.transition import decide_rollback
from lumen.release.verdict import Rejection

MINIMUM_CHECKS = 24

RELEASE_3091_SECURITY_MATRIX = (
    ("no_common_member_format_is_rejected", "no_common_write_format"),
    ("no_common_member_format_names_admitted_member_formats", "admitted_member_formats"),
    ("unknown_reader_version_is_rejected", "unknown_or_corrupt_version"),
    ("unknown_reader_version_names_encoded_version", "encoded_version"),
    ("current_reader_version_neighbour_is_admitted", "admitted"),
    ("digest_mismatch_blocks_activation", "pinned_digest_mismatch"),
    ("digest_mismatch_names_pinned_digest", "pinned_digest"),
    ("unconverged_topology_blocks_activation", "topology_not_converged"),
    ("active_operation_blocks_activation", "operation_not_exclusive"),
    ("missing_member_report_blocks_activation", "required_member_reports_missing"),
    ("unauthorized_target_epochs_block_activation", "target_epochs_unauthorized"),
    ("fully_authorized_activation_neighbour_is_admitted", "admitted"),
    ("missing_finalize_generation_is_rejected", "finalize_generation_required"),
    ("missing_finalize_generation_names_finalize_generation", "finalize_generation"),
    ("stale_finalize_generation_is_rejected", "stale_finalize_generation"),
    ("unsatisfied_soak_is_rejected", "healthy_traffic_soak_unsatisfied"),
    ("incomplete_backup_set_is_rejected", "backup_set_incomplete"),
    ("backup_at_wrong_write_epoch_is_rejected", "backup_write_epoch_mismatch"),
    ("complete_old_epoch_backup_neighbour_is_admitted", "admitted"),
    ("post_finalize_rollback_is_rejected", "forward_recovery_required"),
    ("post_finalize_rollback_names_forward_recovery", "forward_recovery"),
    ("finalized_state_rejects_n_minus_one_member", "n_minus_one_downgrade_rejected"),
    ("n_minus_one_rejection_names_candidate_release", "candidate_release"),
    ("destructive_or_unversioned_evolution_is_rejected", "destructive_or_unversioned_evolution"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _descriptor(*, dual_readable: bool = True, n_operations: tuple[str, ...] = ("search", "index-v2")) -> CompatibilityDescriptor:
    return CompatibilityDescriptor(
        n_minus_one_release="N-1", n_release="N",
        n_minus_one_readable_formats=(1,), n_readable_formats=(1, 2),
        n_minus_one_api_epoch=1, n_api_epoch=2,
        n_minus_one_operations=("search",), n_operations=n_operations,
        current_reader_version=2, previous_reader_version=1,
        dual_readable=dual_readable,
    )


def _spec(*, digest: str = "sha256:n-release", finalize_generation: int | None = 12) -> ReleaseSpec:
    return ReleaseSpec(target_release="N", target_write_epoch=2, target_api_epoch=2, pinned_digest=digest, finalize_generation=finalize_generation)


def _status(
    *, reported_digest: str = "sha256:n-release", converged: bool = True,
    operation_free: bool = True, member_reports: bool = True, authorized: bool = True,
    soak: bool = True, backup_complete: bool = True, backup_epoch: int = 1,
    generation: int = 11, finalized: bool = False,
) -> ReleaseStatus:
    return ReleaseStatus(
        active_write_epoch=2 if finalized else 1, active_api_epoch=2 if finalized else 1,
        generation=generation, finalized=finalized, required_member_reports=member_reports,
        topology_converged=converged, operation_free=operation_free,
        reported_digest=reported_digest, compatibility_authorized=authorized,
        healthy_traffic_soak=soak,
        backup_set=BackupSetAttestation(
            backup_id="backup-n-old-epoch", complete=backup_complete, verified=True,
            topology_release="N", write_epoch=backup_epoch, forward_recovery_point=True,
        ),
    )


def verify_release_3091_security() -> dict:
    checks = []
    descriptor = _descriptor()
    spec = _spec()
    ready = _status()

    # 1-2. R1 -- format selection rejects an explicit empty intersection.
    no_format = select_write_format(descriptor, ((1,), (2,)))
    obs1 = _outcome(no_format); exp1 = RELEASE_3091_SECURITY_MATRIX[0][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = no_format.field_path if isinstance(no_format, Rejection) else ""; exp2 = RELEASE_3091_SECURITY_MATRIX[1][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-5. R2 -- every version outside current/previous fails closed, while current remains valid.
    unknown = decide_reader_version(descriptor, 99)
    obs3 = _outcome(unknown); exp3 = RELEASE_3091_SECURITY_MATRIX[2][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = unknown.field_path if isinstance(unknown, Rejection) else ""; exp4 = RELEASE_3091_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    current = decide_reader_version(descriptor, 2)
    obs5 = _outcome(current); exp5 = RELEASE_3091_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6-12. R5 -- each activation authorization predicate independently fences the target.
    bad_digest = decide_activation(spec, _status(reported_digest="sha256:other"), descriptor)
    obs6 = _outcome(bad_digest); exp6 = RELEASE_3091_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = bad_digest.field_path if isinstance(bad_digest, Rejection) else ""; exp7 = RELEASE_3091_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    unconverged = decide_activation(spec, _status(converged=False), descriptor)
    obs8 = _outcome(unconverged); exp8 = RELEASE_3091_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    active_operation = decide_activation(spec, _status(operation_free=False), descriptor)
    obs9 = _outcome(active_operation); exp9 = RELEASE_3091_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    missing_reports = decide_activation(spec, _status(member_reports=False), descriptor)
    obs10 = _outcome(missing_reports); exp10 = RELEASE_3091_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    unauthorized = decide_activation(spec, _status(authorized=False), descriptor)
    obs11 = _outcome(unauthorized); exp11 = RELEASE_3091_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    admitted = decide_activation(spec, ready, descriptor)
    obs12 = _outcome(admitted); exp12 = RELEASE_3091_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-19. R6/R7 -- finalize cannot substitute rollout completion for its generation, soak, or BackupSet gates.
    no_generation = decide_finalize(_spec(finalize_generation=None), ready, descriptor)
    obs13 = _outcome(no_generation); exp13 = RELEASE_3091_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = no_generation.field_path if isinstance(no_generation, Rejection) else ""; exp14 = RELEASE_3091_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    stale_generation = decide_finalize(_spec(finalize_generation=11), ready, descriptor)
    obs15 = _outcome(stale_generation); exp15 = RELEASE_3091_SECURITY_MATRIX[14][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    no_soak = decide_finalize(spec, _status(soak=False), descriptor)
    obs16 = _outcome(no_soak); exp16 = RELEASE_3091_SECURITY_MATRIX[15][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    incomplete = decide_finalize(spec, _status(backup_complete=False), descriptor)
    obs17 = _outcome(incomplete); exp17 = RELEASE_3091_SECURITY_MATRIX[16][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    wrong_epoch = decide_finalize(spec, _status(backup_epoch=2), descriptor)
    obs18 = _outcome(wrong_epoch); exp18 = RELEASE_3091_SECURITY_MATRIX[17][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    backup_neighbour = decide_finalize(spec, ready, descriptor)
    obs19 = _outcome(backup_neighbour); exp19 = RELEASE_3091_SECURITY_MATRIX[18][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20-23. R8/R9 -- point-of-no-return forbids both rollback and N-1 admission with guidance.
    finalized = _status(finalized=True, generation=12)
    rollback = decide_rollback(spec, finalized)
    obs20 = _outcome(rollback); exp20 = RELEASE_3091_SECURITY_MATRIX[19][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    obs21 = rollback.field_path if isinstance(rollback, Rejection) else ""; exp21 = RELEASE_3091_SECURITY_MATRIX[20][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    downgrade = decide_member_admission(finalized, "N-1")
    obs22 = _outcome(downgrade); exp22 = RELEASE_3091_SECURITY_MATRIX[21][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    obs23 = downgrade.field_path if isinstance(downgrade, Rejection) else ""; exp23 = RELEASE_3091_SECURITY_MATRIX[22][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 24. R11 -- a candidate that removes prior behavior and drops dual-readability is refused.
    destructive = decide_evolution(descriptor, _descriptor(dual_readable=False, n_operations=("index-v2",)))
    obs24 = _outcome(destructive); exp24 = RELEASE_3091_SECURITY_MATRIX[23][1]
    checks.append({"name": RELEASE_3091_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    return {"case_id": "release-3091-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
