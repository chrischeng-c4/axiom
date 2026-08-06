from __future__ import annotations

from raft_runtime.application.outcome_window import (
    OUTCOME_WINDOW_DEFAULT_CAPACITY,
    OutcomeWindow,
)
from raft_runtime.application.proposal_cache import (
    DEFAULT_PROPOSAL_CACHE_CAPACITY,
    ProposalCache,
)

MINIMUM_CHECKS = 13

IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX = (
    ("the_first_recorded_outcome_survives_a_retry_under_the_same_key",
     ('first', 'first', 1)),
    ("a_different_key_records_its_own_outcome_independently",
     ('first', 'other', 2, (('p1', 'first'), ('p2', 'other')))),
    ("the_cache_reports_its_capacity_and_the_documented_default",
     (4096, 4096, 3, 0)),
    ("an_over_capacity_insert_drops_the_oldest_entry",
     ((('b', '2'), ('c', '3')), 2, None)),
    ("a_repeated_key_does_not_consume_a_capacity_slot",
     ((('a', '1'), ('b', '2')), 2)),
    ("a_snapshot_lists_entries_in_insertion_order_and_restore_reproduces_it",
     ((('c', '3'), ('a', '1'), ('b', '2')), (('c', '3'), ('a', '1'), ('b', '2')), True)),
    ("restore_replaces_whatever_the_cache_already_held",
     ((('y', 'new'),), None, 1)),
    ("restore_keeps_the_first_of_two_entries_under_one_key",
     ((('k', 'one'),), 1)),
    ("restore_trims_to_capacity_from_the_oldest_end",
     ((('b', '2'), ('c', '3')), 2)),
    ("a_key_that_was_never_recorded_reads_as_nothing",
     (None, None, 0, '1')),
    ("an_outcome_window_round_trips_an_insert_and_a_claim",
     ('three', 'four', None, 2, 0)),
    ("advance_evicts_strictly_below_the_cutoff_so_the_boundary_survives",
     (2, 2, 2, 'c', 'd', None)),
    ("an_advance_that_does_not_move_the_cutoff_evicts_nothing",
     (0, 0, 0, 2, 'a')),
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


def verify_idempotent_proposal_outcomes_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the first recorded outcome survives a retry under the same key
    exp1 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[0][1]
    retried = cache(4, ("p1", "first"))
    obs1 = plain((text(retried.insert("p1", b"second")),
        text(retried.get("p1")), len(retried)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a different key records its own outcome independently
    exp2 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[1][1]
    two = cache(4, ("p1", "first"), ("p2", "other"))
    obs2 = plain((text(two.get("p1")), text(two.get("p2")), len(two),
        shown(two)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the cache reports its capacity and the documented default
    exp3 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((ProposalCache().capacity(),
        DEFAULT_PROPOSAL_CACHE_CAPACITY, ProposalCache(3).capacity(),
        len(ProposalCache(3))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an over capacity insert drops the oldest entry
    exp4 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[3][1]
    full = cache(2, ("a", "1"), ("b", "2"))
    full.insert("c", b"3")
    obs4 = plain((shown(full), len(full), full.get("a")))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a repeated key does not consume a capacity slot
    exp5 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[4][1]
    repeated = cache(2, ("a", "1"), ("a", "9"), ("b", "2"))
    obs5 = plain((shown(repeated), len(repeated)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a snapshot lists entries in insertion order and restore reproduces it
    exp6 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[5][1]
    source = cache(4, ("c", "3"), ("a", "1"), ("b", "2"))
    copy = ProposalCache(4)
    copy.restore(source.snapshot())
    obs6 = plain((shown(source), shown(copy), shown(source) ==
        shown(copy)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. restore replaces whatever the cache already held
    exp7 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[6][1]
    replaced = cache(4, ("x", "old"))
    replaced.restore((("y", b"new"),))
    obs7 = plain((shown(replaced), replaced.get("x"), len(replaced)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. restore keeps the first of two entries under one key
    exp8 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[7][1]
    deduped = ProposalCache(4)
    deduped.restore((("k", b"one"), ("k", b"two")))
    obs8 = plain((shown(deduped), len(deduped)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. restore trims to capacity from the oldest end
    exp9 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[8][1]
    trimmed = ProposalCache(2)
    trimmed.restore((("a", b"1"), ("b", b"2"), ("c", b"3")))
    obs9 = plain((shown(trimmed), len(trimmed)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a key that was never recorded reads as nothing
    exp10 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((ProposalCache(4).get("absent"), cache(4, ("a",
        "1")).get("b"), len(ProposalCache(4)), text(cache(4, ("a",
        "1")).get("a"))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an outcome window round trips an insert and a claim
    exp11 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[10][1]
    pair = window(8, (3, "three"), (4, "four"))
    obs11 = plain((text(pair.claim(3)), text(pair.claim(4)),
        pair.claim(5), len(pair), pair.floor()))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. advance evicts strictly below the cutoff so the boundary survives
    exp12 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[11][1]
    edge = window(2, (0, "a"), (1, "b"), (2, "c"), (3, "d"))
    moved = edge.advance(4)
    obs12 = plain((moved, edge.floor(), len(edge), text(edge.claim(2)),
        text(edge.claim(3)), edge.claim(1)))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. an advance that does not move the cutoff evicts nothing
    exp13 = IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[12][1]
    still = window(4, (0, "a"), (1, "b"))
    first = still.advance(2)
    second = still.advance(4)
    obs13 = plain((first, second, still.floor(), len(still),
        text(still.claim(0))))
    checks.append({"name": IDEMPOTENT_PROPOSAL_OUTCOMES_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "idempotent-proposal-outcomes-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
