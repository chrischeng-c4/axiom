"""EC security case for #2951 -- fail-closed split capacity holds.

Every expected literal is transcribed from #2951 R4/R5 and AC3.  It observes
the pure model's typed hold values, reasons, fields, and proposed topology;
runtime scheduler, Terraform, and actuator effects are intentionally excluded.
"""

from __future__ import annotations

from lumen.capacity.headroom import decide_split_headroom
from lumen.capacity.verdict import SplitHeadroomInput

MINIMUM_CHECKS = 15

CAPACITY_2951_SECURITY_MATRIX = (
    ("absent_target_capacity_holds_as_capacity_blocked", "CapacityBlocked"),
    ("absent_target_capacity_names_missing", "missing"),
    ("absent_target_capacity_names_the_target_capacity_field", "target_capacity"),
    ("absent_target_capacity_proposes_no_target_topology", None),
    ("full_target_capacity_holds_as_capacity_blocked", "CapacityBlocked"),
    ("unschedulable_target_capacity_holds_as_capacity_blocked", "CapacityBlocked"),
    ("temporary_member_headroom_holds_as_capacity_blocked", "CapacityBlocked"),
    ("temporary_member_headroom_names_temporary_member", "temporary member"),
    ("copy_headroom_holds_as_capacity_blocked", "CapacityBlocked"),
    ("catch_up_headroom_holds_as_capacity_blocked", "CapacityBlocked"),
    ("in_progress_mutation_holds_as_capacity_blocked", "CapacityBlocked"),
    ("neighbouring_complete_capacity_is_admitted", "admitted"),
    ("declared_profile_member_limit_holds_as_capacity_blocked", "CapacityBlocked"),
    ("declared_pool_member_maximum_holds_as_capacity_blocked", "CapacityBlocked"),
    ("headroom_policy_has_no_monetary_ceiling_input", ()),
)


def verify_capacity_2951_security() -> dict:
    checks = []

    # 1-4. R4/AC3 -- missing declared target capacity fails closed with a
    # typed hold, exact reason/field context, and no proposed topology.
    missing = decide_split_headroom(
        SplitHeadroomInput(
            profile_member_limit=3,
            pool_member_maximum=3,
            current_members=2,
            target_capacity="missing",
            temporary_member_headroom=1,
            copy_headroom=1,
            catch_up_headroom=1,
            mutation_in_progress=False,
        )
    )
    obs1 = missing.status
    exp1 = CAPACITY_2951_SECURITY_MATRIX[0][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = missing.reason
    exp2 = CAPACITY_2951_SECURITY_MATRIX[1][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    obs3 = missing.field_path
    exp3 = CAPACITY_2951_SECURITY_MATRIX[2][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    obs4 = missing.target_topology
    exp4 = CAPACITY_2951_SECURITY_MATRIX[3][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R4/AC3 -- declared full capacity is not silently treated as a target.
    full = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="full", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs5 = full.status
    exp5 = CAPACITY_2951_SECURITY_MATRIX[4][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4/AC3 -- an explicitly unschedulable target holds just as closed.
    unschedulable = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="unschedulable", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs6 = unschedulable.status
    exp6 = CAPACITY_2951_SECURITY_MATRIX[5][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7-8. R4/AC3 -- temporary-member migration headroom is independently
    # required and the refusal names that requirement.
    member = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=0, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs7 = member.status
    exp7 = CAPACITY_2951_SECURITY_MATRIX[6][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    obs8 = member.reason
    exp8 = CAPACITY_2951_SECURITY_MATRIX[7][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4/AC3 -- copy headroom cannot be substituted with member headroom.
    copy = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=0, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs9 = copy.status
    exp9 = CAPACITY_2951_SECURITY_MATRIX[8][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4/AC3 -- catch-up headroom is a third independent migration input.
    catch_up = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=0, mutation_in_progress=False)
    )
    obs10 = catch_up.status
    exp10 = CAPACITY_2951_SECURITY_MATRIX[9][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. AC3 -- an already active mutation holds even with otherwise complete
    # declared capacity.
    active = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=True)
    )
    obs11 = active.status
    exp11 = CAPACITY_2951_SECURITY_MATRIX[10][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R4 -- the nearest complete declared capacity remains admitted.
    admitted = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs12 = admitted.status
    exp12 = CAPACITY_2951_SECURITY_MATRIX[11][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R4 -- the declared profile member limit is an independent hard bound.
    profile_maximum = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=2, pool_member_maximum=3, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs13 = profile_maximum.status
    exp13 = CAPACITY_2951_SECURITY_MATRIX[12][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R4 -- a tighter declared pool maximum is also never exceeded.
    pool_maximum = decide_split_headroom(
        SplitHeadroomInput(profile_member_limit=3, pool_member_maximum=2, current_members=2, target_capacity="available", temporary_member_headroom=1, copy_headroom=1, catch_up_headroom=1, mutation_in_progress=False)
    )
    obs14 = pool_maximum.status
    exp14 = CAPACITY_2951_SECURITY_MATRIX[13][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R4 -- monetary ceilings are outside v1's capacity policy surface.
    obs15 = tuple(
        name
        for name in SplitHeadroomInput.__dataclass_fields__
        if "money" in name or "price" in name or "currency" in name or "cost" in name
    )
    exp15 = CAPACITY_2951_SECURITY_MATRIX[14][1]
    checks.append({"name": CAPACITY_2951_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {"case_id": "capacity-2951-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
