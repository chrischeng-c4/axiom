"""EC security case for #2955 -- fail-closed control-plane recovery.

Expected values are EC-owned literals from #2955. R2 refuses a requested
mutation when no operator replica is available; R3 refuses ambiguous recovery
that could resume more than one intent; R4 refuses rendered ownership
overwrites and active-transition conflicts; and R5 refuses a target above the
supplied available capacity.
"""

from __future__ import annotations

from lumen.control_plane.capacity import decide_target_capacity
from lumen.control_plane.ownership import decide_reapply
from lumen.control_plane.recovery import decide_outage_state, decide_recovery
from lumen.control_plane.spec import PersistedIntent, RecoveryState, TransitionState
from lumen.control_plane.verdict import Rejection

MINIMUM_CHECKS = 15

CONTROL_PLANE_2955_SECURITY_MATRIX = (
    ("outage_mutation_request_is_refused", "control_plane_unavailable"),
    ("outage_refusal_names_mutation_request", "mutation_requested"),
    ("outage_without_mutation_remains_frozen_serving", "frozen-serving"),
    ("multiple_recovery_intents_are_refused", "multiple_resumable_intents"),
    ("multiple_recovery_intents_refusal_names_persisted_intents", "persisted_intents"),
    ("empty_recovery_is_a_noop", "no-op"),
    ("rendered_current_capacity_overwrite_is_refused", "controller_owned_field_overwrite"),
    ("current_capacity_overwrite_names_rendered_field", "rendered_fields.current_capacity"),
    ("rendered_target_capacity_overwrite_is_refused", "controller_owned_field_overwrite"),
    ("target_capacity_overwrite_names_rendered_field", "rendered_fields.target_capacity"),
    ("active_transition_conflict_is_refused", "conflicting_transition"),
    ("transition_conflict_names_transition_state", "transition_state"),
    ("target_above_available_capacity_is_refused", "insufficient_available_capacity"),
    ("capacity_refusal_names_target_capacity", "target_capacity"),
    ("available_capacity_neighbour_is_admitted", "admitted"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _field(verdict) -> str:
    return verdict.field_path if isinstance(verdict, Rejection) else ""


def verify_control_plane_2955_security() -> dict:
    checks = []

    outage_refusal = decide_outage_state(False, 17, 23, True)

    # 1-3. R2 -- an explicit mutation request is rejected by the outage entry
    # point, names the request, while the adjacent no-request state still serves.
    obs1 = _reason(outage_refusal)
    exp1 = CONTROL_PLANE_2955_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = _field(outage_refusal)
    exp2 = CONTROL_PLANE_2955_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    outage_idle = decide_outage_state(False, 17, 23, False)
    obs3 = outage_idle.kind if not isinstance(outage_idle, Rejection) else outage_idle.reason.value
    exp3 = CONTROL_PLANE_2955_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    multiple = decide_recovery(RecoveryState(
        control_plane_available=True,
        persisted_intents=(
            PersistedIntent(intent_id="reshard-orders-7", phase="catching-up"),
            PersistedIntent(intent_id="grow-catalog-3", phase="preflight-pending"),
        ),
    ))

    # 4-6. R3 -- recovery cannot choose amongst multiple persistent intents;
    # it names the collection, while an explicitly empty state is a no-op.
    obs4 = _reason(multiple)
    exp4 = CONTROL_PLANE_2955_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = _field(multiple)
    exp5 = CONTROL_PLANE_2955_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    empty = decide_recovery(RecoveryState(control_plane_available=True, persisted_intents=()))
    obs6 = empty.action if not isinstance(empty, Rejection) else empty.reason.value
    exp6 = CONTROL_PLANE_2955_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    owned = {"current_capacity": "n2-standard-4", "target_capacity": "c3-standard-4"}
    idle = TransitionState(phase="idle", transition_id=None)
    current_overwrite = decide_reapply({"current_capacity": "c3-standard-4"}, owned, idle)

    # 7-10. R4 -- each controller-owned capacity field is independently
    # protected at reapply, so a renderer cannot reset just one of them.
    obs7 = _reason(current_overwrite)
    exp7 = CONTROL_PLANE_2955_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = _field(current_overwrite)
    exp8 = CONTROL_PLANE_2955_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    target_overwrite = decide_reapply({"target_capacity": "n2-standard-8"}, owned, idle)
    obs9 = _reason(target_overwrite)
    exp9 = CONTROL_PLANE_2955_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = _field(target_overwrite)
    exp10 = CONTROL_PLANE_2955_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    conflict = decide_reapply(
        {"installation_owner": "platform-team", "transition_id": "other-transition"},
        owned,
        TransitionState(phase="machine-moving", transition_id="capacity-11"),
    )

    # 11-12. R4 -- an active controller transition also rejects a conflicting
    # renderer transition and points to the state that owns the conflict.
    obs11 = _reason(conflict)
    exp11 = CONTROL_PLANE_2955_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = _field(conflict)
    exp12 = CONTROL_PLANE_2955_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    insufficient = decide_target_capacity(3, 6, 5)

    # 13-15. R5 -- preflight refuses a target beyond available capacity, names
    # that target, and admits the adjacent target exactly at availability.
    obs13 = _reason(insufficient)
    exp13 = CONTROL_PLANE_2955_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = _field(insufficient)
    exp14 = CONTROL_PLANE_2955_SECURITY_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    available = decide_target_capacity(3, 5, 5)
    obs15 = _reason(available)
    exp15 = CONTROL_PLANE_2955_SECURITY_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2955_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {"case_id": "control-plane-2955-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
