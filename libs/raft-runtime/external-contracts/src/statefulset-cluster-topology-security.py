from __future__ import annotations

from raft_runtime.application.cluster_topology import (
    ClusterTopology,
    topology_from_env,
)
from raft_runtime.domain.topology import (
    ClusterDims,
    dims_problem,
    peer_ordinal,
)
from raft_runtime.infrastructure.env import (
    parse_peer_overrides,
    replica_mode,
)
from raft_runtime.infrastructure.peer_url import (
    peer_host,
    peer_url,
    scheme_problem,
)
from raft_runtime.infrastructure.pod_name import split_pod_name

MINIMUM_CHECKS = 14

STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX = (
    ("a_non_positive_shard_count_is_refused_and_named",
     (('NonPositiveDimension', ('SHARD_COUNT', 0)), ('NonPositiveDimension', ('SHARD_COUNT', -1)), None)),
    ("a_non_positive_replica_count_is_refused_and_named",
     (('NonPositiveDimension', ('REPLICAS_PER_SHARD', 0)), ('NonPositiveDimension', ('REPLICAS_PER_SHARD', -2)))),
    ("a_voter_count_outside_one_to_replicas_is_refused_with_both_numbers",
     (('VoterCountOutOfRange', (0, 3)), ('VoterCountOutOfRange', (4, 3)), None, None)),
    ("a_node_id_outside_the_replica_range_is_refused_with_both_numbers",
     (('NodeIdOutOfRange', (3, 3)), ('NodeIdOutOfRange', (-1, 3)), None)),
    ("the_dimension_checks_are_reported_in_one_fixed_order",
     (('NonPositiveDimension', ('SHARD_COUNT', 0)), ('NonPositiveDimension', ('REPLICAS_PER_SHARD', 0)), ('VoterCountOutOfRange', (9, 3)))),
    ("an_unknown_url_scheme_is_refused_and_the_supported_set_is_named",
     (('UnsupportedScheme', ('ftp', ('http', 'https'))), ('UnsupportedScheme', ('', ('http', 'https'))), ('UnsupportedScheme', ('HTTP', ('http', 'https'))), ('UnsupportedScheme', ('unix', ('http', 'https'))))),
    ("no_peer_url_is_ever_built_under_an_unknown_scheme",
     (('UnsupportedScheme', ('ftp', ('http', 'https'))), ('UnsupportedScheme', ('gopher', ('http', 'https'))))),
    ("a_pod_name_with_no_hyphen_is_nameless_rather_than_a_silent_prefix",
     (('NamelessPod', ('raft',)), ('NamelessPod', ('',)), ('NamelessPod', ('0',)))),
    ("an_empty_statefulset_prefix_is_nameless_too",
     (('NamelessPod', ('-0',)), ('NamelessPod', ('-',)))),
    ("a_suffix_that_is_not_wholly_numeric_is_refused_with_the_suffix",
     (('BadOrdinal', ('raft-x', 'x')), ('BadOrdinal', ('raft-1a', '1a')), ('BadOrdinal', ('raft-', '')), ('BadOrdinal', ('raft- 1', ' 1')))),
    ("the_scheme_is_refused_before_any_environment_value_is_consulted",
     (('UnsupportedScheme', ('ftp', ('http', 'https'))), ('NonPositiveDimension', ('SHARD_COUNT', 0)))),
    ("a_non_integer_environment_value_is_refused_and_names_its_key",
     (('NonPositiveDimension', ('SHARD_COUNT', 0)), ('NonPositiveDimension', ('REPLICAS_PER_SHARD', 0)), ('NonPositiveDimension', ('VOTER_COUNT', 0)), ('NonPositiveDimension', ('NODE_ID', 0)))),
    ("a_refused_configuration_never_yields_a_partly_built_topology",
     (False, True, ('NodeIdOutOfRange', (5, 1)))),
    ("the_accepted_boundaries_are_admitted_so_the_guard_is_not_a_blanket_no",
     (None, None, None, None)),
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


def lookup_of(values: dict[str, str]) -> object:
    """An environment lookup backed by a plain mapping."""

    def lookup(key: str) -> str | None:
        return values.get(key)

    return lookup


def build(
    values: dict[str, str],
    prefix: str = "fallback",
    scheme: str = "http",
    service: str = "raft",
    port: int = 9000,
) -> object:
    """Derive a topology from a mapping-backed environment."""
    return topology_from_env(
        lookup_of(values),
        fallback_prefix=prefix,
        scheme=scheme,
        service=service,
        port=port,
    )


def summary(result: object) -> object:
    """A derived topology reduced to what a replica actually acts on.

    A refusal is reported as its record name plus its fields, so a rejected
    configuration can never be mistaken for a topology with odd numbers in it.
    Each peer carries both the id it is registered under and the address it
    resolves to: a peer list is wrong if either half is wrong, and the two
    are computed from different arithmetic.
    """
    if isinstance(result, ClusterTopology):
        return (
            result.node_id,
            result.prefix,
            result.dims.shard_index,
            result.dims.replica_index,
            result.dims.is_voter,
            tuple((peer.node_id, peer.url) for peer in result.peers),
        )
    return (type(result).__name__, plain(result))


def named(problem: object) -> object:
    """A domain refusal as (record name, fields); `None` stays `None`."""
    if problem is None:
        return None
    return (type(problem).__name__, plain(problem))


def verify_statefulset_cluster_topology_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a non positive shard count is refused and named
    exp1 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[0][1]
    obs1 = plain((named(dims_problem(0, 3, 1, 0)), named(dims_problem(-1,
        3, 1, 0)), named(dims_problem(1, 3, 1, 0))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a non positive replica count is refused and named
    exp2 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[1][1]
    obs2 = plain((named(dims_problem(1, 0, 1, 0)), named(dims_problem(1,
        -2, 1, 0))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a voter count outside one to replicas is refused with both numbers
    exp3 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[2][1]
    obs3 = plain((named(dims_problem(1, 3, 0, 0)), named(dims_problem(1,
        3, 4, 0)), named(dims_problem(1, 3, 3, 0)), named(dims_problem(1,
        3, 1, 0))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a node id outside the replica range is refused with both numbers
    exp4 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[3][1]
    obs4 = plain((named(dims_problem(1, 3, 1, 3)), named(dims_problem(1,
        3, 1, -1)), named(dims_problem(1, 3, 1, 2))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the dimension checks are reported in one fixed order
    exp5 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[4][1]
    obs5 = plain((named(dims_problem(0, 0, 9, 9)), named(dims_problem(1,
        0, 9, 9)), named(dims_problem(1, 3, 9, 9))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. an unknown url scheme is refused and the supported set is named
    exp6 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[5][1]
    obs6 = plain((named(scheme_problem("ftp")), named(scheme_problem("")),
        named(scheme_problem("HTTP")), named(scheme_problem("unix"))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. no peer url is ever built under an unknown scheme
    exp7 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[6][1]
    obs7 = plain((named(peer_url("ftp", "raft", 0, "peers", 9000)),
        named(peer_url("gopher", "raft", 0, "peers", 9000))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a pod name with no hyphen is nameless rather than a silent prefix
    exp8 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[7][1]
    obs8 = plain((named(split_pod_name("raft")),
        named(split_pod_name("")), named(split_pod_name("0"))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. an empty statefulset prefix is nameless too
    exp9 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[8][1]
    obs9 = plain((named(split_pod_name("-0")),
        named(split_pod_name("-"))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a suffix that is not wholly numeric is refused with the suffix
    exp10 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[9][1]
    obs10 = plain((named(split_pod_name("raft-x")),
        named(split_pod_name("raft-1a")), named(split_pod_name("raft-")),
        named(split_pod_name("raft- 1"))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the scheme is refused before any environment value is consulted
    exp11 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[10][1]
    obs11 = plain((summary(build({"SHARD_COUNT": "0"}, scheme="ftp")),
        summary(build({"SHARD_COUNT": "0"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a non integer environment value is refused and names its key
    exp12 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[11][1]
    obs12 = plain((summary(build({"SHARD_COUNT": "many"})),
        summary(build({"REPLICAS_PER_SHARD": "two"})),
        summary(build({"VOTER_COUNT": "all"})), summary(build({"NODE_ID":
        "me"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. a refused configuration never yields a partly built topology
    exp13 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[12][1]
    obs13 = plain((isinstance(build({"NODE_ID": "5"}), ClusterTopology),
        isinstance(build({}), ClusterTopology), summary(build({"NODE_ID":
        "5"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. the accepted boundaries are admitted so the guard is not a blanket no
    exp14 = STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[13][1]
    obs14 = plain((named(dims_problem(1, 1, 1, 0)), named(dims_problem(9,
        5, 5, 4)), named(dims_problem(1, 3, 1, 0)), named(dims_problem(1,
        3, 3, 2))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "statefulset-cluster-topology-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
