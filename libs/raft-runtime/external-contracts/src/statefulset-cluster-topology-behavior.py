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

STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX = (
    ("the_shard_index_is_the_ordinal_modulo_the_shard_count",
     (0, 1, 2, 0, 1)),
    ("the_replica_index_is_the_ordinal_divided_by_the_shard_count",
     (0, 0, 1, 1, 2)),
    ("a_replica_votes_exactly_while_its_replica_index_is_below_the_quorum",
     (True, True, False, False)),
    ("peer_ordinal_is_the_exact_inverse_of_the_shard_and_replica_split",
     (0, 5, 7, 4)),
    ("a_pod_name_splits_at_its_last_hyphen_into_a_name_and_an_ordinal",
     (('raft', 0), ('raft', 12), ('raft', 7))),
    ("a_statefulset_name_that_itself_contains_hyphens_survives_the_split",
     (('lumen-raft-node', 3), ('a-b-c-d', 11))),
    ("a_peer_host_is_the_pod_prefix_and_ordinal_under_the_governing_service",
     ('raft-0.raft-peers', 'lumen-raft-12.peers.ns.svc')),
    ("a_peer_url_carries_the_scheme_the_derived_host_and_the_port",
     ('http://raft-1.raft-peers:9000', 'https://raft-0.peers:443', None, None)),
    ("the_derived_peer_list_holds_one_entry_per_replica_in_this_shard",
     ((1, 'raft', 0, 1, True, ((0, 'http://raft-0.raft:9000'), (1, 'http://raft-1.raft:9000'), (2, 'http://raft-2.raft:9000'))), (1, 'raft', 1, 2, False, ((0, 'http://raft-1.raft:9000'), (1, 'http://raft-3.raft:9000'), (2, 'http://raft-5.raft:9000'))))),
    ("the_peer_dns_prefix_follows_the_pod_name_and_not_the_binary_name",
     ((0, 'deployed-raft', 0, 0, True, ((0, 'http://deployed-raft-0.raft:9000'), (1, 'http://deployed-raft-1.raft:9000'))), (1, 'deployed-raft', 0, 1, False, ((0, 'http://deployed-raft-0.raft:9000'), (1, 'http://deployed-raft-1.raft:9000'))))),
    ("an_absent_or_blank_pod_name_falls_back_to_the_callers_prefix",
     ((0, 'fallback', 0, 0, True, ((0, 'http://fallback-0.raft:9000'),)), (0, 'fallback', 0, 0, True, ((0, 'http://fallback-0.raft:9000'),)), (0, 'fallback', 0, 0, True, ((0, 'http://fallback-0.raft:9000'),)))),
    ("peer_overrides_are_split_trimmed_and_stripped_of_empty_entries",
     (('a', 'b'), ('a', 'b'), ('a', 'b'), (), (), ())),
    ("replica_mode_is_on_exactly_when_more_than_one_replica_is_declared",
     (False, False, True, False)),
    ("an_unset_environment_derives_a_single_node_shard_of_one",
     (0, 'solo', 0, 0, True, ((0, 'https://solo-0.peers:8443'),))),
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


def verify_statefulset_cluster_topology_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the shard index is the ordinal modulo the shard count
    exp1 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[0][1]
    def grid(ordinal: int) -> ClusterDims:
        return ClusterDims(shard_count=3, replicas_per_shard=3, voter_count=2, ordinal=ordinal)
    obs1 = plain((grid(0).shard_index, grid(1).shard_index,
        grid(2).shard_index, grid(3).shard_index, grid(7).shard_index))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the replica index is the ordinal divided by the shard count
    exp2 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[1][1]
    def grid(ordinal: int) -> ClusterDims:
        return ClusterDims(shard_count=3, replicas_per_shard=3, voter_count=2, ordinal=ordinal)
    obs2 = plain((grid(0).replica_index, grid(2).replica_index,
        grid(3).replica_index, grid(5).replica_index,
        grid(8).replica_index))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a replica votes exactly while its replica index is below the quorum
    exp3 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[2][1]
    def grid(ordinal: int) -> ClusterDims:
        return ClusterDims(shard_count=3, replicas_per_shard=3, voter_count=2, ordinal=ordinal)
    obs3 = plain((grid(0).is_voter, grid(3).is_voter, grid(6).is_voter,
        grid(8).is_voter))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. peer ordinal is the exact inverse of the shard and replica split
    exp4 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[3][1]
    def grid(ordinal: int) -> ClusterDims:
        return ClusterDims(shard_count=3, replicas_per_shard=3, voter_count=2, ordinal=ordinal)
    obs4 = plain((peer_ordinal(3, 0, 0), peer_ordinal(3, 2, 1),
        peer_ordinal(3, grid(7).shard_index, grid(7).replica_index),
        peer_ordinal(1, 0, 4)))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a pod name splits at its last hyphen into a name and an ordinal
    exp5 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[4][1]
    obs5 = plain((split_pod_name("raft-0"), split_pod_name("raft-12"),
        split_pod_name("raft-007")))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a statefulset name that itself contains hyphens survives the split
    exp6 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[5][1]
    obs6 = plain((split_pod_name("lumen-raft-node-3"),
        split_pod_name("a-b-c-d-11")))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. a peer host is the pod prefix and ordinal under the governing service
    exp7 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[6][1]
    obs7 = plain((peer_host("raft", 0, "raft-peers"),
        peer_host("lumen-raft", 12, "peers.ns.svc")))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a peer url carries the scheme the derived host and the port
    exp8 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[7][1]
    obs8 = plain((peer_url("http", "raft", 1, "raft-peers", 9000),
        peer_url("https", "raft", 0, "peers", 443),
        scheme_problem("http"), scheme_problem("https")))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the derived peer list holds one entry per replica in this shard
    exp9 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[8][1]
    obs9 = plain((summary(build({"SHARD_COUNT": "1", "REPLICAS_PER_SHARD":
        "3", "VOTER_COUNT": "3", "NODE_ID": "1", "POD_NAME": "raft-1"})),
        summary(build({"SHARD_COUNT": "2", "REPLICAS_PER_SHARD": "3",
        "VOTER_COUNT": "2", "NODE_ID": "1", "POD_NAME": "raft-5"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the peer dns prefix follows the pod name and not the binary name
    exp10 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[9][1]
    obs10 = plain((summary(build({"POD_NAME": "deployed-raft-0",
        "REPLICAS_PER_SHARD": "2", "VOTER_COUNT": "1"},
        prefix="my-binary")), summary(build({"POD_NAME":
        "deployed-raft-1", "REPLICAS_PER_SHARD": "2", "VOTER_COUNT": "1",
        "NODE_ID": "1"}, prefix="my-binary"))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. an absent or blank pod name falls back to the callers prefix
    exp11 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[10][1]
    obs11 = plain((summary(build({})), summary(build({"POD_NAME":
        "   "})), summary(build({"POD_NAME": "nameless"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. peer overrides are split trimmed and stripped of empty entries
    exp12 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[11][1]
    obs12 = plain((parse_peer_overrides("a,b"),
        parse_peer_overrides(" a , b "), parse_peer_overrides("a,,b,"),
        parse_peer_overrides(""), parse_peer_overrides(None),
        parse_peer_overrides(" , ")))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. replica mode is on exactly when more than one replica is declared
    exp13 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[12][1]
    obs13 = plain((replica_mode(lookup_of({})),
        replica_mode(lookup_of({"REPLICAS_PER_SHARD": "1"})),
        replica_mode(lookup_of({"REPLICAS_PER_SHARD": "2"})),
        replica_mode(lookup_of({"REPLICAS_PER_SHARD": "nine"}))))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. an unset environment derives a single node shard of one
    exp14 = STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[13][1]
    obs14 = plain(summary(build({}, prefix="solo", scheme="https",
        service="peers", port=8443)))
    checks.append({"name": STATEFULSET_CLUSTER_TOPOLOGY_BEHAVIOR_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "statefulset-cluster-topology-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
