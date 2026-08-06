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

STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX = (
    ("the_environment_keys_a_deployment_is_expected_to_set",
     ('POD_NAME', 'SHARD_COUNT', 'REPLICAS_PER_SHARD', 'VOTER_COUNT', 'NODE_ID', 'PEER_OVERRIDES')),
    ("every_documented_key_is_listed_once_in_the_published_order",
     (('POD_NAME', 'SHARD_COUNT', 'REPLICAS_PER_SHARD', 'VOTER_COUNT', 'NODE_ID', 'PEER_OVERRIDES'), 6, 6)),
    ("an_unchanged_replica_count_is_admitted",
     (None, None, None)),
    ("a_changed_replica_count_is_refused_and_names_both_numbers",
     ('MembershipChanged', (3, 5))),
    ("an_unset_dimension_takes_the_default_the_caller_asked_for",
     (7, 1, 0)),
    ("a_blank_dimension_takes_the_default_the_caller_asked_for",
     (7, 7, 4)),
    ("a_dimension_that_is_present_is_read_rather_than_defaulted",
     (5, 5, 0)),
    ("a_peer_override_list_splits_on_commas_trims_and_drops_empties",
     (('a', 'b', 'c'), ('a', 'b'), ('a', 'b'))),
    ("an_unset_or_empty_override_list_yields_no_peers_at_all",
     ((), (), (), ())),
    ("cluster_mode_begins_at_the_second_replica_and_not_before",
     (False, False, True, True)),
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


def verify_static_membership_guard_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the environment keys a deployment is expected to set
    exp1 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((POD_NAME_KEY, SHARD_COUNT_KEY, REPLICAS_PER_SHARD_KEY,
        VOTER_COUNT_KEY, NODE_ID_KEY, PEER_OVERRIDES_KEY))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. every documented key is listed once in the published order
    exp2 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((ALL_KEYS, len(ALL_KEYS), len(set(ALL_KEYS))))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. an unchanged replica count is admitted
    exp3 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((guard(3, 3), guard(1, 1), guard(0, 0)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a changed replica count is refused and names both numbers
    exp4 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[3][1]
    obs4 = plain(guard(3, 5))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an unset dimension takes the default the caller asked for
    exp5 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((integer(None, 7), integer(None, 1), integer(None, 0)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a blank dimension takes the default the caller asked for
    exp6 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[5][1]
    obs6 = plain((integer("", 7), integer("   ", 7), integer("\t", 4)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a dimension that is present is read rather than defaulted
    exp7 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((integer("5", 7), integer(" 5 ", 7), integer("0", 7)))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a peer override list splits on commas trims and drops empties
    exp8 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((parse_peer_overrides("a,b,c"),
        parse_peer_overrides(" a , b "), parse_peer_overrides("a,,b")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an unset or empty override list yields no peers at all
    exp9 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((parse_peer_overrides(None), parse_peer_overrides(""),
        parse_peer_overrides(","), parse_peer_overrides(" , ")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. cluster mode begins at the second replica and not before
    exp10 = STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((mode(None), mode("1"), mode("2"), mode("9")))
    checks.append({"name": STATIC_MEMBERSHIP_GUARD_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "static-membership-guard-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
