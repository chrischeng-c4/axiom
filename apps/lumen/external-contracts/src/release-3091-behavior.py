"""EC behavior case for #3091 -- gated, durable N release finalization.

Every expected value is an EC-owned literal from #3091 R1-R11.  These rows
exercise only the pure release design: format selection, reader admission,
status shape, public-surface selection, activation/finalization, rollback,
member admission, idempotent intents, and additive evolution.  Storage,
process, clock, network, and replay claims are deliberately runtime-only.
"""

from __future__ import annotations

from lumen.release.admission import (
    decide_activation,
    decide_evolution,
    decide_finalize,
    decide_member_admission,
    decide_reader_version,
    select_public_surface,
    select_write_format,
)
from lumen.release.spec import BackupSetAttestation, CompatibilityDescriptor, ReleaseSpec
from lumen.release.status import ReleaseStatus
from lumen.release.transition import apply_intent, decide_rollback
from lumen.release.verdict import Rejection

MINIMUM_CHECKS = 20

RELEASE_3091_BEHAVIOR_MATRIX = (
    ("mixed_members_select_the_newest_commonly_readable_write_format", 1),
    ("n_reader_admits_the_declared_current_format", "admitted"),
    ("n_reader_admits_the_declared_previous_format", "admitted"),
    ("status_persists_active_write_epoch", 1),
    ("status_persists_active_api_epoch", 1),
    ("status_persists_upgrade_generation", 11),
    ("mixed_serving_members_expose_the_api_intersection", ("search",)),
    ("fully_authorized_activation_is_admitted", "admitted"),
    ("finalize_with_explicit_generation_and_satisfied_soak_is_admitted", "admitted"),
    ("finalize_records_the_verified_backup_as_forward_recovery", "backup-n-old-epoch"),
    ("pre_finalize_rollback_restores_n_minus_one_api_epoch", 1),
    ("pre_finalize_rollback_restores_n_minus_one_write_epoch", 1),
    ("finalized_release_admits_the_n_member", "admitted"),
    ("finalized_surface_exposes_authorized_additive_n_operation", ("index-v2",)),
    ("finalized_surface_retains_prior_client_operation", ("search",)),
    ("repeated_activation_intent_has_no_additional_transition", "none"),
    ("repeated_activation_intent_returns_the_equal_state", True),
    ("repeated_finalize_intent_has_no_additional_transition", "none"),
    ("repeated_finalize_intent_returns_the_equal_state", True),
    ("additive_dual_readable_evolution_is_admitted", "admitted"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _descriptor(*, n_operations: tuple[str, ...] = ("search", "index-v2")) -> CompatibilityDescriptor:
    return CompatibilityDescriptor(
        n_minus_one_release="N-1",
        n_release="N",
        n_minus_one_readable_formats=(1,),
        n_readable_formats=(1, 2),
        n_minus_one_api_epoch=1,
        n_api_epoch=2,
        n_minus_one_operations=("search",),
        n_operations=n_operations,
        current_reader_version=2,
        previous_reader_version=1,
        dual_readable=True,
    )


def _spec(*, finalize_generation: int = 12) -> ReleaseSpec:
    return ReleaseSpec(
        target_release="N",
        target_write_epoch=2,
        target_api_epoch=2,
        pinned_digest="sha256:n-release",
        finalize_generation=finalize_generation,
    )


def _status(*, finalized: bool = False, generation: int = 11) -> ReleaseStatus:
    return ReleaseStatus(
        active_write_epoch=2 if finalized else 1,
        active_api_epoch=2 if finalized else 1,
        generation=generation,
        finalized=finalized,
        required_member_reports=True,
        topology_converged=True,
        operation_free=True,
        reported_digest="sha256:n-release",
        compatibility_authorized=True,
        healthy_traffic_soak=True,
        backup_set=BackupSetAttestation(
            backup_id="backup-n-old-epoch",
            complete=True,
            verified=True,
            topology_release="N",
            write_epoch=1,
            forward_recovery_point=True,
        ),
    )


def verify_release_3091_behavior() -> dict:
    checks = []
    descriptor = _descriptor()
    spec = _spec()
    staged = _status()

    # 1. R1 -- an N member may write 2, but N-1 can read only 1.
    obs1 = select_write_format(descriptor, ((1,), (1, 2)))
    exp1 = RELEASE_3091_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2-3. R2 -- the declared current and previous reader versions are both valid.
    current = decide_reader_version(descriptor, 2)
    obs2 = _outcome(current); exp2 = RELEASE_3091_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    previous = decide_reader_version(descriptor, 1)
    obs3 = _outcome(previous); exp3 = RELEASE_3091_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4-6. R3 -- the three durable topology/catalog dimensions remain distinct.
    obs4 = staged.active_write_epoch; exp4 = RELEASE_3091_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = staged.active_api_epoch; exp5 = RELEASE_3091_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = staged.generation; exp6 = RELEASE_3091_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- N-only operations remain absent while one N-1 server remains.
    mixed_surface = select_public_surface(descriptor, ("N-1", "N"))
    obs7 = mixed_surface.operations; exp7 = RELEASE_3091_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- every authorization predicate is explicitly present and true.
    activation = decide_activation(spec, staged, descriptor)
    obs8 = _outcome(activation); exp8 = RELEASE_3091_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. R6/R7 -- finalization needs both an explicit, fresh generation and
    # a verified complete recovery point at the old-compatible write epoch.
    finalization = decide_finalize(spec, staged, descriptor)
    obs9 = _outcome(finalization); exp9 = RELEASE_3091_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = finalization.forward_recovery_point if not isinstance(finalization, Rejection) else "rejected"
    exp10 = RELEASE_3091_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-12. R8 -- rollback remains the N-1-compatible handoff before finalization.
    rollback = decide_rollback(spec, staged)
    obs11 = rollback.target_api_epoch if not isinstance(rollback, Rejection) else -1
    exp11 = RELEASE_3091_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = rollback.target_write_epoch if not isinstance(rollback, Rejection) else -1
    exp12 = RELEASE_3091_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-15. R9 -- after finalization N is admitted and its additive surface
    # retains the prior client call rather than replacing it.
    finalized = _status(finalized=True, generation=12)
    n_member = decide_member_admission(finalized, "N")
    obs13 = _outcome(n_member); exp13 = RELEASE_3091_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    final_surface = select_public_surface(descriptor, ("N",))
    obs14 = final_surface.additive_operations
    exp14 = RELEASE_3091_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = final_surface.retained_operations
    exp15 = RELEASE_3091_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16-19. R10 -- repeated declarative activation and finalization are both idempotent.
    active_state = apply_intent(staged, "activate")
    repeated_active = apply_intent(active_state.state, "activate")
    obs16 = repeated_active.transition; exp16 = RELEASE_3091_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = repeated_active.state == active_state.state; exp17 = RELEASE_3091_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    finalized_state = apply_intent(finalized, "finalize")
    repeated_finalized = apply_intent(finalized_state.state, "finalize")
    obs18 = repeated_finalized.transition; exp18 = RELEASE_3091_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    obs19 = repeated_finalized.state == finalized_state.state; exp19 = RELEASE_3091_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R11 -- the advertised upgrade is additive and its formats remain dual-readable.
    evolution = decide_evolution(_descriptor(n_operations=("search",)), descriptor)
    obs20 = _outcome(evolution); exp20 = RELEASE_3091_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": RELEASE_3091_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    return {"case_id": "release-3091-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
