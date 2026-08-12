"""EC behavior case for #3146 -- one shared Lumen lifecycle.

Every expected value below is an EC-owned literal transcribed from #3146:
R1 carries one generation and reason through an idempotent transition; R2/R3
separate health, readiness, recovery, dependency loss, and supported
read-only degradation; R4 closes public admission and sends h2c GOAWAY; R5
publishes the ordered domain-hook plan; R6 preserves the rendered total,
runtime, and reserve budget; and R7 projects the stable terminal evidence.
Runtime I/O, signals, request completion, and Kubernetes timing are excluded.
"""

from __future__ import annotations

from lumen.lifecycle.admission import decide_lifecycle_transition
from lumen.lifecycle.budget import decide_lifecycle_budget
from lumen.lifecycle.gates import decide_probe_state
from lumen.lifecycle.hooks import ordered_hook_plan
from lumen.lifecycle.spec import HookOutcome, LifecycleFacts, LifecyclePolicy, LifecycleRequest, LifecycleState
from lumen.lifecycle.status import project_lifecycle_status
from lumen.lifecycle.verdict import Rejection

MINIMUM_CHECKS = 28

LIFECYCLE_3146_BEHAVIOR_MATRIX = (
    ("authorized_quiesce_enters_draining", "Draining"),
    ("authorized_quiesce_advances_generation_once", 8),
    ("authorized_quiesce_records_reason", "authorized_quiesce"),
    ("draining_transition_makes_readiness_not_ready", "not_ready"),
    ("draining_transition_closes_public_admission", "closed"),
    ("draining_transition_declares_h2c_goaway", "h2c_goaway"),
    ("repeated_quiesce_keeps_existing_generation", 8),
    ("recovery_is_health_positive", "healthy"),
    ("recovery_is_readiness_negative", "not_ready"),
    ("temporary_global_dependency_loss_remains_healthy", "healthy"),
    ("correct_read_only_degradation_is_admitted", "read_only_degraded"),
    ("read_only_degradation_declares_507_mutations", "507_insufficient_storage"),
    ("draining_hook_plan_has_required_order", ("close_write_proposal_admission", "preserve_admitted_mutation_outcomes", "quiesce_transfer_drain_raft_host", "stop_shard_catalog_capacity_snapshot_background_tasks", "close_peer_listener_after_raft_safe_close", "sync_supported_legacy_durable_writer", "flush_tracing_and_metrics")),
    ("policy_total_is_preserved", 30),
    ("policy_runtime_is_preserved", 25),
    ("policy_sigkill_reserve_is_preserved", 5),
    ("status_projects_stable_phase", "Drained"),
    ("status_projects_stable_generation", 8),
    ("status_projects_terminal_drained_report", "Drained=True"),
    ("incompatible_configuration_blocks_readiness", "not_ready"),
    ("incompatible_format_blocks_readiness", "not_ready"),
    ("missing_auth_material_blocks_readiness", "not_ready"),
    ("missing_tls_material_blocks_readiness", "not_ready"),
    ("uninitialized_catalog_routing_blocks_readiness", "not_ready"),
    ("unadmitted_raft_member_blocks_readiness", "not_ready"),
    ("fatal_local_failure_is_unhealthy", "unhealthy"),
    ("status_projects_stable_reason", "authorized_quiesce"),
    ("status_projects_per_hook_outcomes", (("close_write_proposal_admission", "finished"), ("flush_tracing_and_metrics", "finished"))),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _ready_facts(**changes) -> LifecycleFacts:
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
        "read_only_degraded": False,
        "reads_correct": True,
        "mutation_outcome": "accepted",
    }
    values.update(changes)
    return LifecycleFacts(**values)


def verify_lifecycle_3146_behavior() -> dict:
    checks = []
    serving = LifecycleState(phase="Serving", generation=7, reason="startup_complete", readiness="ready")
    quiesce = LifecycleRequest(trigger="quiesce", authorized=True, reason="authorized_quiesce")
    draining = decide_lifecycle_transition(quiesce, serving)

    # 1-6. R1/R4 -- one authorized trigger creates the shared draining state.
    obs1 = draining.phase if not isinstance(draining, Rejection) else "rejected"
    exp1 = LIFECYCLE_3146_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = draining.generation if not isinstance(draining, Rejection) else -1
    exp2 = LIFECYCLE_3146_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = draining.reason if not isinstance(draining, Rejection) else "rejected"
    exp3 = LIFECYCLE_3146_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = draining.readiness if not isinstance(draining, Rejection) else "rejected"
    exp4 = LIFECYCLE_3146_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = draining.public_admission if not isinstance(draining, Rejection) else "rejected"
    exp5 = LIFECYCLE_3146_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = draining.protocol_action if not isinstance(draining, Rejection) else "rejected"
    exp6 = LIFECYCLE_3146_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R1/R4 -- repeated triggers reuse the one lifecycle generation.
    repeated_current = LifecycleState(
        phase=draining.phase if not isinstance(draining, Rejection) else "Serving",
        generation=draining.generation if not isinstance(draining, Rejection) else 7,
        reason=draining.reason if not isinstance(draining, Rejection) else "startup_complete",
        readiness=draining.readiness if not isinstance(draining, Rejection) else "ready",
    )
    repeated = decide_lifecycle_transition(quiesce, repeated_current)
    obs7 = repeated.generation if not isinstance(repeated, Rejection) else -1
    exp7 = LIFECYCLE_3146_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    recovery = decide_probe_state(_ready_facts(storage_restored=False))
    # 8-9. R2 -- recovery is observable as healthy but cannot yet become ready.
    obs8 = recovery.health
    exp8 = LIFECYCLE_3146_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = recovery.readiness
    exp9 = LIFECYCLE_3146_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 20-25. R2 -- every other local startup/admission fact independently
    # blocks readiness while the process can remain health-positive in recovery.
    incompatible_configuration = decide_probe_state(_ready_facts(configuration_compatible=False))
    obs20 = incompatible_configuration.readiness
    exp20 = LIFECYCLE_3146_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    incompatible_format = decide_probe_state(_ready_facts(formats_compatible=False))
    obs21 = incompatible_format.readiness
    exp21 = LIFECYCLE_3146_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    missing_auth = decide_probe_state(_ready_facts(auth_material_ready=False))
    obs22 = missing_auth.readiness
    exp22 = LIFECYCLE_3146_BEHAVIOR_MATRIX[21][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    missing_tls = decide_probe_state(_ready_facts(tls_material_ready=False))
    obs23 = missing_tls.readiness
    exp23 = LIFECYCLE_3146_BEHAVIOR_MATRIX[22][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    uninitialized_catalog_routing = decide_probe_state(_ready_facts(catalog_routing_initialized=False))
    obs24 = uninitialized_catalog_routing.readiness
    exp24 = LIFECYCLE_3146_BEHAVIOR_MATRIX[23][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    unadmitted_raft_member = decide_probe_state(_ready_facts(raft_member_admitted=False))
    obs25 = unadmitted_raft_member.readiness
    exp25 = LIFECYCLE_3146_BEHAVIOR_MATRIX[24][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})

    dependency_loss = decide_probe_state(_ready_facts(leader_available=False, quorum_available=False, cloud_available=False))
    # 10. R3 -- temporary global dependency loss is not a fatal local failure.
    obs10 = dependency_loss.health
    exp10 = LIFECYCLE_3146_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    read_only = decide_probe_state(_ready_facts(read_only_degraded=True, mutation_outcome="507_insufficient_storage"))
    # 11-12. R3 -- supported degradation is explicit about readiness and writes.
    obs11 = read_only.readiness
    exp11 = LIFECYCLE_3146_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = read_only.mutation_outcome
    exp12 = LIFECYCLE_3146_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    fatal_local_failure = decide_probe_state(_ready_facts(fatal_local_failure=True))
    # 26. R3 -- a fatal local invariant or supervisor failure is unhealthy.
    obs26 = fatal_local_failure.health
    exp26 = LIFECYCLE_3146_BEHAVIOR_MATRIX[25][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    # 13. R5 -- domain effects stay in the one immutable order used by draining.
    plan = ordered_hook_plan("Draining")
    obs13 = plan.steps if not isinstance(plan, Rejection) else ("rejected",)
    exp13 = LIFECYCLE_3146_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    budget = decide_lifecycle_budget(LifecyclePolicy(total_seconds=30, runtime_seconds=25, reserve_seconds=5))
    # 14-16. R6 -- service-k8s' three rendered values remain distinct.
    obs14 = budget.total_seconds if not isinstance(budget, Rejection) else -1
    exp14 = LIFECYCLE_3146_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = budget.runtime_seconds if not isinstance(budget, Rejection) else -1
    exp15 = LIFECYCLE_3146_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = budget.reserve_seconds if not isinstance(budget, Rejection) else -1
    exp16 = LIFECYCLE_3146_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    status = project_lifecycle_status(
        LifecycleState(phase="Drained", generation=8, reason="authorized_quiesce", readiness="not_ready"),
        (
            HookOutcome(name="close_write_proposal_admission", outcome="finished"),
            HookOutcome(name="flush_tracing_and_metrics", outcome="finished"),
        ),
    )
    # 17-19. R7 -- the finalizer receives stable terminal evidence, not a flag.
    obs17 = status.phase
    exp17 = LIFECYCLE_3146_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = status.generation
    exp18 = LIFECYCLE_3146_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    obs19 = status.terminal_condition
    exp19 = LIFECYCLE_3146_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 27-28. R7 -- the terminal projection preserves its cause and each
    # distinguishable hook outcome for operator and finalizer consumers.
    obs27 = status.reason
    exp27 = LIFECYCLE_3146_BEHAVIOR_MATRIX[26][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})
    obs28 = tuple((outcome.name, outcome.outcome) for outcome in status.hook_outcomes)
    exp28 = LIFECYCLE_3146_BEHAVIOR_MATRIX[27][1]
    checks.append({"name": LIFECYCLE_3146_BEHAVIOR_MATRIX[27][0], "expected": exp28, "observed": obs28, "passed": obs28 == exp28})

    return {"case_id": "lifecycle-3146-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
