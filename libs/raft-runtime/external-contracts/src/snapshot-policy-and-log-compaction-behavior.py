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

MINIMUM_CHECKS = 12

SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX = (
    ("a_host_that_was_told_nothing_does_not_snapshot_on_its_own",
     (('Disabled', ()), ('Disabled', ()))),
    ("an_interval_policy_fires_once_the_gap_reaches_the_interval",
     (False, True, True, False)),
    ("the_policies_that_are_not_interval_policies_never_fire",
     (False, False, False)),
    ("the_compactable_point_is_exactly_the_applied_floor",
     (0, 1, 42, 999)),
    ("the_documented_timing_defaults_a_host_starts_from",
     (20, 5, 400, 10000, 20, (20, 5, 400, 10000, ('Disabled', ())))),
    ("the_shutdown_drain_window_is_twice_the_rpc_timeout",
     (800, 300, 2)),
    ("the_retry_count_is_the_propose_budget_divided_by_the_retry_interval",
     (500, 2, 0)),
    ("a_policy_that_fires_yields_the_applied_index_as_the_compaction_point",
     (10, 25, 1, 7)),
    ("a_policy_that_does_not_fire_yields_nothing_to_compact",
     (0, 0, 0)),
    ("the_paths_this_runtime_publishes",
     ('/raft/request-vote', '/raft/append-entries', '/raft/install-snapshot', '/raft/publish', '/raftz')),
    ("the_consensus_set_and_the_wider_peer_set",
     (('/raft/request-vote', '/raft/append-entries', '/raft/install-snapshot'), ('/raft/request-vote', '/raft/append-entries', '/raft/install-snapshot', '/raft/publish'), 3, 4)),
    ("a_peer_identity_is_demanded_for_exactly_the_peer_paths",
     (True, True, True, True, False)),
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


def verify_snapshot_policy_and_log_compaction_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a host that was told nothing does not snapshot on its own
    exp1 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[0][1]
    obs1 = plain((named(DEFAULT_SNAPSHOT_POLICY),
        named(host().snapshot_policy)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. an interval policy fires once the gap reaches the interval
    exp2 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[1][1]
    obs2 = plain((should_snapshot(every(10), 9, 0),
        should_snapshot(every(10), 10, 0), should_snapshot(every(10), 25,
        10), should_snapshot(every(10), 14, 10)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the policies that are not interval policies never fire
    exp3 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[2][1]
    obs3 = plain((should_snapshot(Disabled(), 1000, 0),
        should_snapshot(External(), 1000, 0),
        should_snapshot(DEFAULT_SNAPSHOT_POLICY, 1000000, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. the compactable point is exactly the applied floor
    exp4 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[3][1]
    obs4 = plain((compactable_upto(0), compactable_upto(1),
        compactable_upto(42), compactable_upto(999)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the documented timing defaults a host starts from
    exp5 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((DEFAULT_TICK_MS, DEFAULT_PUMP_MS,
        DEFAULT_RPC_TIMEOUT_MS, DEFAULT_PROPOSE_TIMEOUT_MS,
        PROPOSE_RETRY_MS, settings(host())))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the shutdown drain window is twice the rpc timeout
    exp6 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[5][1]
    obs6 = plain((drain_budget_ms(host()),
        drain_budget_ms(host(rpc_timeout_ms=150)),
        drain_budget_ms(host(rpc_timeout_ms=1))))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. the retry count is the propose budget divided by the retry interval
    exp7 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((propose_attempts(host()),
        propose_attempts(host(propose_timeout_ms=45)),
        propose_attempts(host(propose_timeout_ms=19))))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a policy that fires yields the applied index as the compaction point
    exp8 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((compact_upto(host(every(10)), 10, 0),
        compact_upto(host(every(10)), 25, 10),
        compact_upto(host(every(1)), 1, 0), compact_upto(host(every(1)),
        7, 6)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a policy that does not fire yields nothing to compact
    exp9 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((compact_upto(host(), 100, 0),
        compact_upto(host(every(10)), 5, 0),
        compact_upto(host(External()), 100, 0)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the paths this runtime publishes
    exp10 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((REQUEST_VOTE_PATH, APPEND_ENTRIES_PATH,
        INSTALL_SNAPSHOT_PATH, PUBLISH_PATH, RAFTZ_PATH))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the consensus set and the wider peer set
    exp11 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((CONSENSUS_PATHS, PEER_PATHS, len(CONSENSUS_PATHS),
        len(PEER_PATHS)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a peer identity is demanded for exactly the peer paths
    exp12 = SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((requires_peer_identity(REQUEST_VOTE_PATH),
        requires_peer_identity(APPEND_ENTRIES_PATH),
        requires_peer_identity(INSTALL_SNAPSHOT_PATH),
        requires_peer_identity(PUBLISH_PATH),
        requires_peer_identity(RAFTZ_PATH)))
    checks.append({"name": SNAPSHOT_POLICY_AND_LOG_COMPACTION_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "snapshot-policy-and-log-compaction-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
