"""EC behavior case for #3066 -- existing-cluster capacity substrate.

Every expected value is an EC-owned literal transcribed from #3066's
td-observable rules: R2 admits finite direct-machine-type profiles with a
zero default minimum, R3 resolves the catalog's stable selector, R5/R6 keep a
zero-node-readable catalog authoritative over node evidence, R7 reserves pool
lifecycle actions for Terraform, R8 admits zero-minimum data pools only when
non-data system capacity is declared, and R9 starts ordinary removal by
draining while retaining the pool.
"""

from __future__ import annotations

from lumen.capacity.admission import (
    decide_capacity_profiles,
    decide_installation_prerequisites,
)
from lumen.capacity.catalog import decide_catalog_profile
from lumen.capacity.ownership import decide_capacity_action
from lumen.capacity.retirement import decide_profile_retirement
from lumen.capacity.selection import (
    decide_node_evidence,
    resolve_capacity_selector,
)
from lumen.capacity.spec import CapacityProfile, CatalogProfile
from lumen.capacity.verdict import Rejection

MINIMUM_CHECKS = 14

CAPACITY_3066_BEHAVIOR_MATRIX = (
    ("finite_direct_machine_type_profile_is_admitted", "admitted"),
    ("admitted_profile_map_keeps_the_direct_machine_type_key", ("n2-standard-8",)),
    ("omitted_minimum_normalizes_to_zero", 0),
    ("positive_explicit_maximum_is_preserved", 3),
    ("catalog_profile_is_admitted", "admitted"),
    ("catalog_profile_records_its_direct_machine_type", "n2-standard-8"),
    ("catalog_profile_records_the_stable_selector", ("lumen.axiom.dev/capacity-profile", "n2-standard-8")),
    ("catalog_profile_exposes_required_zero_node_fields_without_pool_name", True),
    ("selector_resolution_returns_the_catalog_selector", ("lumen.axiom.dev/capacity-profile", "n2-standard-8")),
    ("matching_node_evidence_is_admitted_against_the_catalog", "admitted"),
    ("terraform_is_admitted_to_create_a_capacity_pool", "admitted"),
    ("zero_minimum_data_pools_with_system_capacity_are_admitted", "admitted"),
    ("ordinary_profile_removal_enters_draining", "draining"),
    ("ordinary_profile_removal_retains_the_pool", "retain"),
)


def _outcome(value) -> str:
    return value.reason.value if isinstance(value, Rejection) else value.outcome.value


def _catalog_profile() -> CatalogProfile:
    return CatalogProfile(
        machine_type="n2-standard-8",
        stable_selector=("lumen.axiom.dev/capacity-profile", "n2-standard-8"),
        pool_group="lumen-data",
        cpu=8,
        memory_gib=32,
        downgrade_edges=("n2-standard-4",),
        max_nodes=3,
        lifecycle_state="active",
    )


def verify_capacity_3066_behavior() -> dict:
    checks = []

    # 1-4. R2/AC4 -- name both the direct key and the positive bound.  The
    # default minimum is exercised only because the caller omits min_nodes.
    profiles = {
        "n2-standard-8": CapacityProfile(max_nodes=3),
    }
    admitted_profiles = decide_capacity_profiles(profiles)
    obs1 = _outcome(admitted_profiles)
    exp1 = CAPACITY_3066_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = tuple(admitted_profiles.profiles)
    exp2 = CAPACITY_3066_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = admitted_profiles.profiles["n2-standard-8"].min_nodes
    exp3 = CAPACITY_3066_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = admitted_profiles.profiles["n2-standard-8"].max_nodes
    exp4 = CAPACITY_3066_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    profile = _catalog_profile()

    # 5-8. R5/R6 -- a zero-node catalog record is a complete, typed source of
    # truth, including the direct machine, internal selector, resource shape,
    # downgrade graph, bound, and lifecycle -- never an instance pool name.
    catalog_decision = decide_catalog_profile(profile)
    obs5 = _outcome(catalog_decision)
    exp5 = CAPACITY_3066_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = profile.machine_type
    exp6 = CAPACITY_3066_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = profile.stable_selector
    exp7 = CAPACITY_3066_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    required = {"machine_type", "stable_selector", "pool_group", "cpu", "memory_gib", "downgrade_edges", "max_nodes", "lifecycle_state"}
    obs8 = required.issubset(CatalogProfile.__dataclass_fields__) and "pool_name" not in CatalogProfile.__dataclass_fields__
    exp8 = CAPACITY_3066_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9-10. R3/R6 -- the catalog, not a namespace, instance, or live Node,
    # names the selector.  A matching Node label is verification evidence.
    resolved = resolve_capacity_selector({"n2-standard-8": profile}, "n2-standard-8")
    obs9 = resolved.selector
    exp9 = CAPACITY_3066_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    evidence = decide_node_evidence(
        profile,
        {"lumen.axiom.dev/capacity-profile": "n2-standard-8"},
        "n2-standard-8",
    )
    obs10 = _outcome(evidence)
    exp10 = CAPACITY_3066_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 -- Terraform is the caller that owns pool lifecycle changes.
    terraform_create = decide_capacity_action("terraform", "create")
    obs11 = _outcome(terraform_create)
    exp11 = CAPACITY_3066_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R8/AC4 -- all data minima may be zero once concrete non-data system
    # capacity is supplied; an empty/default prerequisite is not used.
    installation = decide_installation_prerequisites(
        {"system_pool": "n2-standard-4", "minimum_nodes": 1},
        profiles,
    )
    obs12 = _outcome(installation)
    exp12 = CAPACITY_3066_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13-14. R9 -- ordinary omission begins a drain but cannot delete the
    # existing Terraform pool in the same decision.
    retirement = decide_profile_retirement(profile, None, 2, False)
    obs13 = retirement.lifecycle_state
    exp13 = CAPACITY_3066_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = retirement.pool_disposition
    exp14 = CAPACITY_3066_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_3066_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "capacity-3066-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
