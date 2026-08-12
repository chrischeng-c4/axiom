"""EC security case for #2952 -- fail-closed control-plane policy.

Expected values are EC-owned literals from #2952: R1 rejects an
instance-scoped operator; R2/AC1 reject every production or development shape
that weakens the declared profile; R3/AC2 reject Lumen tiers and node-pool
names; R4 rejects an implicit or unbenchmarked default; and R7/AC4 returns an
actionable preflight rejection without changing the pure live transition model.
"""

from __future__ import annotations

from lumen.control_plane.admission import (
    decide_capacity_transition,
    decide_install_policy,
    decide_machine_policy,
)
from lumen.control_plane.spec import (
    CapacityPreflight,
    CapacityTransitionPolicy,
    InstallPolicySpec,
    MachinePolicySpec,
    SpendPolicy,
)
from lumen.control_plane.status import ControlPlaneStatus
from lumen.control_plane.verdict import Rejection

MINIMUM_CHECKS = 27

CONTROL_PLANE_2952_SECURITY_MATRIX = (
    ("instance_scoped_operator_is_rejected", "instance_scope_not_allowed"),
    ("instance_scope_rejection_names_scope", "control_plane_scope"),
    ("production_wrong_replica_count_is_rejected", "production_requires_two_fixed_replicas"),
    ("production_without_leader_election_is_rejected", "production_requires_leader_election"),
    ("production_without_topology_spread_is_rejected", "production_requires_topology_spread"),
    ("production_pdb_weaker_than_one_unavailable_is_rejected", "production_requires_pdb_max_unavailable_one"),
    ("production_hpa_is_rejected", "production_hpa_not_allowed"),
    ("development_wrong_replica_count_is_rejected", "development_requires_one_fixed_replica"),
    ("development_without_named_non_ha_limitation_is_rejected", "development_requires_non_ha_limitation"),
    ("lumen_tier_is_rejected", "tier_not_owned"),
    ("tier_rejection_names_tier", "tier"),
    ("node_pool_name_is_rejected", "node_pool_name_not_owned"),
    ("node_pool_rejection_names_node_pool", "node_pool_name"),
    ("implicit_default_machine_is_rejected", "implicit_machine_default_not_allowed"),
    ("default_rejection_names_default_machine", "default_machine"),
    ("missing_capacity_is_rejected", "missing_capacity"),
    ("missing_capacity_has_actionable_preflight_status", "preflight-required"),
    ("unsupported_capacity_is_rejected", "unsupported_capacity"),
    ("unsupported_capacity_has_actionable_preflight_status", "preflight-unsupported"),
    ("failed_preflight_preserves_ready_replicas", 2),
    ("failed_preflight_preserves_current_machine", "n2-standard-4"),
    ("failed_preflight_preserves_target_machine", "c3-standard-4"),
    ("failed_preflight_preserves_transition_generation_and_phase", (11, "preflight-pending")),
    ("unsupported_preflight_preserves_ready_replicas", 2),
    ("unsupported_preflight_preserves_current_machine", "n2-standard-4"),
    ("unsupported_preflight_preserves_target_machine", "c3-standard-4"),
    ("unsupported_preflight_preserves_transition_generation_and_phase", (11, "preflight-pending")),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _field(verdict) -> str:
    return verdict.field_path if isinstance(verdict, Rejection) else ""


def _production(**changes) -> InstallPolicySpec:
    values = {
        "control_plane_scope": "shared", "profile": "production", "fixed_replicas": 2,
        "leader_election_required": True, "topology_spread_required": True,
        "pdb_max_unavailable": 1, "hpa_enabled": False, "non_ha_limitation": "",
    }
    values.update(changes)
    return InstallPolicySpec(**values)


def verify_control_plane_2952_security() -> dict:
    checks = []

    # 1. R1 -- instance scope fails closed at install admission.
    v1 = decide_install_policy(_production(control_plane_scope="per-lumen-resource"))
    obs1 = _reason(v1)
    exp1 = CONTROL_PLANE_2952_SECURITY_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    # 2. R1 -- the refusal names the forbidden scope input.
    obs2 = _field(v1)
    exp2 = CONTROL_PLANE_2952_SECURITY_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2/AC1 -- no production input can lower its replica count.
    v3 = decide_install_policy(_production(fixed_replicas=1))
    obs3 = _reason(v3)
    exp3 = CONTROL_PLANE_2952_SECURITY_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    # 4. R2/AC1 -- no production input can remove leader election.
    v4 = decide_install_policy(_production(leader_election_required=False))
    obs4 = _reason(v4)
    exp4 = CONTROL_PLANE_2952_SECURITY_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5. R2/AC1 -- no production input can remove topology spread.
    v5 = decide_install_policy(_production(topology_spread_required=False))
    obs5 = _reason(v5)
    exp5 = CONTROL_PLANE_2952_SECURITY_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    # 6. R2/AC1 -- no production input can weaken its PDB.
    v6 = decide_install_policy(_production(pdb_max_unavailable=2))
    obs6 = _reason(v6)
    exp6 = CONTROL_PLANE_2952_SECURITY_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    # 7. R2/AC1 -- no production input can enable HPA.
    v7 = decide_install_policy(_production(hpa_enabled=True))
    obs7 = _reason(v7)
    exp7 = CONTROL_PLANE_2952_SECURITY_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2/AC1 -- development rejects every count other than one.
    v8 = decide_install_policy(_production(profile="development", fixed_replicas=2, leader_election_required=False, topology_spread_required=False, pdb_max_unavailable=0))
    obs8 = _reason(v8)
    exp8 = CONTROL_PLANE_2952_SECURITY_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    # 9. R2/AC1 -- development must name its non-HA limitation.
    v9 = decide_install_policy(_production(profile="development", fixed_replicas=1, leader_election_required=False, topology_spread_required=False, pdb_max_unavailable=0, non_ha_limitation=""))
    obs9 = _reason(v9)
    exp9 = CONTROL_PLANE_2952_SECURITY_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    base_machine = {
        "initial_machine": "n2-standard-4", "transitions": CapacityTransitionPolicy(mode="manual-approved"),
        "spend_policy": SpendPolicy(mode="bounded"), "tier": None,
        "node_pool_name": None, "default_machine": None,
    }

    # 10. R3/AC2 -- tiers are not an installation-policy vocabulary.
    v10 = decide_machine_policy(MachinePolicySpec(**{**base_machine, "tier": "premium"}))
    obs10 = _reason(v10)
    exp10 = CONTROL_PLANE_2952_SECURITY_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. R3/AC2 -- the tier refusal names the offending field.
    obs11 = _field(v10)
    exp11 = CONTROL_PLANE_2952_SECURITY_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    # 12. R3/AC2 -- node-pool names are not an installation-policy vocabulary.
    v12 = decide_machine_policy(MachinePolicySpec(**{**base_machine, "node_pool_name": "lumen-ssd"}))
    obs12 = _reason(v12)
    exp12 = CONTROL_PLANE_2952_SECURITY_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    # 13. R3/AC2 -- the node-pool refusal names the offending field.
    obs13 = _field(v12)
    exp13 = CONTROL_PLANE_2952_SECURITY_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R4/AC2 -- an implicit default remains forbidden pre-#2953.
    v14 = decide_machine_policy(MachinePolicySpec(**{**base_machine, "initial_machine": None, "default_machine": "n2-standard-4"}))
    obs14 = _reason(v14)
    exp14 = CONTROL_PLANE_2952_SECURITY_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R4/AC2 -- the default refusal names the offending field.
    obs15 = _field(v14)
    exp15 = CONTROL_PLANE_2952_SECURITY_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    live = ControlPlaneStatus(current_machine="n2-standard-4", target_machine="c3-standard-4", transition_generation=11, phase="preflight-pending", ready_replicas=2, installation_owner="platform-team")

    # 16. R7/AC4 -- missing capacity returns a named rejection.
    missing = decide_capacity_transition(live, CapacityPreflight(capacity=None, supported=None))
    obs16 = _reason(missing)
    exp16 = CONTROL_PLANE_2952_SECURITY_MATRIX[15][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    # 17. R7/AC4 -- missing capacity provides actionable preflight status.
    obs17 = missing.preflight_status if isinstance(missing, Rejection) else "admitted"
    exp17 = CONTROL_PLANE_2952_SECURITY_MATRIX[16][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    # 18. R7/AC4 -- unsupported capacity returns a separate named rejection.
    unsupported = decide_capacity_transition(live, CapacityPreflight(capacity="n2-standard-4", supported=False))
    obs18 = _reason(unsupported)
    exp18 = CONTROL_PLANE_2952_SECURITY_MATRIX[17][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    # 19. R7/AC4 -- unsupported capacity provides actionable preflight status.
    obs19 = unsupported.preflight_status if isinstance(unsupported, Rejection) else "admitted"
    exp19 = CONTROL_PLANE_2952_SECURITY_MATRIX[18][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    # 20. R7/AC4 -- failed pure preflight retains ready replicas.
    obs20 = missing.live_state.ready_replicas if isinstance(missing, Rejection) else -1
    exp20 = CONTROL_PLANE_2952_SECURITY_MATRIX[19][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    # 21. R7/AC4 -- it retains the controller-owned current machine.
    obs21 = missing.live_state.current_machine if isinstance(missing, Rejection) else "admitted"
    exp21 = CONTROL_PLANE_2952_SECURITY_MATRIX[20][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    # 22. R7/AC4 -- it retains the controller-owned target machine.
    obs22 = missing.live_state.target_machine if isinstance(missing, Rejection) else "admitted"
    exp22 = CONTROL_PLANE_2952_SECURITY_MATRIX[21][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    # 23. R7/AC4 -- it retains generation and phase as one transition identity.
    obs23 = (missing.live_state.transition_generation, missing.live_state.phase) if isinstance(missing, Rejection) else (None, None)
    exp23 = CONTROL_PLANE_2952_SECURITY_MATRIX[22][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})
    # 24. R7/AC4 -- unsupported preflight retains ready replicas.
    obs24 = unsupported.live_state.ready_replicas if isinstance(unsupported, Rejection) else -1
    exp24 = CONTROL_PLANE_2952_SECURITY_MATRIX[23][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    # 25. R7/AC4 -- unsupported preflight retains the controller-owned current machine.
    obs25 = unsupported.live_state.current_machine if isinstance(unsupported, Rejection) else "admitted"
    exp25 = CONTROL_PLANE_2952_SECURITY_MATRIX[24][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    # 26. R7/AC4 -- unsupported preflight retains the controller-owned target machine.
    obs26 = unsupported.live_state.target_machine if isinstance(unsupported, Rejection) else "admitted"
    exp26 = CONTROL_PLANE_2952_SECURITY_MATRIX[25][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})
    # 27. R7/AC4 -- unsupported preflight retains generation and phase as one transition identity.
    obs27 = (unsupported.live_state.transition_generation, unsupported.live_state.phase) if isinstance(unsupported, Rejection) else (None, None)
    exp27 = CONTROL_PLANE_2952_SECURITY_MATRIX[26][1]
    checks.append({"name": CONTROL_PLANE_2952_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})

    return {"case_id": "control-plane-2952-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
