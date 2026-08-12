"""EC security case for #3087 -- fail-closed operator handover.

Expected values are EC-owned literals from #3087: R1 refuses CRDs that are not
established or additive; R2/R6 refuse zero or multiple reconciliation owners;
R5 refuses rollback past finalization or into N-only behavior and identifies
the blocking instance. The neighboring admissible rollback proves these are
guards, not a design that refuses every request.
"""

from __future__ import annotations

from lumen.operator_upgrade.admission import decide_operator_admission
from lumen.operator_upgrade.handover import decide_owner
from lumen.operator_upgrade.rollback import decide_rollback
from lumen.operator_upgrade.spec import (
    CrdAdmissionRequest,
    HandoverState,
    ManagedInstance,
    OperatorReplica,
    RollbackRequest,
)
from lumen.operator_upgrade.verdict import Refused

MINIMUM_CHECKS = 10

OPERATOR_UPGRADE_3087_SECURITY_MATRIX = (
    ("unestablished_crd_is_refused_with_reason", "crd_not_established"),
    ("unestablished_crd_refusal_names_established_field", "established"),
    ("non_additive_crd_is_refused_with_reason", "non_additive_crd"),
    ("non_additive_crd_refusal_names_additive_field", "additive_to_active"),
    ("zero_lease_owners_are_refused", "no_reconciliation_owner"),
    ("multiple_lease_owners_are_refused", "multiple_reconciliation_owners"),
    ("finalized_instance_blocks_rollback", "rollback_not_pre_finalize"),
    ("n_only_behavior_blocks_rollback", "n_only_behavior_required"),
    ("rollback_refusal_identifies_the_blocking_instance", "orders-1"),
    ("neighbouring_pre_finalize_rollback_is_admitted", "admitted"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Refused) else "admitted"


def verify_operator_upgrade_3087_security() -> dict:
    checks = []

    # 1. R1 -- a candidate CRD cannot authorize N before the API server has
    #    established it; naming ``established`` makes the refusal actionable.
    unestablished = decide_operator_admission(
        CrdAdmissionRequest(established=False, additive_to_active=True)
    )
    obs1 = _reason(unestablished)
    exp1 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[0][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- inspect the distinct refusal location, not a design-supplied
    #    validity flag or the same reason value a second time.
    obs2 = unestablished.field_path if isinstance(unestablished, Refused) else "admitted"
    exp2 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[1][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- establishment alone is not enough for a breaking CRD change.
    non_additive = decide_operator_admission(
        CrdAdmissionRequest(established=True, additive_to_active=False)
    )
    obs3 = _reason(non_additive)
    exp3 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[2][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- the second forbidden argument is named independently.
    obs4 = non_additive.field_path if isinstance(non_additive, Refused) else "admitted"
    exp4 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[3][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2/R6 -- an explicit two-replica state with neither replica holding
    #    the Lease must not invent a reconciliation writer.
    no_owner = decide_owner(
        HandoverState(
            replicas=(
                OperatorReplica(version="N-1", available=True, holds_lease=False),
                OperatorReplica(version="N", available=True, holds_lease=False),
            )
        )
    )
    obs5 = _reason(no_owner)
    exp5 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[4][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2/R6 -- the equally explicit split-Lease model is refused rather
    #    than choosing a winner and concealing concurrent writers.
    multiple_owners = decide_owner(
        HandoverState(
            replicas=(
                OperatorReplica(version="N-1", available=True, holds_lease=True),
                OperatorReplica(version="N", available=True, holds_lease=True),
            )
        )
    )
    obs6 = _reason(multiple_owners)
    exp6 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[5][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R5 -- finalized state is irreversible from N-1's perspective.
    finalized = decide_rollback(
        RollbackRequest(
            instances=(ManagedInstance(instance_id="orders-0", pre_finalize=False, requires_n_only_behavior=False),)
        )
    )
    obs7 = _reason(finalized)
    exp7 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[6][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- pre-finalize alone is insufficient when the instance needs
    #    behavior N-1 cannot provide.
    n_only = decide_rollback(
        RollbackRequest(
            instances=(ManagedInstance(instance_id="orders-1", pre_finalize=True, requires_n_only_behavior=True),)
        )
    )
    obs8 = _reason(n_only)
    exp8 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[7][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R5 -- the structured refusal identifies the precise instance that
    #    prevented rollback; a generic error would leave an operator guessing.
    obs9 = n_only.blocking_instance_id if isinstance(n_only, Refused) else "admitted"
    exp9 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[8][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5 -- a neighboring compatible input remains admitted, proving that
    #     the preceding refusals are specific guards rather than blanket denial.
    compatible = decide_rollback(
        RollbackRequest(
            instances=(ManagedInstance(instance_id="orders-1", pre_finalize=True, requires_n_only_behavior=False),)
        )
    )
    obs10 = _reason(compatible)
    exp10 = OPERATOR_UPGRADE_3087_SECURITY_MATRIX[9][1]
    checks.append({"name": OPERATOR_UPGRADE_3087_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "operator-upgrade-3087-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
