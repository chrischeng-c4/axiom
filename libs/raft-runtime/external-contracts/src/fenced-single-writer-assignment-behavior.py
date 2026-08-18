from __future__ import annotations

from raft_runtime.application.fenced_assignment import FencedAssignment
from raft_runtime.domain.fencing import FIRST_EPOCH, is_expired, next_epoch

MINIMUM_CHECKS = 14

FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX = (
    ("a_fresh_fence_is_idle_holds_no_token_and_has_allocated_nothing",
     (True, None, None, 0)),
    ("the_first_assignment_allocates_the_documented_first_epoch",
     (1, ('a', 1), 1, False)),
    ("every_successive_assignment_allocates_the_next_epoch",
     (1, 2, 3, 4)),
    ("an_assignment_is_live_up_to_but_not_including_its_deadline",
     (False, True, True, None, (100, 100))),
    ("validation_admits_the_current_holder_at_the_current_epoch",
     (None, None)),
    ("renewal_moves_the_deadline_forward_and_keeps_the_token",
     ((('a', 1), 200), 200, ('a', 1))),
    ("renewal_refuses_a_deadline_that_is_not_later_than_the_current_one",
     ((100, 100), (100, 50), 100)),
    ("release_by_the_holder_clears_the_fence_and_keeps_the_counter",
     (None, True, None, 1)),
    ("the_epoch_survives_release_so_the_next_holder_is_strictly_newer",
     (('b', 2), 2)),
    ("expiry_sweeps_a_fence_only_once_its_deadline_has_arrived",
     (False, False, True, True)),
    ("an_expired_fence_is_reassignable_to_a_different_owner",
     (('b', 2), ('b', 2), 2)),
    ("a_live_fence_refuses_a_second_owner_and_names_the_holder",
     (('a', 1), ('a', 1), 1)),
    ("an_assignment_must_expire_strictly_after_the_instant_it_is_made",
     ((0, 0), (-1, 0), True, 0)),
    ("a_renewal_keeps_the_epoch_so_a_renewed_holder_is_not_a_new_one",
     (('a', 1), 1, None, (1, 2))),
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


def verify_fenced_single_writer_assignment_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a fresh fence is idle holds no token and has allocated nothing
    exp1 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[0][1]
    fresh = FencedAssignment()
    obs1 = plain((fresh.idle(), fresh.token(), fresh.active(),
        fresh.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the first assignment allocates the documented first epoch
    exp2 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[1][1]
    first = FencedAssignment()
    obs2 = plain((FIRST_EPOCH, first.assign("a", 100, 0), first.epoch(),
        first.idle()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. every successive assignment allocates the next epoch
    exp3 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[2][1]
    counted = FencedAssignment()
    t1 = counted.assign("a", 100, 0)
    counted.expire(100)
    t2 = counted.assign("b", 200, 100)
    counted.expire(200)
    t3 = counted.assign("c", 300, 200)
    obs3 = plain((t1.epoch, t2.epoch, t3.epoch, next_epoch(3)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an assignment is live up to but not including its deadline
    exp4 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[3][1]
    live = FencedAssignment()
    live.assign("a", 100, 0)
    obs4 = plain((is_expired(100, 99), is_expired(100, 100),
        is_expired(100, 101), live.validate("a", 1, 99),
        live.validate("a", 1, 100)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. validation admits the current holder at the current epoch
    exp5 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[4][1]
    holder = FencedAssignment()
    holder.assign("a", 100, 0)
    obs5 = plain((holder.validate("a", 1, 0), holder.validate("a", 1,
        99)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. renewal moves the deadline forward and keeps the token
    exp6 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[5][1]
    renewed = FencedAssignment()
    renewed.assign("a", 100, 0)
    obs6 = plain((renewed.renew("a", 1, 200, 50),
        renewed.active().expires_at_ms, renewed.token()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. renewal refuses a deadline that is not later than the current one
    exp7 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[6][1]
    shorter = FencedAssignment()
    shorter.assign("a", 100, 0)
    obs7 = plain((shorter.renew("a", 1, 100, 50), shorter.renew("a", 1,
        50, 50), shorter.active().expires_at_ms))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. release by the holder clears the fence and keeps the counter
    exp8 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[7][1]
    released = FencedAssignment()
    released.assign("a", 100, 0)
    obs8 = plain((released.release("a", 1), released.idle(),
        released.token(), released.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the epoch survives release so the next holder is strictly newer
    exp9 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[8][1]
    handed = FencedAssignment()
    handed.assign("a", 100, 0)
    handed.release("a", 1)
    obs9 = plain((handed.assign("b", 200, 100), handed.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. expiry sweeps a fence only once its deadline has arrived
    exp10 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[9][1]
    swept = FencedAssignment()
    swept.assign("a", 100, 0)
    obs10 = plain((swept.expire(99), swept.idle(), swept.expire(100),
        swept.idle()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an expired fence is reassignable to a different owner
    exp11 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[10][1]
    lapsed = FencedAssignment()
    lapsed.assign("a", 100, 0)
    obs11 = plain((lapsed.assign("b", 200, 100), lapsed.token(),
        lapsed.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a live fence refuses a second owner and names the holder
    exp12 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[11][1]
    contended = FencedAssignment()
    contended.assign("a", 100, 0)
    obs12 = plain((contended.assign("b", 200, 50), contended.token(),
        contended.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. an assignment must expire strictly after the instant it is made
    exp13 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[12][1]
    backdated = FencedAssignment()
    obs13 = plain((backdated.assign("a", 0, 0), backdated.assign("a", -1,
        0), backdated.idle(), backdated.epoch()))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a renewal keeps the epoch so a renewed holder is not a new one
    exp14 = FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[13][1]
    extended = FencedAssignment()
    extended.assign("a", 100, 0)
    extended.renew("a", 1, 200, 50)
    obs14 = plain((extended.token(), extended.epoch(),
        extended.validate("a", 1, 150), extended.validate("a", 2, 150)))
    checks.append({"name": FENCED_SINGLE_WRITER_ASSIGNMENT_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "fenced-single-writer-assignment-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
