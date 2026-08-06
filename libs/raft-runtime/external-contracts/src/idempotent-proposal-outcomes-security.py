from __future__ import annotations

from raft_runtime.application.outcome_window import (
    OUTCOME_WINDOW_DEFAULT_CAPACITY,
    OutcomeWindow,
)
from raft_runtime.application.proposal_cache import (
    DEFAULT_PROPOSAL_CACHE_CAPACITY,
    ProposalCache,
)

MINIMUM_CHECKS = 11

IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX = (
    ("a_non_positive_cache_capacity_is_refused_at_construction",
     ('ValueError', 'ValueError', True, 1)),
    ("a_non_positive_window_capacity_is_refused_at_construction",
     ('ValueError', 'ValueError', 1, 0)),
    ("an_unbounded_retry_stream_cannot_grow_the_cache_past_its_capacity",
     (3, 3, ('k197', 'k198', 'k199'))),
    ("the_entry_dropped_under_pressure_is_always_the_oldest_one",
     (('b', 'c', 'd'), None, '2')),
    ("an_index_below_the_floor_is_refused_rather_than_resurrected",
     (2, None, None, 2, 'c')),
    ("the_floor_moves_to_the_cutoff_and_never_backwards",
     (1, 0, 0, 8, 0)),
    ("a_claim_after_eviction_reads_as_nothing_rather_than_a_stale_outcome",
     (2, None, None, 0, 2)),
    ("an_unbounded_index_stream_cannot_grow_the_window_past_its_span",
     (5, 95, 4)),
    ("restore_cannot_be_used_to_exceed_the_capacity",
     (2, ('k18', 'k19'), 2)),
    ("a_recorded_outcome_is_never_replaced_by_a_later_one",
     ('first', 'first', 'first', 1)),
    ("the_documented_default_capacities_are_the_ones_actually_used",
     (4096, 8192, 4096, 8192)),
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


def cache(capacity: int, *pairs: tuple[str, str]) -> ProposalCache:
    """A cache built by inserting each (key, outcome) pair in order."""
    built = ProposalCache(capacity)
    for key, outcome in pairs:
        built.insert(key, outcome.encode("ascii"))
    return built


def window(capacity: int, *pairs: tuple[int, str]) -> OutcomeWindow:
    """A window built by inserting each (index, outcome) pair in order."""
    built = OutcomeWindow(capacity)
    for index, outcome in pairs:
        built.insert(index, outcome.encode("ascii"))
    return built


def flooded(capacity: int, count: int) -> ProposalCache:
    """A cache after `count` retries, each under its own fresh key."""
    built = ProposalCache(capacity)
    for i in range(count):
        built.insert(f"k{i}", b"o")
    return built


def swept(capacity: int, count: int) -> OutcomeWindow:
    """A window after `count` inserts, each followed by an advance."""
    built = OutcomeWindow(capacity)
    for i in range(count):
        built.insert(i, b"o")
        built.advance(i)
    return built


def text(outcome: object) -> object:
    """A stored outcome as text, because an observation has to survive JSON."""
    if outcome is None:
        return None
    return outcome.decode("ascii")


def shown(built: ProposalCache) -> tuple[tuple[str, str], ...]:
    """A cache snapshot as text pairs, in the order the cache stores them."""
    return tuple((key, value.decode("ascii")) for key, value in built.snapshot())


def keys(built: ProposalCache) -> tuple[str, ...]:
    """Just the keys a cache is holding, in stored order."""
    return tuple(key for key, _ in built.snapshot())


def refused(make: object) -> object:
    """The name of the exception a construction raised, or its result."""
    try:
        return make()
    except Exception as err:
        return type(err).__name__


def verify_idempotent_proposal_outcomes_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a non positive cache capacity is refused at construction
    exp1 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[0][1]
    obs1 = plain((refused(lambda: ProposalCache(0)), refused(lambda:
        ProposalCache(-1)), refused(lambda: ProposalCache(1)) is not None,
        ProposalCache(1).capacity()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a non positive window capacity is refused at construction
    exp2 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[1][1]
    obs2 = plain((refused(lambda: OutcomeWindow(0)), refused(lambda:
        OutcomeWindow(-5)), OutcomeWindow(1).capacity(),
        OutcomeWindow(1).floor()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an unbounded retry stream cannot grow the cache past its capacity
    exp3 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[2][1]
    obs3 = plain((len(flooded(3, 200)), flooded(3, 200).capacity(),
        keys(flooded(3, 200))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the entry dropped under pressure is always the oldest one
    exp4 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[3][1]
    pressed = cache(3, ("a", "1"), ("b", "2"), ("c", "3"))
    pressed.insert("d", b"4")
    obs4 = plain((keys(pressed), pressed.get("a"),
        text(pressed.get("b"))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an index below the floor is refused rather than resurrected
    exp5 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[4][1]
    zombie = window(2, (0, "a"), (1, "b"), (2, "c"), (3, "d"))
    zombie.advance(4)
    zombie.insert(0, b"zombie")
    zombie.insert(1, b"zombie")
    obs5 = plain((zombie.floor(), zombie.claim(0), zombie.claim(1),
        len(zombie), text(zombie.claim(2))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the floor moves to the cutoff and never backwards
    exp6 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[5][1]
    ratchet = window(2, (5, "e"))
    forward = ratchet.advance(10)
    backward = ratchet.advance(3)
    sideways = ratchet.advance(9)
    obs6 = plain((forward, backward, sideways, ratchet.floor(),
        len(ratchet)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a claim after eviction reads as nothing rather than a stale outcome
    exp7 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[6][1]
    gone = window(1, (0, "old"), (1, "new"))
    dropped = gone.advance(3)
    obs7 = plain((dropped, gone.claim(0), gone.claim(1), len(gone),
        gone.floor()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. an unbounded index stream cannot grow the window past its span
    exp8 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[7][1]
    obs8 = plain((len(swept(4, 100)), swept(4, 100).floor(), swept(4,
        100).capacity()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. restore cannot be used to exceed the capacity
    exp9 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[8][1]
    stuffed = ProposalCache(2)
    stuffed.restore(tuple((f"k{i}", b"o") for i in range(20)))
    obs9 = plain((len(stuffed), keys(stuffed), stuffed.capacity()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a recorded outcome is never replaced by a later one
    exp10 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[9][1]
    sticky = cache(4, ("p", "first"))
    second_try = sticky.insert("p", b"second")
    third_try = sticky.insert("p", b"third")
    obs10 = plain((text(second_try), text(third_try),
        text(sticky.get("p")), len(sticky)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the documented default capacities are the ones actually used
    exp11 = IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[10][1]
    obs11 = plain((DEFAULT_PROPOSAL_CACHE_CAPACITY,
        OUTCOME_WINDOW_DEFAULT_CAPACITY, ProposalCache().capacity(),
        OutcomeWindow().capacity()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "idempotent-proposal-outcomes-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
