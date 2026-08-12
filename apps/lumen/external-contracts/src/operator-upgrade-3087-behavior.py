"""EC behavior case for #3087 -- CRD-first N/N-1 operator handover.

Every expected value below is an EC-owned literal transcribed from #3087:
R1 admits only an established additive CRD; R2 and R6 derive exactly one
reconciliation owner; R3 preserves unowned CR spec and status; R4 changes a
managed image only for an explicit desired-image change; R5 admits compatible
rollback; and R7 fixes the CRD/operator/instance artifact sequence.
"""

from __future__ import annotations

from lumen.operator_upgrade.admission import decide_operator_admission
from lumen.operator_upgrade.handover import decide_owner
from lumen.operator_upgrade.order import artifact_order
from lumen.operator_upgrade.reconciliation import apply_owned_patch
from lumen.operator_upgrade.release import decide_rollout
from lumen.operator_upgrade.rollback import decide_rollback
from lumen.operator_upgrade.spec import (
    CrdAdmissionRequest,
    HandoverState,
    ImageReleaseRequest,
    ManagedCustomResource,
    ManagedInstance,
    OperatorReplica,
    OwnedPatch,
    RollbackRequest,
)
from lumen.operator_upgrade.verdict import Refused

MINIMUM_CHECKS = 11

OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX = (
    ("established_additive_crd_admits_operator_n", "admitted"),
    ("healthy_lease_has_n_as_sole_reconciliation_owner", "N"),
    ("owned_patch_preserves_unowned_spec_field", "retain-me"),
    ("owned_patch_preserves_unknown_additive_spec_field", "future-spec"),
    ("owned_patch_preserves_unowned_status_field", "ready"),
    ("owned_patch_preserves_unknown_additive_status_field", "future-status"),
    ("same_image_does_not_initiate_a_rollout", "no_rollout"),
    ("explicit_image_change_initiates_a_rollout", "rollout"),
    ("compatible_instances_admit_rollback", "admitted"),
    ("n_leader_loss_hands_reconciliation_to_n_minus_one", "N-1"),
    ("artifacts_are_ordered_crd_operator_instance", ("crd", "operator", "instance")),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Refused) else "admitted"


def verify_operator_upgrade_3087_behavior() -> dict:
    checks = []

    # 1. R1 -- N is admitted only after the candidate CRD is both established
    #    and additive relative to the active version.
    admitted = decide_operator_admission(
        CrdAdmissionRequest(established=True, additive_to_active=True)
    )
    obs1 = _outcome(admitted)
    exp1 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2 -- an ordinary N-held Lease produces exactly the N reconciliation
    #    owner, rather than allowing both rollout versions to write.
    healthy_owner = decide_owner(
        HandoverState(
            replicas=(
                OperatorReplica(version="N-1", available=True, holds_lease=False),
                OperatorReplica(version="N", available=True, holds_lease=True),
            )
        )
    )
    obs2 = healthy_owner.owner_version if not isinstance(healthy_owner, Refused) else healthy_owner.reason.value
    exp2 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    resource = ManagedCustomResource(
        spec={"owned": "old", "unowned": "retain-me", "future": "future-spec"},
        status={"phase": "ready", "future_status": "future-status", "owned_status": "old-status"},
    )
    patched = apply_owned_patch(
        resource,
        OwnedPatch(spec={"owned": "new"}, status={"owned_status": "new-status"}),
    )

    # 3. R3 -- N's patch does not erase an N-1 spec field it does not own.
    obs3 = patched.spec["unowned"]
    exp3 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3 -- unknown additive fields are also unowned and must survive.
    obs4 = patched.spec["future"]
    exp4 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- status has the same preservation rule as spec.
    obs5 = patched.status["phase"]
    exp5 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 -- an unknown additive status field is no more disposable than an
    #    unknown additive spec field.
    obs6 = patched.status["future_status"]
    exp6 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- matching current and desired images are a no-op.
    unchanged = decide_rollout(
        ImageReleaseRequest(current_image="lumen:vN-1", desired_image="lumen:vN-1")
    )
    obs7 = unchanged.action
    exp7 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R4 -- changing the desired image is the only modeled rollout trigger.
    changed = decide_rollout(
        ImageReleaseRequest(current_image="lumen:vN-1", desired_image="lumen:vN")
    )
    obs8 = changed.action
    exp8 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R5 -- every compatible managed instance permits a binary rollback.
    rollback = decide_rollback(
        RollbackRequest(
            instances=(
                ManagedInstance(instance_id="orders-0", pre_finalize=True, requires_n_only_behavior=False),
                ManagedInstance(instance_id="orders-1", pre_finalize=True, requires_n_only_behavior=False),
            )
        )
    )
    obs9 = _outcome(rollback)
    exp9 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R6 -- after N loses leadership, the still-available N-1 replica is
    #    the sole owner rather than an uncoordinated concurrent writer.
    failover_owner = decide_owner(
        HandoverState(
            replicas=(
                OperatorReplica(version="N-1", available=True, holds_lease=True),
                OperatorReplica(version="N", available=False, holds_lease=False),
            )
        )
    )
    obs10 = failover_owner.owner_version if not isinstance(failover_owner, Refused) else failover_owner.reason.value
    exp10 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- the pure model owns only this immutable abstract order; CLI
    #     rendering and documentation remain Rust integration evidence.
    obs11 = artifact_order()
    exp11 = OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "operator-upgrade-3087-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
