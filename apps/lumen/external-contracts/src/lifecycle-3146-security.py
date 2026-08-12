"""EC security case for #3146 -- fail-closed shared lifecycle decisions.

Expected literals are transcribed from #3146: R3 permits read-only degradation
only with correct reads and declared 507 mutations; R4/R7 accept only an
authorized quiesce trigger; R5 refuses hook execution outside Draining; R6
refuses budgets with no SIGKILL reserve; and R7 never reports Drained=True
while any hook remains unfinished. Runtime-only signal, I/O, and process claims
are intentionally absent.
"""

from __future__ import annotations

from lumen.lifecycle.admission import decide_lifecycle_transition
from lumen.lifecycle.budget import decide_lifecycle_budget
from lumen.lifecycle.gates import decide_probe_state
from lumen.lifecycle.hooks import ordered_hook_plan
from lumen.lifecycle.spec import HookOutcome, LifecycleFacts, LifecyclePolicy, LifecycleRequest, LifecycleState
from lumen.lifecycle.status import project_lifecycle_status
from lumen.lifecycle.verdict import Rejection

MINIMUM_CHECKS = 15

LIFECYCLE_3146_SECURITY_MATRIX = (
    ("unauthorized_quiesce_is_refused", "unauthorized_quiesce"),
    ("unauthorized_quiesce_refusal_names_authorization", "authorized"),
    ("authorized_quiesce_neighbour_is_admitted", "admitted"),
    ("unsupported_lifecycle_trigger_is_refused", "unsupported_trigger"),
    ("unsupported_trigger_refusal_names_trigger", "trigger"),
    ("read_only_degradation_with_incorrect_reads_is_unhealthy", "unhealthy"),
    ("incorrect_read_degradation_is_not_admitted", "not_ready"),
    ("read_only_degradation_with_wrong_mutation_outcome_is_unhealthy", "unhealthy"),
    ("non_draining_hook_plan_is_refused", "hooks_require_draining"),
    ("non_draining_hook_refusal_names_phase", "phase"),
    ("draining_hook_plan_neighbour_is_admitted", "admitted"),
    ("zero_sigkill_reserve_is_refused", "sigkill_reserve_required"),
    ("budget_refusal_names_reserve", "reserve_seconds"),
    ("reserved_budget_neighbour_is_admitted", "admitted"),
    ("unfinished_hook_never_reports_drained", "Drained=False"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _facts(**changes) -> LifecycleFacts:
    values = {
        "configuration_compatible": True,
        "formats_compatible": True,
        "storage_restored": True,
        "auth_material_ready": True,
        "tls_material_ready": True,
        "catalog_routing_initialized": True,
        "raft_member_admitted": True,
        "local_forward_progress": True,
        "leader_available": True,
        "quorum_available": True,
        "cloud_available": True,
        "fatal_local_failure": False,
        "read_only_degraded": True,
        "reads_correct": True,
        "mutation_outcome": "507_insufficient_storage",
    }
    values.update(changes)
    return LifecycleFacts(**values)


def verify_lifecycle_3146_security() -> dict:
    checks = []
    serving = LifecycleState(phase="Serving", generation=7, reason="startup_complete", readiness="ready")

    unauthorized = decide_lifecycle_transition(
        LifecycleRequest(trigger="quiesce", authorized=False, reason="operator_request"), serving
    )
    # 1-3. R4/R7 -- authorization is checked at transition admission, and the
    # nearest explicitly authorized request remains usable.
    obs1 = _outcome(unauthorized)
    exp1 = LIFECYCLE_3146_SECURITY_MATRIX[0][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = unauthorized.field_path if isinstance(unauthorized, Rejection) else ""
    exp2 = LIFECYCLE_3146_SECURITY_MATRIX[1][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    authorized = decide_lifecycle_transition(
        LifecycleRequest(trigger="quiesce", authorized=True, reason="operator_request"), serving
    )
    obs3 = _outcome(authorized)
    exp3 = LIFECYCLE_3146_SECURITY_MATRIX[2][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    unknown = decide_lifecycle_transition(
        LifecycleRequest(trigger="maintenance_pause", authorized=True, reason="operator_request"), serving
    )
    # 4-5. R4 -- authorization cannot turn an unrecognised trigger into drain.
    obs4 = _outcome(unknown)
    exp4 = LIFECYCLE_3146_SECURITY_MATRIX[3][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = unknown.field_path if isinstance(unknown, Rejection) else ""
    exp5 = LIFECYCLE_3146_SECURITY_MATRIX[4][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    bad_reads = decide_probe_state(_facts(reads_correct=False))
    # 6-7. R3 -- degraded readiness is never an excuse to claim incorrect reads.
    obs6 = bad_reads.health
    exp6 = LIFECYCLE_3146_SECURITY_MATRIX[5][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = bad_reads.readiness
    exp7 = LIFECYCLE_3146_SECURITY_MATRIX[6][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    bad_writes = decide_probe_state(_facts(mutation_outcome="accepted"))
    # 8. R3 -- declared 507 behavior is part of the degradation admission rule.
    obs8 = bad_writes.health
    exp8 = LIFECYCLE_3146_SECURITY_MATRIX[7][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    wrong_phase = ordered_hook_plan("Serving")
    # 9-11. R5 -- hooks cannot be claimed outside drain, while Draining admits
    # the same entry point.
    obs9 = _outcome(wrong_phase)
    exp9 = LIFECYCLE_3146_SECURITY_MATRIX[8][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = wrong_phase.field_path if isinstance(wrong_phase, Rejection) else ""
    exp10 = LIFECYCLE_3146_SECURITY_MATRIX[9][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    draining_plan = ordered_hook_plan("Draining")
    obs11 = _outcome(draining_plan)
    exp11 = LIFECYCLE_3146_SECURITY_MATRIX[10][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    no_reserve = decide_lifecycle_budget(LifecyclePolicy(total_seconds=30, runtime_seconds=30, reserve_seconds=0))
    # 12-14. R6 -- a policy must reserve SIGKILL time and identify its bad value.
    obs12 = _outcome(no_reserve)
    exp12 = LIFECYCLE_3146_SECURITY_MATRIX[11][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = no_reserve.field_path if isinstance(no_reserve, Rejection) else ""
    exp13 = LIFECYCLE_3146_SECURITY_MATRIX[12][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    reserved = decide_lifecycle_budget(LifecyclePolicy(total_seconds=30, runtime_seconds=25, reserve_seconds=5))
    obs14 = _outcome(reserved)
    exp14 = LIFECYCLE_3146_SECURITY_MATRIX[13][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    partial = project_lifecycle_status(
        LifecycleState(phase="Draining", generation=8, reason="authorized_quiesce", readiness="not_ready"),
        (HookOutcome(name="flush_tracing_and_metrics", outcome="timeout"),),
    )
    # 15. R7 -- terminal evidence cannot overclaim success while a hook timed out.
    obs15 = partial.terminal_condition
    exp15 = LIFECYCLE_3146_SECURITY_MATRIX[14][1]
    checks.append({"name": LIFECYCLE_3146_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {"case_id": "lifecycle-3146-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
