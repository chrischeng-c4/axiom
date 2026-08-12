"""EC security case for #3092 -- fail-closed N-1-to-N release journeys.

Every expected value is an EC-owned literal from #3092 R1-R11 and AC2-AC6.
The case drives only pure design entry points: real availability, process
restarts, traffic, BackupSet I/O, and Kubernetes leadership remain runtime
oracles and are intentionally absent.
"""

from __future__ import annotations

from lumen.release_upgrade.admission import (
    decide_finalize,
    decide_image_observation,
    decide_lifecycle_operation,
    decide_membership_transition,
    decide_release_metadata,
    decide_rollout,
    decide_operator_handover,
)
from lumen.release_upgrade.compatibility import decide_restore
from lumen.release_upgrade.evidence import validate_run_evidence
from lumen.release_upgrade.spec import (
    BackupManifest,
    FinalizeRequest,
    ImageObservationRequest,
    LifecycleOperationRequest,
    MembershipTransitionRequest,
    OperatorHandoverRequest,
    ReleaseIdentity,
    ReleaseMetadataRequest,
    RolloutRequest,
    RunEvidenceManifest,
)
from lumen.release_upgrade.status import RolloutStatus
from lumen.release_upgrade.verdict import Rejected

MINIMUM_CHECKS = 23

RELEASE_UPGRADE_3092_SECURITY_MATRIX = (
    ("equal_generations_are_rejected", "generations_must_differ"),
    ("equal_generation_refusal_names_target_generation", "target.generation"),
    ("equal_digests_are_rejected", "digests_must_differ"),
    ("equal_digest_refusal_names_target_digest", "target.image_digest"),
    ("neighbouring_distinct_metadata_is_admitted", "admitted"),
    ("non_crd_first_rollout_is_rejected", "crd_must_precede_operator"),
    ("in_place_one_voter_replacement_is_rejected", "temporary_learner_required"),
    ("leader_before_follower_is_rejected", "followers_must_precede_leader"),
    ("digest_mismatch_is_blocked", "digest_mismatch"),
    ("digest_mismatch_records_pinned_and_observed_digests", ("sha256:target", "sha256:repointed")),
    ("digest_mismatch_is_non_authoritative", "non_authoritative"),
    ("every_typed_fault_is_halted_with_old_authority", (("halted", "N-1"), ("halted", "N-1"), ("halted", "N-1"), ("halted", "N-1"), ("halted", "N-1"), ("halted", "N-1"))),
    ("every_typed_fault_exposes_its_exact_blocked_phase", ("incompatible_image", "image_pull_or_readiness", "capacity", "operator_restart", "member_restart", "leader_transfer")),
    ("all_active_operation_kinds_conflict", ("conflict", "conflict", "conflict", "conflict")),
    ("no_active_operation_admits_exactly_one_competing_request", ("admitted", "conflict", "conflict", "conflict")),
    ("finalize_without_soak_or_verified_backup_names_each_missing_predicate", (("healthy_soak_required", "healthy_soak"), ("verified_backup_required", "verified_old_compatible_backup"))),
    ("healthy_verified_finalize_neighbour_is_admitted", "admitted"),
    ("finalized_status_refuses_n_minus_one_readiness_and_downgrade_and_preserves_matched_epochs", ("not_ready", "downgrade_rejected", ("public-v1", "durable-v1"))),
    ("multiple_operator_leaders_are_rejected_and_rollback_retains_crd_n", ("multiple_active_reconcilers", "N")),
    ("incomplete_evidence_returns_concrete_missing_keys", ("backup", "cleanup")),
    ("forward_incompatible_backup_is_rejected_with_typed_reason", "backup_not_forward_compatible"),
    ("malformed_release_metadata_is_rejected_with_typed_field_path", ("rejected", "target.binary_identity")),
    ("admitted_finalize_produces_finalized_status", "finalized"),
)


def _outcome(value) -> str:
    return value.reason.value if isinstance(value, Rejected) else "admitted"


def _identity(generation: str, digest: str) -> ReleaseIdentity:
    return ReleaseIdentity(generation=generation, image_digest=digest, binary_identity="lumen-bin", build_identity="build-id")


def verify_release_upgrade_3092_security() -> dict:
    checks = []
    previous = _identity("N-1", "sha256:previous")
    target = _identity("N", "sha256:target")

    # 1-5. R1 -- both independently forbidden metadata collisions identify
    # their unsafe target field; a distinct neighbouring journey remains valid.
    equal_generation = decide_release_metadata(ReleaseMetadataRequest(previous=previous, target=_identity("N-1", "sha256:target")))
    obs1 = _outcome(equal_generation); exp1 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[0][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = equal_generation.field_path if isinstance(equal_generation, Rejected) else "admitted"; exp2 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[1][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    equal_digest = decide_release_metadata(ReleaseMetadataRequest(previous=previous, target=_identity("N", "sha256:previous")))
    obs3 = _outcome(equal_digest); exp3 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[2][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = equal_digest.field_path if isinstance(equal_digest, Rejected) else "admitted"; exp4 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[3][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    distinct = decide_release_metadata(ReleaseMetadataRequest(previous=previous, target=target))
    obs5 = _outcome(distinct); exp5 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[4][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # R1 -- incomplete immutable identity metadata is a typed refusal with the
    # exact field the release producer must repair, rather than a generic fail.
    malformed = decide_release_metadata(ReleaseMetadataRequest(
        previous=previous,
        target=ReleaseIdentity(generation="N", image_digest="sha256:target", binary_identity="", build_identity="build-id"),
    ))
    obs22 = ("rejected" if isinstance(malformed, Rejected) else "admitted", malformed.field_path if isinstance(malformed, Rejected) else "")
    exp22 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[21][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    # 6-8. R2-R4 -- every ordering rule is exercised with a forbidden explicit
    # request, never through a convenient default.
    rollout = decide_rollout(RolloutRequest(crd_generation="N", operator_from="N-1", operator_to="N", operator_replicas=2, phases=("operator-N-1-to-N", "crd-N")))
    obs6 = _outcome(rollout); exp6 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[5][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    in_place = decide_membership_transition(MembershipTransitionRequest(voters=1, phases=("replace-voter",), learner_node="", voter_node="node-a"))
    obs7 = _outcome(in_place); exp7 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[6][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    leader_first = decide_membership_transition(MembershipTransitionRequest(voters=3, phases=("leader-2", "follower-0", "follower-1"), learner_node="node-d", voter_node="node-a"))
    obs8 = _outcome(leader_first); exp8 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[7][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-11. R6/AC2 -- a repointed image neither replaces the pin nor becomes
    # authoritative; both values are preserved for the blocked diagnosis.
    mismatch = decide_image_observation(ImageObservationRequest(requested_generation="N", pinned_digest="sha256:target", observed_digest="sha256:repointed"))
    obs9 = _outcome(mismatch); exp9 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[8][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = (mismatch.pinned_digest, mismatch.observed_digest) if not isinstance(mismatch, Rejected) else ()
    exp10 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[9][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = mismatch.authority if not isinstance(mismatch, Rejected) else "rejected"; exp11 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[10][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12-13. R7/AC2 -- each typed fault must itself derive a value-level halt
    # result and exact typed phase while retaining old authority.  Supplying no
    # blocked phase prevents the fixture from echoing an EC-provided value.
    faults = ("incompatible_image", "image_pull_or_readiness", "capacity", "operator_restart", "member_restart", "leader_transfer")
    statuses = tuple(RolloutStatus(authoritative_generation="N-1", fault=fault) for fault in faults)
    obs12 = tuple(("halted" if status.is_halted() else "not_halted", status.authoritative_generation) for status in statuses)
    exp12 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[11][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = tuple(status.blocked_phase for status in statuses)
    exp13 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[12][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14-15. R10 -- each conflict kind is fenced at the lifecycle entry point.
    # Competing requests begin from the same no-active state; the first admitted
    # request becomes active before each remaining contender is decided.
    conflicts = tuple(_outcome(decide_lifecycle_operation(LifecycleOperationRequest(kind=kind, active_operation="upgrade"))) for kind in ("capacity", "split", "restore", "delete"))
    obs14 = conflicts; exp14 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[13][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    active_operation = None
    contenders = ("capacity", "split", "restore", "delete")
    competing_outcomes = []
    for kind in contenders:
        contender = decide_lifecycle_operation(LifecycleOperationRequest(kind=kind, active_operation=active_operation))
        competing_outcomes.append(_outcome(contender))
        if not isinstance(contender, Rejected):
            active_operation = kind
    obs15 = tuple(competing_outcomes); exp15 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[14][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16-17. R8/AC5 -- each prerequisite has an independent typed refusal;
    # only the fully explicit neighbouring request can finalize.
    missing = (
        decide_finalize(FinalizeRequest(healthy_soak=False, verified_old_compatible_backup=True)),
        decide_finalize(FinalizeRequest(healthy_soak=True, verified_old_compatible_backup=False)),
    )
    obs16 = tuple((_outcome(item), item.field_path if isinstance(item, Rejected) else "") for item in missing)
    exp16 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[15][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    finalize = decide_finalize(FinalizeRequest(healthy_soak=True, verified_old_compatible_backup=True))
    obs17 = _outcome(finalize); exp17 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[16][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # R8/AC4 -- finalization is not merely admitted: the admitted verdict
    # produces the terminal state whose public downgrade boundary is enforced.
    finalized_status = finalize.status if not isinstance(finalize, Rejected) else None
    obs23 = "finalized" if finalized_status is not None and finalized_status.finalized else "not_finalized"
    exp23 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[22][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 18. AC4/R8 -- the status produced by finalization cannot make an N-1
    # member ready or permit downgrade, and exposes matched public/durable epochs.
    status = finalized_status
    obs18 = ("ready" if status.can_ready("N-1") else "not_ready", "downgrade_allowed" if status.can_downgrade() else "downgrade_rejected", (status.public_epoch, status.durable_epoch))
    exp18 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[17][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. AC6 -- a pure handover rejects overlap and keeps CRD N in the
    # rollback state instead of modelling a real Lease or API-server write.
    handover = decide_operator_handover(OperatorHandoverRequest(active_leaders=("N-1", "N"), rollback_crd_generation="N"))
    obs19 = (_outcome(handover), handover.rollback_crd_generation if isinstance(handover, Rejected) else "admitted")
    exp19 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[18][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R11 -- missing evidence is returned as concrete keys, not a boolean
    # completeness claim that a caller cannot repair.
    incomplete_evidence = validate_run_evidence(RunEvidenceManifest(keys=("release", "traffic", "api_format", "kubernetes")))
    obs20 = incomplete_evidence.missing_keys if not isinstance(incomplete_evidence, Rejected) else ()
    exp20 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[19][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21. R9/AC5 -- the same restore entry point fails closed for a backup
    # whose durable epoch the supported forward runtime cannot read.
    incompatible_backup = decide_restore(BackupManifest(public_epoch="public-v1", durable_epoch="durable-v2", runtime_public_epoch="public-v1", runtime_durable_epoch="durable-v1"))
    obs21 = _outcome(incompatible_backup)
    exp21 = RELEASE_UPGRADE_3092_SECURITY_MATRIX[20][1]
    checks.append({"name": RELEASE_UPGRADE_3092_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    return {"case_id": "release-upgrade-3092-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
