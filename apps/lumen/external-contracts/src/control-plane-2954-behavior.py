"""EC behavior case for #2954 -- shared-transition Lumen projection.

Every expected value is an EC-owned literal transcribed from #2954 R1--R6
and AC1/AC3/AC4.  The case drives only the pure design boundary: it does not
claim to observe a Deployment, a Lease, SSA, or a Kubernetes event.
"""

from __future__ import annotations

from lumen.control_plane.adapter import adapt_shared_transition, adapter_contract
from lumen.control_plane.intent import PersistedIntent, resume_intent
from lumen.control_plane.ownership import CapacityInput, InstallationInput, decide_ownership
from lumen.control_plane.status import project_observability
from lumen.control_plane.verdict import SharedTransition

MINIMUM_CHECKS = 27

CONTROL_PLANE_2954_BEHAVIOR_MATRIX = (
    ("shared_projection_preserves_generation", 17),
    ("shared_projection_preserves_current_machine", "n2-standard-4"),
    ("shared_projection_preserves_target_machine", "c3-standard-4"),
    ("progressing_shared_transition_has_lumen_phase", "progressing"),
    ("progressing_shared_transition_has_lumen_condition", "Progressing"),
    ("progressing_shared_transition_has_metric_phase_label", "progressing"),
    ("projection_names_shared_actuator_as_delegate", "shared-actuator"),
    ("adapter_contract_accepts_shared_transition_input", "shared_transition"),
    ("adapter_contract_exposes_lumen_projection_states", ("preflight", "progressing", "stalled", "hold", "converged")),
    ("ownership_admits_installation_image", "ghcr.io/axiom/lumen:v2"),
    ("ownership_admits_installation_identity", "lumen-operator"),
    ("ownership_admits_capacity_current_machine", "n2-standard-4"),
    ("ownership_admits_capacity_resources", "cpu=4,memory=16Gi"),
    ("equal_reapply_retains_one_intent_identity", "intent-17"),
    ("equal_reapply_retains_target_machine", "c3-standard-4"),
    ("equal_reapply_has_retained_disposition", "retained"),
    ("observability_records_initial_machine", "e2-standard-4"),
    ("converged_observability_has_converged_alert", "MachineTransitionConverged"),
    ("preflight_observability_has_preflight_phase", "preflight"),
    ("progressing_observability_has_progressing_condition", "Progressing"),
    ("stalled_observability_has_stalled_event", "MachineTransitionStalled"),
    ("hold_observability_has_hold_alert", "MachineTransitionHeld"),
    ("converged_observability_has_converged_phase", "converged"),
    ("observability_records_current_machine", "n2-standard-4"),
    ("observability_records_target_machine", "c3-standard-4"),
    ("observability_records_generation", 17),
    ("stalled_shared_transition_preserves_error_vocabulary", "UNSCHEDULABLE"),
)


def verify_control_plane_2954_behavior() -> dict:
    checks = []
    intent = PersistedIntent(
        identity="intent-17",
        generation=17,
        initial_machine="e2-standard-4",
        current_machine="n2-standard-4",
        target_machine="c3-standard-4",
    )
    progressing = SharedTransition(phase="progressing", error_code=None)
    projection = adapt_shared_transition(intent, progressing)

    # 1-7. R1/R3 -- the adapter projects the supplied shared transition; it
    # never chooses placement or performs a local rollout decision.
    obs1 = projection.generation; exp1 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = projection.current_machine; exp2 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = projection.target_machine; exp3 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = projection.phase; exp4 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = projection.condition; exp5 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = projection.metric_labels["phase"]; exp6 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = projection.delegate; exp7 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    contract = adapter_contract()
    # 8-9. AC1 -- this is an adapter boundary, not a second Deployment state machine.
    obs8 = contract.input_kind; exp8 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = contract.projection_states; exp9 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    ownership = decide_ownership(
        InstallationInput(image="ghcr.io/axiom/lumen:v2", identity="lumen-operator", fixed_replicas=2, policy="RollingUpdate"),
        CapacityInput(current_machine="n2-standard-4", target_machine="c3-standard-4", resources="cpu=4,memory=16Gi"),
    )
    # 10-13. R2 -- installation and capacity values arrive in separate admitted partitions.
    obs10 = ownership.installation.image; exp10 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = ownership.installation.identity; exp11 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = ownership.capacity.current_machine; exp12 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = ownership.capacity.resources; exp13 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    resumed = resume_intent(intent, InstallationInput(image="ghcr.io/axiom/lumen:v2", identity="lumen-operator", fixed_replicas=2, policy="RollingUpdate"))
    # 14-16. R5/AC3 -- an equal original-install or GitOps reapply retains,
    # rather than replaces or duplicates, the persisted intent.
    obs14 = resumed.intent.identity; exp14 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = resumed.intent.target_machine; exp15 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = resumed.disposition; exp16 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    converged = project_observability(intent, SharedTransition(phase="converged", error_code=None))
    # 17-18. R6/AC4 -- the record keeps the transition's complete machine
    # history and names its terminal alert rather than only reporting success.
    obs17 = converged.initial_machine; exp17 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = converged.alert_kind; exp18 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    preflight = project_observability(intent, SharedTransition(phase="preflight", error_code=None))
    progressing_record = project_observability(intent, SharedTransition(phase="progressing", error_code=None))
    stalled = project_observability(intent, SharedTransition(phase="stalled", error_code="UNSCHEDULABLE"))
    held = project_observability(intent, SharedTransition(phase="hold", error_code="TIMEOUT"))
    # 19-26. R6/AC4 -- this status entry point independently projects every
    # shared outcome and retains all machine-generation coordinates in its
    # observable record.  The values are not inferred from adapter rows above.
    obs19 = preflight.phase; exp19 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = progressing_record.condition; exp20 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    obs21 = stalled.event_kind; exp21 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = held.alert_kind; exp22 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[21][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    obs23 = converged.phase; exp23 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[22][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    obs24 = converged.current_machine; exp24 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[23][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    obs25 = converged.target_machine; exp25 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[24][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    obs26 = converged.generation; exp26 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[25][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    stalled_projection = adapt_shared_transition(intent, SharedTransition(phase="stalled", error_code="UNSCHEDULABLE"))
    # 27. R1 -- the Lumen projection carries the shared transition's actionable
    # error vocabulary; it does not collapse the failure into a success flag.
    obs27 = stalled_projection.error_code; exp27 = CONTROL_PLANE_2954_BEHAVIOR_MATRIX[26][1]
    checks.append({"name": CONTROL_PLANE_2954_BEHAVIOR_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})

    return {"case_id": "control-plane-2954-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
