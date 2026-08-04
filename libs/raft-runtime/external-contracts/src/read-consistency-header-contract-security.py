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

MINIMUM_CHECKS = 12

READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX = (
    ("an_unrecognized_value_falls_back_to_the_strongest_mode",
     ((('Leader', ()), True, None), (('Leader', ()), True, None), (('Leader', ()), True, None))),
    ("an_absent_header_is_a_leader_read_and_never_an_any_replica_read",
     ((('Leader', ()),), False, True)),
    ("an_empty_bounded_argument_never_reaches_the_integer_conversion",
     ((('Leader', ()), True, None), True)),
    ("a_bounded_argument_that_is_only_partly_numeric_is_refused",
     (('Leader', ()), ('Leader', ()), ('Leader', ()), ('Leader', ()))),
    ("a_signed_bounded_argument_is_refused",
     (('Leader', ()), ('Leader', ()))),
    ("a_digit_outside_ascii_is_refused_rather_than_converted",
     (('Leader', ()), ('Leader', ()))),
    ("an_unbalanced_or_doubled_parenthesis_is_refused",
     (('Leader', ()), ('Leader', ()), ('Leader', ()), ('Leader', ()))),
    ("whitespace_inside_the_bounded_argument_is_refused",
     (('Leader', ()), ('Leader', ()), ('Leader', ()))),
    ("the_budget_is_never_inflated_or_deflated_by_the_parser",
     (1, 2, 999999)),
    ("a_header_that_is_only_whitespace_is_a_leader_read",
     ((('Leader', ()), True, None), (('Leader', ()), True, None), (('Leader', ()), True, None))),
    ("a_leading_or_trailing_word_around_a_known_value_is_refused",
     (('Leader', ()), ('Leader', ()), ('Leader', ()), ('Leader', ()))),
    ("every_accepted_form_is_still_admitted_after_all_of_these_refusals",
     (('Leader', ()), ('Any_', ()), ('Bounded', (3,)), ('Leader', ()))),
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


def verify_read_consistency_header_contract_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. an unrecognized value falls back to the strongest mode
    exp1 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[0][1]
    obs1 = plain(reads("gibberish", "", "bounded"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. an absent header is a leader read and never an any replica read
    exp2 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[1][1]
    obs2 = plain((selected(None), selected(None) == selected("any"),
        selected(None) == selected("leader")))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an empty bounded argument never reaches the integer conversion
    exp3 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[2][1]
    obs3 = plain((read("bounded()"), selected("bounded()") ==
        selected("leader")))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a bounded argument that is only partly numeric is refused
    exp4 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[3][1]
    obs4 = plain(selected("bounded(1a)", "bounded(a1)", "bounded(1 2)",
        "bounded(1.5)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a signed bounded argument is refused
    exp5 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[4][1]
    obs5 = plain(selected("bounded(+5)", "bounded(-5)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a digit outside ascii is refused rather than converted
    exp6 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[5][1]
    obs6 = plain(selected("bounded(\u0662)", "bounded(\uff15)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. an unbalanced or doubled parenthesis is refused
    exp7 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[6][1]
    obs7 = plain(selected("bounded(5))", "bounded((5)", "bounded(5",
        "5)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. whitespace inside the bounded argument is refused
    exp8 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[7][1]
    obs8 = plain(selected("bounded( 5 )", "bounded(5 )", "bounded( 5)"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the budget is never inflated or deflated by the parser
    exp9 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[8][1]
    obs9 = plain((tolerated_staleness_ms(parse_read_consistency("bounded(1)")), tolerated_staleness_ms(parse_read_consistency("bounded(2)")), tolerated_staleness_ms(parse_read_consistency("bounded(999999)"))))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a header that is only whitespace is a leader read
    exp10 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[9][1]
    obs10 = plain(reads(" ", "\t", "\n"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. a leading or trailing word around a known value is refused
    exp11 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[10][1]
    obs11 = plain(selected("leaderx", "xleader", "any any", "leader,any"))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. every accepted form is still admitted after all of these refusals
    exp12 = READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[11][1]
    obs12 = plain(selected("leader", "any", "bounded(3)", None))
    checks.append({"name": READ_CONSISTENCY_HEADER_CONTRACT_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "read-consistency-header-contract-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
