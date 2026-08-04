from __future__ import annotations

from raft_runtime.application.cluster_topology import (
    ensure_static_membership_unchanged,
)
from raft_runtime.infrastructure.env import (
    ALL_KEYS,
    NODE_ID_KEY,
    PEER_OVERRIDES_KEY,
    POD_NAME_KEY,
    REPLICAS_PER_SHARD_KEY,
    SHARD_COUNT_KEY,
    VOTER_COUNT_KEY,
    parse_peer_overrides,
    read_int,
    replica_mode,
)

MINIMUM_CHECKS = 10

STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX = (
    ("a_non_numeric_dimension_is_refused_rather_than_quietly_defaulted",
     (None, None, 2)),
    ("a_non_numeric_replica_count_never_starts_cluster_mode",
     (False, False, False, False)),
    ("the_refusal_reports_the_current_count_before_the_desired_one",
     (('MembershipChanged', (2, 8)), ('MembershipChanged', (8, 2)))),
    ("a_shrink_and_a_growth_are_both_refused",
     (('MembershipChanged', (5, 4)), ('MembershipChanged', (4, 5)), ('MembershipChanged', (1, 0)), ('MembershipChanged', (0, 1)))),
    ("an_override_entry_that_is_only_whitespace_is_dropped",
     (('a', 'b'), (), ('a',))),
    ("a_separator_other_than_a_comma_does_not_split_the_list",
     (('a;b',), ('a b',), ('a|b',))),
    ("the_default_is_always_the_caller_s_and_never_a_silent_zero",
     (False, False, 3, 3)),
    ("a_signed_dimension_is_read_through_and_left_for_the_topology_to_refuse",
     (-3, 3, 0)),
    ("the_key_names_are_exactly_the_documented_ones_and_nothing_near_them",
     (5, 7, 7, 7)),
    ("every_admitted_form_is_still_admitted_after_all_of_these_refusals",
     (None, 12, True, ('x', 'y'))),
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


def named(problem: object) -> object:
    """A refusal as (record name, fields); an admission stays `None`.

    The fields alone are not enough: a refusal that transposed its two counts
    would still carry the same pair of numbers, so the row has to observe the
    order, and a refusal that lost its record name could not be told apart
    from any other two-integer refusal.
    """
    if problem is None:
        return None
    return (type(problem).__name__, plain(problem))


def lookup_of(values: dict[str, str]) -> object:
    """An environment lookup backed by a plain mapping."""
    def lookup(key: str) -> str | None:
        return values.get(key)
    return lookup


def empty() -> object:
    """An environment in which nothing at all is set."""
    return lookup_of({})


def guard(current: int, desired: int) -> object:
    """The static-membership verdict for one before/after replica count."""
    return named(ensure_static_membership_unchanged(current, desired))


def integer(value: str | None, default: int) -> object:
    """`read_int` over an environment holding exactly one key, or nothing."""
    values = {} if value is None else {NODE_ID_KEY: value}
    return read_int(lookup_of(values), NODE_ID_KEY, default)


def mode(value: str | None) -> object:
    """Whether an environment with this replica count is in cluster mode."""
    values = {} if value is None else {REPLICAS_PER_SHARD_KEY: value}
    return replica_mode(lookup_of(values))


def verify_static_membership_guard_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a non numeric dimension is refused rather than quietly defaulted
    exp1 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[0][1]
    obs1 = plain((integer("abc", 7), integer("1x", 7), integer("\u0662",
        7)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a non numeric replica count never starts cluster mode
    exp2 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[1][1]
    obs2 = plain((mode("abc"), mode(""), mode("0"), mode("-4")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the refusal reports the current count before the desired one
    exp3 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[2][1]
    obs3 = plain((guard(2, 8), guard(8, 2)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a shrink and a growth are both refused
    exp4 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[3][1]
    obs4 = plain((guard(5, 4), guard(4, 5), guard(1, 0), guard(0, 1)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an override entry that is only whitespace is dropped
    exp5 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[4][1]
    obs5 = plain((parse_peer_overrides("a, ,b"),
        parse_peer_overrides("  ,\t"), parse_peer_overrides("a,   ")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a separator other than a comma does not split the list
    exp6 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[5][1]
    obs6 = plain((parse_peer_overrides("a;b"),
        parse_peer_overrides("a b"), parse_peer_overrides("a|b")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the default is always the caller s and never a silent zero
    exp7 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[6][1]
    obs7 = plain((integer(None, 3) == 0, integer("", 3) == 0,
        integer(None, 3), integer("", 3)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a signed dimension is read through and left for the topology to refuse
    exp8 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[7][1]
    obs8 = plain((integer("-3", 7), integer("+3", 7), integer("-0", 7)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the key names are exactly the documented ones and nothing near them
    exp9 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[8][1]
    obs9 = plain((read_int(lookup_of({"NODE_ID": "5"}), NODE_ID_KEY, 7),
        read_int(lookup_of({"NODE-ID": "5"}), NODE_ID_KEY, 7),
        read_int(lookup_of({"node_id": "5"}), NODE_ID_KEY, 7),
        read_int(empty(), NODE_ID_KEY, 7)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. every admitted form is still admitted after all of these refusals
    exp10 = STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[9][1]
    obs10 = plain((guard(6, 6), integer("12", 0), mode("3"),
        parse_peer_overrides("x,y")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "static-membership-guard-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
