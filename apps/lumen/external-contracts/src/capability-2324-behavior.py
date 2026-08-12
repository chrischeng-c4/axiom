"""EC behavior case for #2324 -- capability ownership and reusable gates.

Every expected value below is an EC-owned literal transcribed from #2324:
R1 names the ordered reusable gates and the wrapper-before-pass rule; R2 names
the canonical shared owners, capability identifiers, and integration seams;
R3 constrains terminal results; R4 requires a total mixed-failure partition;
and AC2/AC3 require one declared owner and the closed terminal vocabulary.
"""

from __future__ import annotations

from lumen.capability_ownership import (
    classify_failure_slices,
    decide_terminal_result,
    ownership_inventory,
    required_gate_sequence,
    validate_ownership_inventory,
)

MINIMUM_CHECKS = 12

CAPABILITY_2324_BEHAVIOR_MATRIX = (
    (
        "reusable_gate_commands_are_ordered_and_exact",
        (
            "aw capability check --project lumen --verify --write-evidence",
            "aw health --project lumen full --verify-traceability --verify-cb --verify-cold --verify-tests",
        ),
    ),
    ("missing_live_command_requires_a_wrapper_before_passed", "require_shared_or_thin_app_wrapper_before_passed"),
    (
        "inventory_declares_each_canonical_owner",
        (
            ("auth", "service-auth"),
            ("cli", "cli-std"),
            ("http", "service-http"),
            ("index-storage-policy", "Lumen-domain"),
            ("kubernetes-render", "service-k8s"),
            ("lumen-crd-reshard-policy", "Lumen-domain"),
            ("observability", "service-observability"),
            ("peer-identity", "peer-tls"),
            ("raft-host", "raft-runtime"),
            ("search-planner", "Lumen-domain"),
        ),
    ),
    (
        "inventory_declares_capability_linkage",
        (
            ("auth", "security-hardening"),
            ("cli", "api-cli-agent-integration"),
            ("http", "api-cli-agent-integration"),
            ("index-storage-policy", "indexing"),
            ("kubernetes-render", "kubernetes-native-deployment"),
            ("lumen-crd-reshard-policy", "kubernetes-native-deployment"),
            ("observability", "operations-observability"),
            ("peer-identity", "security-hardening"),
            ("raft-host", "scaling-availability"),
            ("search-planner", "querying"),
        ),
    ),
    (
        "inventory_declares_integration_seam_linkage",
        (
            ("auth", "service_auth"),
            ("cli", "cli_std"),
            ("http", "service_http"),
            ("index-storage-policy", "lumen.index_storage_policy"),
            ("kubernetes-render", "service_k8s"),
            ("lumen-crd-reshard-policy", "lumen.operator.reshard_policy"),
            ("observability", "service_observability"),
            ("peer-identity", "peer_tls"),
            ("raft-host", "raft_runtime"),
            ("search-planner", "lumen.search_planner"),
        ),
    ),
    ("complete_inventory_is_admitted", "admitted"),
    ("no_failure_has_no_shared_non_domain_slice", ()),
    ("mixed_failure_retains_the_shared_concern", ("http",)),
    ("mixed_failure_retains_the_lumen_domain_concern", ("search-planner",)),
    ("shared_slice_requires_repair_and_rerun", "repair_and_rerun"),
    ("bounded_domain_issue_yields_tracked_skip", "tracked_skip(#2324)"),
    ("no_failure_yields_passed", "passed"),
)


def _outcome(verdict) -> str:
    """Read a value, never a design-computed validity boolean."""
    reason = getattr(verdict, "reason", None)
    return reason.value if reason is not None else verdict


def verify_capability_2324_behavior() -> dict:
    checks = []

    gate_plan = required_gate_sequence()

    # 1. R1 -- the preflight reusable gate precedes the final full gate.
    obs1 = tuple(gate_plan.commands)
    exp1 = CAPABILITY_2324_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- a missing live command is a design-time stop, not a pass claim.
    obs2 = gate_plan.missing_live_command_policy
    exp2 = CAPABILITY_2324_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    inventory = ownership_inventory()

    # 3. R2/AC2 -- every concern has its canonical shared owner or Lumen-domain.
    obs3 = tuple(sorted((concern, record["owner"]) for concern, record in inventory.items()))
    exp3 = CAPABILITY_2324_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- owner declarations remain connected to the capability registry.
    obs4 = tuple(sorted((concern, record["capability_id"]) for concern, record in inventory.items()))
    exp4 = CAPABILITY_2324_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2/AC2 -- each declaration names the source/integration seam it proves.
    obs5 = tuple(sorted((concern, record["integration_seam"]) for concern, record in inventory.items()))
    exp5 = CAPABILITY_2324_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. AC2 -- the complete literal inventory is an admitted ownership model.
    obs6 = _outcome(validate_ownership_inventory(inventory))
    exp6 = CAPABILITY_2324_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    no_failures = classify_failure_slices(())

    # 7. R3/R4 -- no failure cannot invent work for the shared owner.
    obs7 = tuple(no_failures.shared_non_domain)
    exp7 = CAPABILITY_2324_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    mixed = classify_failure_slices(
        (
            {"concern": "http", "owner": "service-http"},
            {"concern": "search-planner", "owner": "Lumen-domain"},
        )
    )

    # 8. R4 -- a mixed input leaves the shared slice visible for immediate repair.
    obs8 = tuple(mixed.shared_non_domain)
    exp8 = CAPABILITY_2324_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- it separately preserves the app-domain slice for issue tracking.
    obs9 = tuple(mixed.lumen_domain)
    exp9 = CAPABILITY_2324_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R3/R4 -- the shared partition carries the repair-and-rerun action.
    obs10 = mixed.shared_action
    exp10 = CAPABILITY_2324_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R3/AC3 -- the neighbouring app-only case is the sole skip path.
    obs11 = decide_terminal_result(("Lumen-domain",), 2324)
    exp11 = CAPABILITY_2324_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. AC3 -- terminal success has the issue's exact closed vocabulary.
    obs12 = decide_terminal_result((), None)
    exp12 = CAPABILITY_2324_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPABILITY_2324_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "capability-2324-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
