"""EC security case for #2954 -- fail-closed control-plane projection.

Expected literals are EC-owned transcriptions of #2954 R1--R6 and AC1/AC3/AC4.
The case deliberately exercises explicit wrong-owner, local-rollout, unknown-
transition, and hold inputs.  It observes refusal values and named fields, not
design-computed validity flags or runtime Kubernetes behavior.
"""

from __future__ import annotations

from lumen.control_plane.adapter import adapt_shared_transition, adapter_contract
from lumen.control_plane.intent import PersistedIntent, resume_intent
from lumen.control_plane.ownership import CapacityInput, InstallationInput, decide_ownership
from lumen.control_plane.status import project_observability, project_outcome
from lumen.control_plane.verdict import SharedTransition

MINIMUM_CHECKS = 19

CONTROL_PLANE_2954_SECURITY_MATRIX = (
    ("capacity_owned_image_is_refused", "wrong_owner"),
    ("capacity_owned_image_refusal_names_image", "capacity.image"),
    ("neighbouring_owner_partition_is_admitted", "admitted"),
    ("installation_owned_target_is_refused", "wrong_owner"),
    ("installation_owned_target_refusal_names_target", "installation.target_machine"),
    ("local_placement_decision_is_refused_by_adapter", "local_rollout_decision_forbidden"),
    ("local_placement_refusal_names_decision", "shared_transition.local_placement_decision"),
    ("unknown_shared_phase_is_refused", "unknown_shared_phase"),
    ("unknown_phase_refusal_names_phase", "shared_transition.phase"),
    ("neighbouring_progressing_phase_is_projected", "progressing"),
    ("adapter_contract_excludes_local_placement_vocabulary", ("preflight", "progressing", "stalled", "hold", "converged")),
    ("unschedulable_projects_stalled_condition", "Stalled"),
    ("unschedulable_holds_prior_current_machine", "n2-standard-4"),
    ("timeout_projects_hold_condition", "Hold"),
    ("timeout_holds_prior_current_machine", "n2-standard-4"),
    ("equal_reapply_never_replaces_identity", "intent-17"),
    ("equal_reapply_never_resets_generation", 17),
    ("equal_reapply_never_duplicates_intent", "retained"),
    ("hold_observability_has_rollback_hold_event", "MachineTransitionHeld"),
)


def verify_control_plane_2954_security() -> dict:
    checks = []
    intent = PersistedIntent(identity="intent-17", generation=17, initial_machine="e2-standard-4", current_machine="n2-standard-4", target_machine="c3-standard-4")
    installation = InstallationInput(image="ghcr.io/axiom/lumen:v2", identity="lumen-operator", fixed_replicas=2, policy="RollingUpdate")
    capacity = CapacityInput(current_machine="n2-standard-4", target_machine="c3-standard-4", resources="cpu=4,memory=16Gi")

    wrong_image = decide_ownership(installation, CapacityInput(current_machine="n2-standard-4", target_machine="c3-standard-4", resources="cpu=4,memory=16Gi", image="ghcr.io/axiom/lumen:wrong-owner"))
    # 1-3. R2 -- a capacity request cannot smuggle installation image policy;
    # the neighbouring explicit, correctly partitioned request remains admitted.
    obs1 = wrong_image.reason; exp1 = CONTROL_PLANE_2954_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = wrong_image.field_path; exp2 = CONTROL_PLANE_2954_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    admitted = decide_ownership(installation, capacity)
    obs3 = admitted.disposition; exp3 = CONTROL_PLANE_2954_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    wrong_target = decide_ownership(InstallationInput(image="ghcr.io/axiom/lumen:v2", identity="lumen-operator", fixed_replicas=2, policy="RollingUpdate", target_machine="c3-standard-4"), capacity)
    # 4-5. R2 -- installation likewise cannot claim capacity-owned placement.
    obs4 = wrong_target.reason; exp4 = CONTROL_PLANE_2954_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = wrong_target.field_path; exp5 = CONTROL_PLANE_2954_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    local = adapt_shared_transition(intent, SharedTransition(phase="progressing", error_code=None, local_placement_decision="place-target"))
    unknown = adapt_shared_transition(intent, SharedTransition(phase="deploying", error_code=None))
    progressing = adapt_shared_transition(intent, SharedTransition(phase="progressing", error_code=None))
    # 6-10. R3/AC1 -- the adapter refuses both a local rollout decision and an
    # invented shared phase, while a supplied shared progressing phase projects.
    obs6 = local.reason; exp6 = CONTROL_PLANE_2954_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = local.field_path; exp7 = CONTROL_PLANE_2954_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = unknown.reason; exp8 = CONTROL_PLANE_2954_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = unknown.field_path; exp9 = CONTROL_PLANE_2954_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = progressing.phase; exp10 = CONTROL_PLANE_2954_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. AC1 -- inspect the declared vocabulary itself, not a design-provided
    # "delegated" boolean that could overclaim the absence of a local engine.
    obs11 = adapter_contract().projection_states; exp11 = CONTROL_PLANE_2954_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    unschedulable = project_outcome(intent, SharedTransition(phase="stalled", error_code="UNSCHEDULABLE"))
    timed_out = project_outcome(intent, SharedTransition(phase="hold", error_code="TIMEOUT"))
    # 12-15. R4 -- failure reports an actionable condition but never promotes
    # the target over the last healthy, authoritative current placement.
    obs12 = unschedulable.condition; exp12 = CONTROL_PLANE_2954_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = unschedulable.authoritative_machine; exp13 = CONTROL_PLANE_2954_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = timed_out.condition; exp14 = CONTROL_PLANE_2954_SECURITY_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = timed_out.authoritative_machine; exp15 = CONTROL_PLANE_2954_SECURITY_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    resumed = resume_intent(intent, installation)
    # 16-18. R5/AC3 -- an explicit equal reapply holds identity, generation,
    # and disposition independently, leaving no default-value escape hatch.
    obs16 = resumed.intent.identity; exp16 = CONTROL_PLANE_2954_SECURITY_MATRIX[15][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = resumed.intent.generation; exp17 = CONTROL_PLANE_2954_SECURITY_MATRIX[16][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = resumed.disposition; exp18 = CONTROL_PLANE_2954_SECURITY_MATRIX[17][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    held = project_observability(intent, SharedTransition(phase="hold", error_code="TIMEOUT"))
    # 19. R6/AC4 -- hold must be externally distinguishable, not merely a
    # generic failure metric or a boolean unsuccessful result.
    obs19 = held.event_kind; exp19 = CONTROL_PLANE_2954_SECURITY_MATRIX[18][1]
    checks.append({"name": CONTROL_PLANE_2954_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    return {"case_id": "control-plane-2954-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
