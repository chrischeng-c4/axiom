from __future__ import annotations

from raft_runtime.application.host_config import (
    DEFAULT_PROPOSE_TIMEOUT_MS,
    DEFAULT_PUMP_MS,
    DEFAULT_RPC_TIMEOUT_MS,
    DEFAULT_TICK_MS,
    PROPOSE_RETRY_MS,
    HostConfig,
    compact_upto,
    drain_budget_ms,
    propose_attempts,
)
from raft_runtime.domain.snapshot import (
    DEFAULT_SNAPSHOT_POLICY,
    Disabled,
    EveryEntries,
    External,
    compactable_upto,
    should_snapshot,
)
from raft_runtime.infrastructure.routes import (
    APPEND_ENTRIES_PATH,
    CONSENSUS_PATHS,
    INSTALL_SNAPSHOT_PATH,
    PEER_PATHS,
    PUBLISH_PATH,
    RAFTZ_PATH,
    REQUEST_VOTE_PATH,
    is_consensus_path,
    is_peer_path,
    requires_peer_identity,
)

MINIMUM_CHECKS = 10

SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX = (
    ("a_host_that_has_applied_nothing_never_produces_a_compaction_point",
     (0, 0, 0)),
    ("a_non_positive_interval_never_becomes_a_snapshot_on_every_entry",
     ((False, 0), (False, 0))),
    ("a_disabled_policy_is_never_overridden_by_a_large_backlog",
     ((False, 0), (False, 0))),
    ("the_interval_boundary_is_reached_but_never_anticipated",
     (False, True, True, False)),
    ("compaction_never_reaches_past_the_applied_floor",
     (True, 7, True)),
    ("the_forwarded_proposal_route_is_peer_traffic_but_not_consensus_traffic",
     (True, False, True, False, True)),
    ("a_path_nobody_published_is_neither_peer_nor_consensus_traffic",
     (False, False, False, False, False)),
    ("the_retry_count_is_a_quotient_and_never_a_remainder",
     (5, 6, 1)),
    ("the_drain_window_never_collapses_below_the_rpc_timeout_it_covers",
     (True, True, 6)),
    ("every_configuration_that_should_compact_still_does_after_all_of_these",
     ((True, 8), (20, 5, 50, 10000, ('EveryEntries', (4,))))),
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


def named(policy: object) -> object:
    """A snapshot policy as (record name, fields).

    `Disabled` and `External` are both field-less records, so their plain
    views are both the empty tuple. A host that quietly swapped one for the
    other would be indistinguishable without the record name — and that swap
    is exactly the difference between never compacting and never compacting
    *on our own*.
    """
    return (type(policy).__name__, plain(policy))


def every(interval: int) -> object:
    """An interval policy that snapshots every `interval` applied entries."""
    return EveryEntries(interval=interval)


def host(policy: object = None, **overrides: int) -> HostConfig:
    """A host configuration, defaulted except where a row overrides it."""
    if policy is None:
        return HostConfig(**overrides)
    return HostConfig(snapshot_policy=policy, **overrides)


def settings(config: HostConfig) -> object:
    """A host configuration reduced to its numbers plus its named policy."""
    return (config.tick_ms, config.pump_ms, config.rpc_timeout_ms,
            config.propose_timeout_ms, named(config.snapshot_policy))


def fires(policy: object, applied: int, last: int) -> object:
    """Whether a policy asks for a snapshot, and what it lets us compact."""
    return (should_snapshot(policy, applied, last),
            compact_upto(host(policy), applied, last))


def verify_snapshot_policy_and_log_compaction_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a host that has applied nothing never produces a compaction point
    exp1 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[0][1]
    obs1 = plain((compact_upto(host(every(1)), 0, 0),
        compact_upto(host(External()), 0, 0), compact_upto(host(), 0, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a non positive interval never becomes a snapshot on every entry
    exp2 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[1][1]
    obs2 = plain((fires(every(0), 100, 0), fires(every(-1), 100, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a disabled policy is never overridden by a large backlog
    exp3 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[2][1]
    obs3 = plain((fires(Disabled(), 1000000, 0),
        fires(DEFAULT_SNAPSHOT_POLICY, 5, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the interval boundary is reached but never anticipated
    exp4 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[3][1]
    obs4 = plain((should_snapshot(every(5), 4, 0),
        should_snapshot(every(5), 5, 0), should_snapshot(every(5), 6, 0),
        should_snapshot(every(5), 0, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. compaction never reaches past the applied floor
    exp5 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[4][1]
    obs5 = plain((compactable_upto(7) == 7, compact_upto(host(every(1)),
        7, 6), compact_upto(host(every(1)), 7, 6) <= 7))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the forwarded proposal route is peer traffic but not consensus traffic
    exp6 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[5][1]
    obs6 = plain((is_peer_path(PUBLISH_PATH),
        is_consensus_path(PUBLISH_PATH),
        requires_peer_identity(PUBLISH_PATH), PUBLISH_PATH in
        CONSENSUS_PATHS, PUBLISH_PATH in PEER_PATHS))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a path nobody published is neither peer nor consensus traffic
    exp7 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[6][1]
    obs7 = plain((is_peer_path("/raft"), is_peer_path("/raft/"),
        is_consensus_path(RAFTZ_PATH),
        requires_peer_identity("/anything"), is_peer_path("")))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the retry count is a quotient and never a remainder
    exp8 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[7][1]
    obs8 = plain((propose_attempts(host(propose_timeout_ms=100)),
        propose_attempts(host(propose_timeout_ms=120)),
        propose_attempts(host(propose_timeout_ms=20))))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the drain window never collapses below the rpc timeout it covers
    exp9 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[8][1]
    obs9 = plain((drain_budget_ms(host()) == 2 * DEFAULT_RPC_TIMEOUT_MS,
        drain_budget_ms(host(rpc_timeout_ms=1)) > 1,
        drain_budget_ms(host(rpc_timeout_ms=3))))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. every configuration that should compact still does after all of these
    exp10 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[9][1]
    obs10 = plain((fires(every(4), 8, 4), settings(host(every(4),
        rpc_timeout_ms=50))))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "snapshot-policy-and-log-compaction-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
