from __future__ import annotations

from raft_runtime.domain.read_consistency import (
    ANY,
    LEADER,
    READ_CONSISTENCY_HEADER,
    Bounded,
    is_strongest,
    tolerated_staleness_ms,
)
from raft_runtime.infrastructure.headers import (
    BOUNDED_PREFIX,
    BOUNDED_SUFFIX,
    parse_read_consistency,
)

MINIMUM_CHECKS = 11

READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX = (
    ("the_header_a_client_sets_and_the_bounded_form_it_spells",
     ('x-read-consistency', 'bounded(', ')')),
    ("an_absent_header_selects_a_leader_read",
     (('Leader', ()), True, None)),
    ("an_explicit_leader_request_stays_a_leader_read",
     (('Leader', ()), True, None)),
    ("an_explicit_any_replica_request_stays_an_any_replica_read",
     (('Any_', ()), False, None)),
    ("a_bounded_request_carries_the_exact_budget_it_asked_for",
     ((('Bounded', (250,)), False, 250), (('Bounded', (1,)), False, 1), (('Bounded', (0,)), False, 0))),
    ("surrounding_whitespace_is_trimmed_before_the_value_is_read",
     ((('Leader', ()), True, None), (('Any_', ()), False, None), (('Bounded', (5,)), False, 5))),
    ("the_value_is_matched_without_regard_to_case",
     ((('Leader', ()), True, None), (('Any_', ()), False, None), (('Bounded', (7,)), False, 7))),
    ("only_the_leader_mode_reports_itself_as_the_strongest",
     (True, False, False)),
    ("only_the_bounded_mode_reports_a_staleness_budget",
     (None, None, 9)),
    ("the_bounded_form_the_parser_accepts_is_built_from_the_published_constants",
     (('Bounded', (42,)), False, 42)),
    ("a_zero_budget_is_a_bounded_read_and_not_a_leader_read",
     ((('Bounded', (0,)), False, 0), False)),
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


def mode(value: object) -> object:
    """A parsed consistency mode as (record name, fields).

    `Leader` and `Any_` are both field-less records, so their plain views are
    both the empty tuple. Naming the record is what makes the two
    distinguishable, and confusing a leader read with an any-replica read is
    precisely the failure this case exists to catch.
    """
    return (type(value).__name__, plain(value))


def read(raw: str | None) -> object:
    """A header value reduced to everything a caller acts on.

    The record name says which mode was selected, and the two derived
    questions say what the rest of the system does with it: whether the read
    must be served by the leader, and how much staleness it tolerates. A
    parser can be wrong in any one of the three independently.
    """
    parsed = parse_read_consistency(raw)
    return (mode(parsed), is_strongest(parsed), tolerated_staleness_ms(parsed))


def reads(*raws: str | None) -> tuple[object, ...]:
    """Several header values read the same way, in the order given."""
    return tuple(read(raw) for raw in raws)


def selected(*raws: str | None) -> tuple[object, ...]:
    """Just the mode each header value selects, without the derived answers."""
    return tuple(mode(parse_read_consistency(raw)) for raw in raws)


def verify_read_consistency_header_contract_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the header a client sets and the bounded form it spells
    exp1 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((READ_CONSISTENCY_HEADER, BOUNDED_PREFIX,
        BOUNDED_SUFFIX))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. an absent header selects a leader read
    exp2 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[1][1]
    obs2 = plain(read(None))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an explicit leader request stays a leader read
    exp3 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[2][1]
    obs3 = plain(read("leader"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. an explicit any replica request stays an any replica read
    exp4 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[3][1]
    obs4 = plain(read("any"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a bounded request carries the exact budget it asked for
    exp5 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[4][1]
    obs5 = plain(reads("bounded(250)", "bounded(1)", "bounded(0)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. surrounding whitespace is trimmed before the value is read
    exp6 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[5][1]
    obs6 = plain(reads("  leader  ", " any", "bounded(5) "))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the value is matched without regard to case
    exp7 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[6][1]
    obs7 = plain(reads("LEADER", "Any", "BOUNDED(7)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. only the leader mode reports itself as the strongest
    exp8 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((is_strongest(LEADER), is_strongest(ANY),
        is_strongest(Bounded(max_staleness_ms=9))))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. only the bounded mode reports a staleness budget
    exp9 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((tolerated_staleness_ms(LEADER),
        tolerated_staleness_ms(ANY),
        tolerated_staleness_ms(Bounded(max_staleness_ms=9))))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the bounded form the parser accepts is built from the published constants
    exp10 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[9][1]
    obs10 = plain(read(BOUNDED_PREFIX + "42" + BOUNDED_SUFFIX))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a zero budget is a bounded read and not a leader read
    exp11 = READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((read("bounded(0)"), read("bounded(0)") ==
        read("leader")))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "read-consistency-header-contract-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
