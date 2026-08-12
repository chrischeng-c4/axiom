"""EC behavior case for #2952 -- installation-owned control-plane policy.

Every expected value below is an EC-owned literal transcribed from #2952:
R1 requires one shared control plane; R2 and AC1 require the fixed production
HA shape and an explicitly non-HA development shape; R3 and AC2 require a
direct-machine vocabulary with an explicit initial machine, transitions, and
spend policy; R4 forbids a calibration-free default; and R5/AC3 require a
reapply merge to retain controller-owned transition state.
"""

from __future__ import annotations

from lumen.control_plane.admission import decide_install_policy, decide_machine_policy
from lumen.control_plane.spec import (
    CapacityTransitionPolicy,
    InstallPolicySpec,
    InstallationInput,
    MachinePolicySpec,
    SpendPolicy,
)
from lumen.control_plane.status import ControlPlaneStatus, reapply_install_input
from lumen.control_plane.verdict import AdmittedInstallPolicy, AdmittedMachinePolicy

MINIMUM_CHECKS = 17

CONTROL_PLANE_2952_BEHAVIOR_MATRIX = (
    ("shared_control_plane_scope_is_admitted", "shared"),
    ("production_uses_exactly_two_fixed_replicas", 2),
    ("production_requires_leader_election", True),
    ("production_requires_topology_spread", True),
    ("production_pdb_allows_at_most_one_unavailable", 1),
    ("production_disables_hpa", False),
    ("development_uses_exactly_one_fixed_replica", 1),
    ("development_declares_non_ha", "non-HA"),
    ("explicit_direct_machine_is_preserved", "n2-standard-4"),
    ("transition_policy_is_preserved", "manual-approved"),
    ("spend_policy_is_preserved", "bounded"),
    ("absent_initial_machine_is_an_admitted_no_default_state", "no-default"),
    ("reapply_retains_current_machine", "n2-standard-4"),
    ("reapply_retains_target_machine", "c3-standard-4"),
    ("reapply_retains_transition_generation", 11),
    ("reapply_retains_transition_phase", "preflight-pending"),
    ("reapply_takes_installation_owned_field", "platform-team"),
)


def verify_control_plane_2952_behavior() -> dict:
    checks = []

    production = InstallPolicySpec(
        control_plane_scope="shared",
        profile="production",
        fixed_replicas=2,
        leader_election_required=True,
        topology_spread_required=True,
        pdb_max_unavailable=1,
        hpa_enabled=False,
        non_ha_limitation="",
    )
    production_verdict = decide_install_policy(production)

    # 1. R1 -- Lumen has one cluster control plane, not one per CR.
    obs1 = production_verdict.control_plane_scope if isinstance(production_verdict, AdmittedInstallPolicy) else "rejected"
    exp1 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2/AC1 -- production uses exactly two fixed replicas.
    obs2 = production_verdict.fixed_replicas if isinstance(production_verdict, AdmittedInstallPolicy) else -1
    exp2 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    # 3. R2/AC1 -- leader election is an admitted production requirement.
    obs3 = production_verdict.leader_election_required if isinstance(production_verdict, AdmittedInstallPolicy) else None
    exp3 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    # 4. R2/AC1 -- topology spread is an admitted production requirement.
    obs4 = production_verdict.topology_spread_required if isinstance(production_verdict, AdmittedInstallPolicy) else None
    exp4 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    # 5. R2/AC1 -- the PDB admits at most one unavailable replica.
    obs5 = production_verdict.pdb_max_unavailable if isinstance(production_verdict, AdmittedInstallPolicy) else -1
    exp5 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    # 6. R2/AC1 -- HPA is explicitly disabled in this profile.
    obs6 = production_verdict.hpa_enabled if isinstance(production_verdict, AdmittedInstallPolicy) else None
    exp6 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    development_verdict = decide_install_policy(
        InstallPolicySpec(
            control_plane_scope="shared", profile="development", fixed_replicas=1,
            leader_election_required=False, topology_spread_required=False,
            pdb_max_unavailable=0, hpa_enabled=False, non_ha_limitation="non-HA",
        )
    )

    # 7. R2/AC1 -- development is exactly one fixed replica.
    obs7 = development_verdict.fixed_replicas if isinstance(development_verdict, AdmittedInstallPolicy) else -1
    exp7 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    # 8. R2/AC1 -- development exposes its non-HA limitation.
    obs8 = development_verdict.non_ha_limitation if isinstance(development_verdict, AdmittedInstallPolicy) else "rejected"
    exp8 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    explicit_machine = MachinePolicySpec(
        initial_machine="n2-standard-4",
        transitions=CapacityTransitionPolicy(mode="manual-approved"),
        spend_policy=SpendPolicy(mode="bounded"),
        tier=None,
        node_pool_name=None,
        default_machine=None,
    )
    machine_verdict = decide_machine_policy(explicit_machine)

    # 9. R3/AC2 -- an explicit direct GCE machine is preserved.
    obs9 = machine_verdict.initial_machine if isinstance(machine_verdict, AdmittedMachinePolicy) else "rejected"
    exp9 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    # 10. R3/AC2 -- the allowed transition policy is preserved.
    obs10 = machine_verdict.transitions.mode if isinstance(machine_verdict, AdmittedMachinePolicy) else "rejected"
    exp10 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    # 11. R3/AC2 -- the allowed spend policy is preserved.
    obs11 = machine_verdict.spend_policy.mode if isinstance(machine_verdict, AdmittedMachinePolicy) else "rejected"
    exp11 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    no_default_verdict = decide_machine_policy(
        MachinePolicySpec(initial_machine=None, transitions=CapacityTransitionPolicy(mode="manual-approved"), spend_policy=SpendPolicy(mode="bounded"), tier=None, node_pool_name=None, default_machine=None)
    )

    # 12. R4 -- absence is an explicit no-default state, not hidden selection.
    obs12 = no_default_verdict.default_state if isinstance(no_default_verdict, AdmittedMachinePolicy) else "rejected"
    exp12 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    live = ControlPlaneStatus(current_machine="n2-standard-4", target_machine="c3-standard-4", transition_generation=11, phase="preflight-pending", installation_owner="old-owner")
    reapplied = reapply_install_input(live, InstallationInput(installation_owner="platform-team"))

    # 13. R5/AC3 -- reapply retains the live current machine.
    obs13 = reapplied.current_machine
    exp13 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    # 14. R5/AC3 -- reapply retains the live target machine.
    obs14 = reapplied.target_machine
    exp14 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    # 15. R5/AC3 -- reapply retains the live transition generation.
    obs15 = reapplied.transition_generation
    exp15 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    # 16. R5/AC3 -- reapply retains the live transition phase.
    obs16 = reapplied.phase
    exp16 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    # 17. R5/AC3 -- reapply does take installation-owned input.
    obs17 = reapplied.installation_owner
    exp17 = CONTROL_PLANE_2952_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": CONTROL_PLANE_2952_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    return {"case_id": "control-plane-2952-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
