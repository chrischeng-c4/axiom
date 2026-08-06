from __future__ import annotations

from raft_runtime.application.fenced_assignment import FencedAssignment

MINIMUM_CHECKS = 13

FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX = (
    ("an_unassigned_fence_refuses_every_validation_and_every_release",
     ((), (), ())),
    ("a_superseded_epoch_is_refused_and_the_expected_one_is_named",
     ((2, 1), (2, 3), None)),
    ("a_different_owner_at_the_right_epoch_is_refused_and_named",
     (('a', 'b'), None)),
    ("a_stale_epoch_outranks_an_owner_mismatch",
     (2, 1)),
    ("expiry_is_reported_only_once_epoch_and_owner_agree",
     ((100, 100), (100, 150), ('a', 'b'), (1, 9))),
    ("an_expired_holder_cannot_renew_itself_back_to_life",
     ((100, 100), 100)),
    ("a_non_holder_cannot_renew_the_fence",
     (('a', 'b'), 100)),
    ("a_non_holder_cannot_release_the_fence",
     (('a', 'b'), False, ('a', 1))),
    ("a_superseded_holder_cannot_release_the_current_assignment",
     ((2, 1), False, None, True)),
    ("release_reports_the_epoch_problem_before_the_owner_problem",
     (2, 1)),
    ("a_second_release_finds_nothing_left_to_release",
     ((), True)),
    ("a_swept_fence_leaves_no_token_behind",
     (True, None, 1, ())),
    ("reassignment_is_refused_up_to_the_deadline_and_admitted_at_it",
     (('a', 1), ('b', 2), 2)),
)


def plain(value: object) -> object:
    """A literal-shaped view: records by their fields, enum members by value.

    An expected value has to be a plain literal, and `repr` of a dataclass or
    an enum member is not one. Reading a record as the tuple of its fields
    keeps every field observable while staying transcribable.
    """
    fields = getattr(type(value), "__dataclass_fields__", None)
    if fields is not None:
        return tuple(plain(getattr(value, n)) for n in fields)
    if getattr(type(value), "__members__", None) is not None:
        return plain(value.value)
    if isinstance(value, tuple):
        return tuple(plain(v) for v in value)
    if isinstance(value, list):
        return [plain(v) for v in value]
    if isinstance(value, dict):
        return {k: plain(v) for k, v in value.items()}
    return value


def verify_fenced_single_writer_assignment_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an unassigned fence refuses every validation and every release
    exp1 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[0][1]
    empty = FencedAssignment()
    obs1 = plain((empty.validate("a", 1, 0), empty.validate("", 0, 0),
        empty.release("a", 1)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a superseded epoch is refused and the expected one is named
    exp2 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[1][1]
    superseded = FencedAssignment()
    superseded.assign("a", 100, 0)
    superseded.expire(100)
    superseded.assign("a", 200, 100)
    obs2 = plain((superseded.validate("a", 1, 150),
        superseded.validate("a", 3, 150), superseded.validate("a", 2,
        150)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a different owner at the right epoch is refused and named
    exp3 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[2][1]
    impostor = FencedAssignment()
    impostor.assign("a", 100, 0)
    obs3 = plain((impostor.validate("b", 1, 50), impostor.validate("a", 1,
        50)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a stale epoch outranks an owner mismatch
    exp4 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[3][1]
    outranked = FencedAssignment()
    outranked.assign("a", 100, 0)
    outranked.expire(100)
    outranked.assign("a", 200, 100)
    obs4 = plain(outranked.validate("b", 1, 150))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. expiry is reported only once epoch and owner agree
    exp5 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[4][1]
    ordered = FencedAssignment()
    ordered.assign("a", 100, 0)
    obs5 = plain((ordered.validate("a", 1, 100), ordered.validate("a", 1,
        150), ordered.validate("b", 1, 100), ordered.validate("a", 9,
        100)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an expired holder cannot renew itself back to life
    exp6 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[5][1]
    stale = FencedAssignment()
    stale.assign("a", 100, 0)
    obs6 = plain((stale.renew("a", 1, 300, 100),
        stale.active().expires_at_ms))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a non holder cannot renew the fence
    exp7 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[6][1]
    foreign = FencedAssignment()
    foreign.assign("a", 100, 0)
    obs7 = plain((foreign.renew("b", 1, 300, 50),
        foreign.active().expires_at_ms))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a non holder cannot release the fence
    exp8 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[7][1]
    thief = FencedAssignment()
    thief.assign("a", 100, 0)
    obs8 = plain((thief.release("b", 1), thief.idle(), thief.token()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a superseded holder cannot release the current assignment
    exp9 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[8][1]
    old = FencedAssignment()
    old.assign("a", 100, 0)
    old.expire(100)
    old.assign("a", 200, 100)
    obs9 = plain((old.release("a", 1), old.idle(), old.release("a", 2),
        old.idle()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. release reports the epoch problem before the owner problem
    exp10 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[9][1]
    ranked = FencedAssignment()
    ranked.assign("a", 100, 0)
    ranked.expire(100)
    ranked.assign("a", 200, 100)
    obs10 = plain(ranked.release("b", 1))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a second release finds nothing left to release
    exp11 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[10][1]
    twice = FencedAssignment()
    twice.assign("a", 100, 0)
    twice.release("a", 1)
    obs11 = plain((twice.release("a", 1), twice.idle()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a swept fence leaves no token behind
    exp12 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[11][1]
    residue = FencedAssignment()
    residue.assign("a", 100, 0)
    residue.expire(100)
    obs12 = plain((residue.idle(), residue.token(), residue.epoch(),
        residue.validate("a", 1, 100)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. reassignment is refused up to the deadline and admitted at it
    exp13 = FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[12][1]
    boundary = FencedAssignment()
    boundary.assign("a", 100, 0)
    obs13 = plain((boundary.assign("b", 200, 99), boundary.assign("b",
        200, 100), boundary.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "fenced-single-writer-assignment-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
