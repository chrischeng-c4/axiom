"""EC security case for #3066 -- fail-closed capacity admission and retirement.

Expected values are EC-owned literals from #3066's td-observable rules: R2
rejects non-direct keys and non-positive maxima; R3/R6 reject catalog misses
and Node evidence inconsistent with the authoritative machine-to-selector
mapping; R7 refuses runtime pool lifecycle actions; R8 names absent system
capacity; and R9 requires both zero assigned members and a separate explicit
destructive authorization before deletion.
"""

from __future__ import annotations

from lumen.capacity.admission import (
    decide_capacity_profiles,
    decide_installation_prerequisites,
)
from lumen.capacity.ownership import decide_capacity_action
from lumen.capacity.retirement import decide_profile_retirement
from lumen.capacity.selection import (
    decide_node_evidence,
    resolve_capacity_selector,
)
from lumen.capacity.spec import CapacityProfile, CatalogProfile
from lumen.capacity.verdict import Rejection

MINIMUM_CHECKS = 24

CAPACITY_3066_SECURITY_MATRIX = (
    ("zero_maximum_is_rejected", "maximum_nodes_must_be_positive"),
    ("zero_maximum_refusal_names_maximum_nodes", "profiles.n2-standard-8.max_nodes"),
    ("non_direct_machine_type_key_is_rejected", "direct_machine_type_required"),
    ("non_direct_machine_type_refusal_names_the_key", "profiles.profile-standard"),
    ("unknown_machine_type_selector_lookup_is_rejected", "unknown_machine_type"),
    ("unknown_machine_type_lookup_names_machine_type", "machine_type"),
    ("node_machine_type_mismatch_is_rejected", "node_machine_type_mismatch"),
    ("node_machine_type_mismatch_names_machine_type", "node_machine_type"),
    ("node_selector_mismatch_is_rejected", "node_selector_mismatch"),
    ("node_selector_mismatch_names_selector", "node_labels.lumen.axiom.dev/capacity-profile"),
    ("operator_create_is_refused", "capacity_pool_mutation_not_owned"),
    ("operator_create_refusal_names_actor", "actor"),
    ("operator_update_is_refused", "capacity_pool_mutation_not_owned"),
    ("operator_update_refusal_names_action", "action"),
    ("operator_delete_is_refused", "capacity_pool_mutation_not_owned"),
    ("operator_delete_refusal_names_action", "action"),
    ("zero_minimum_pools_without_system_capacity_are_rejected", "system_capacity_required"),
    ("system_capacity_refusal_names_system_capacity", "system_capacity"),
    ("profile_removal_with_assigned_members_cannot_delete", "assigned_members_must_be_zero"),
    ("assigned_members_refusal_names_assigned_members", "assigned_members"),
    ("zero_member_removal_without_second_authorization_cannot_delete", "destructive_authorization_required"),
    ("authorization_refusal_names_destructive_authorization", "destructive_authorization"),
    ("zero_member_removal_with_explicit_authorization_deletes", "delete"),
    ("neighbouring_valid_direct_profile_remains_admitted", "admitted"),
)


def _reason(value) -> str:
    return value.reason.value if isinstance(value, Rejection) else "admitted"


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


def verify_capacity_3066_security() -> dict:
    checks = []

    # 1-2. R2/AC4 -- name the invalid zero bound explicitly; an omitted bound
    # would only test a dataclass default, not the admission check.
    zero_maximum = decide_capacity_profiles({"n2-standard-8": CapacityProfile(min_nodes=0, max_nodes=0)})
    obs1 = _reason(zero_maximum)
    exp1 = CAPACITY_3066_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = zero_maximum.field_path
    exp2 = CAPACITY_3066_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3-4. R2 -- a profile alias is not a direct GCE machine type and must not
    # become an unbounded or implementation-defined pool key.
    invalid_key = decide_capacity_profiles({"profile-standard": CapacityProfile(min_nodes=0, max_nodes=3)})
    obs3 = _reason(invalid_key)
    exp3 = CAPACITY_3066_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = invalid_key.field_path
    exp4 = CAPACITY_3066_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    profile = _catalog_profile()
    catalog = {"n2-standard-8": profile}

    # 5-6. R3/R6 -- resolution refuses an undeclared direct type rather than
    # manufacturing a selector from the request or a namespace identity.
    unknown = resolve_capacity_selector(catalog, "n2-standard-16")
    obs5 = _reason(unknown)
    exp5 = CAPACITY_3066_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = unknown.field_path
    exp6 = CAPACITY_3066_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-10. R5/R6 -- a live Node is evidence only, so both its machine and its
    # selector must agree with the pre-existing catalog record.
    wrong_machine = decide_node_evidence(profile, {"lumen.axiom.dev/capacity-profile": "n2-standard-8"}, "n2-standard-16")
    obs7 = _reason(wrong_machine)
    exp7 = CAPACITY_3066_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = wrong_machine.field_path
    exp8 = CAPACITY_3066_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    wrong_selector = decide_node_evidence(profile, {"lumen.axiom.dev/capacity-profile": "n2-standard-16"}, "n2-standard-8")
    obs9 = _reason(wrong_selector)
    exp9 = CAPACITY_3066_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = wrong_selector.field_path
    exp10 = CAPACITY_3066_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-16. R7 -- each runtime lifecycle verb is a separate forbidden input;
    # the design must not protect create while allowing update or delete.
    operator_create = decide_capacity_action("operator", "create")
    obs11 = _reason(operator_create)
    exp11 = CAPACITY_3066_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = operator_create.field_path
    exp12 = CAPACITY_3066_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    operator_update = decide_capacity_action("operator", "update")
    obs13 = _reason(operator_update)
    exp13 = CAPACITY_3066_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = operator_update.field_path
    exp14 = CAPACITY_3066_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    operator_delete = decide_capacity_action("operator", "delete")
    obs15 = _reason(operator_delete)
    exp15 = CAPACITY_3066_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = operator_delete.field_path
    exp16 = CAPACITY_3066_SECURITY_MATRIX[15][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    profiles = {"n2-standard-8": CapacityProfile(min_nodes=0, max_nodes=3)}

    # 17-18. R8/AC4 -- no non-data capacity is a concrete failed prerequisite
    # when every Lumen data pool is explicitly allowed to start at zero.
    no_system = decide_installation_prerequisites({}, profiles)
    obs17 = _reason(no_system)
    exp17 = CAPACITY_3066_SECURITY_MATRIX[16][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = no_system.field_path
    exp18 = CAPACITY_3066_SECURITY_MATRIX[17][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19-23. R9 -- deletion has two independent explicit gates: no assigned
    # members and a second destructive authorization after the drain decision.
    assigned = decide_profile_retirement(profile, None, 1, True)
    obs19 = _reason(assigned)
    exp19 = CAPACITY_3066_SECURITY_MATRIX[18][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    obs20 = assigned.field_path
    exp20 = CAPACITY_3066_SECURITY_MATRIX[19][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})
    missing_authorization = decide_profile_retirement(profile, None, 0, False)
    obs21 = _reason(missing_authorization)
    exp21 = CAPACITY_3066_SECURITY_MATRIX[20][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = missing_authorization.field_path
    exp22 = CAPACITY_3066_SECURITY_MATRIX[21][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    deletion = decide_profile_retirement(profile, None, 0, True)
    obs23 = deletion.pool_disposition
    exp23 = CAPACITY_3066_SECURITY_MATRIX[22][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 24. R2 -- refusal is narrow: the adjacent literal direct-machine profile
    # with an explicit positive finite maximum remains admitted.
    neighbour = decide_capacity_profiles({"n2-standard-8": CapacityProfile(min_nodes=0, max_nodes=3)})
    obs24 = _reason(neighbour)
    exp24 = CAPACITY_3066_SECURITY_MATRIX[23][1]
    checks.append({"name": CAPACITY_3066_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})

    return {
        "case_id": "capacity-3066-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
