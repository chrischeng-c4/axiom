from __future__ import annotations

from raft_runtime.application.store import RaftStore
from raft_runtime.infrastructure.applied_index_file import (
    decode_applied_index,
)

MINIMUM_CHECKS = 11

DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX = (
    ("a_corrupt_payload_is_an_error_rather_than_a_silently_lowered_floor",
     ('AppliedIndexError', 'AppliedIndexError', 'AppliedIndexError')),
    ("a_payload_that_merely_contains_a_digit_is_refused_whole",
     ('AppliedIndexError', 'AppliedIndexError', 'AppliedIndexError')),
    ("a_non_ascii_payload_raises_instead_of_resetting_the_floor",
     ('AppliedIndexError', 'AppliedIndexError')),
    ("a_signed_or_negative_payload_is_not_a_valid_applied_floor",
     ('AppliedIndexError', 'AppliedIndexError', 'AppliedIndexError')),
    ("a_refused_payload_never_yields_a_number_the_caller_could_use",
     (False, False, 0)),
    ("seeding_refuses_a_path_that_already_holds_a_snapshot",
     (('/held',), '/free')),
    ("the_refusal_names_the_exact_path_it_refused",
     (('/a',), ('/b',))),
    ("a_vacant_path_is_handed_back_unchanged",
     ('/fresh', '')),
    ("the_existence_oracle_is_asked_about_exactly_the_path_being_seeded",
     (('/held',), '/free', ('/held', '/free'))),
    ("seeding_never_touches_the_durable_hard_state",
     (('/held',), '/free', None)),
    ("a_refusal_and_an_acceptance_are_not_the_same_shape",
     (False, True, ('/held',))),
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


def outcome(call: object, *args: object) -> object:
    """What a call produced: its plain value, or the name of what it raised.

    A refusal has to be observable without a `try` in the case body, so the
    refusal is folded into the observation itself.
    """
    try:
        return plain(call(*args))
    except Exception as err:
        return type(err).__name__


class Probe:
    """An existence oracle that records exactly which paths it was asked about."""

    def __init__(self, taken: tuple[str, ...]) -> None:
        self.taken = taken
        self.asked: list[str] = []

    def __call__(self, path: str) -> bool:
        self.asked.append(path)
        return path in self.taken


def verify_deterministic_replicated_state_machine_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a corrupt payload is an error rather than a silently lowered floor
    exp1 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[0][1]
    obs1 = plain((outcome(decode_applied_index, b"garbage"),
        outcome(decode_applied_index, b"\x00"),
        outcome(decode_applied_index, b"1 2")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a payload that merely contains a digit is refused whole
    exp2 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[1][1]
    obs2 = plain((outcome(decode_applied_index, b"12a"),
        outcome(decode_applied_index, b"a1"),
        outcome(decode_applied_index, b"0x10")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a non ascii payload raises instead of resetting the floor
    exp3 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[2][1]
    obs3 = plain((outcome(decode_applied_index, b"\xff\xfe1"),
        outcome(decode_applied_index, b"\xc3\xa9")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a signed or negative payload is not a valid applied floor
    exp4 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[3][1]
    obs4 = plain((outcome(decode_applied_index, b"-1"),
        outcome(decode_applied_index, b"+1"),
        outcome(decode_applied_index, b"1.0")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a refused payload never yields a number the caller could use
    exp5 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[4][1]
    obs5 = plain((outcome(decode_applied_index, b"9a") == 9,
        outcome(decode_applied_index, b"9a") == 0,
        outcome(decode_applied_index, b"")))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. seeding refuses a path that already holds a snapshot
    exp6 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[5][1]
    guarded = RaftStore()
    occupied = Probe(("/held",))
    obs6 = plain((guarded.seed_snapshot("/held", occupied),
        guarded.seed_snapshot("/free", occupied)))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the refusal names the exact path it refused
    exp7 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[6][1]
    named = RaftStore()
    both = Probe(("/a", "/b"))
    obs7 = plain((named.seed_snapshot("/a", both),
        named.seed_snapshot("/b", both)))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a vacant path is handed back unchanged
    exp8 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[7][1]
    vacant = RaftStore()
    empty = Probe(())
    obs8 = plain((vacant.seed_snapshot("/fresh", empty),
        vacant.seed_snapshot("", empty)))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the existence oracle is asked about exactly the path being seeded
    exp9 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[8][1]
    asked = RaftStore()
    watcher = Probe(("/held",))
    obs9 = plain((asked.seed_snapshot("/held", watcher),
        asked.seed_snapshot("/free", watcher), tuple(watcher.asked)))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. seeding never touches the durable hard state
    exp10 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[9][1]
    untouched = RaftStore()
    guard = Probe(("/held",))
    obs10 = plain((untouched.seed_snapshot("/held", guard),
        untouched.seed_snapshot("/free", guard), untouched.last_saved()))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a refusal and an acceptance are not the same shape
    exp11 = DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[10][1]
    shaped = RaftStore()
    oracle = Probe(("/held",))
    obs11 = plain((type(shaped.seed_snapshot("/held", oracle)) is str,
        type(shaped.seed_snapshot("/free", oracle)) is str,
        shaped.seed_snapshot("/held", oracle)))
    checks.append({"name": DETERMINISTIC_REPLICATED_STATE_MACHINE_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "deterministic-replicated-state-machine-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
