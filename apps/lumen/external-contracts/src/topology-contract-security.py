"""EC security case for #2937 — fail-closed topology admission.

Expected values are EC-owned literals transcribed from the issue: R6 (admission
rejects zero counts, even voter counts, ambiguous legacy combinations, and
mutations for which no safe controller exists), R2 (one data member per node
across every Lumen instance in the cluster), R4 (users do not configure HPA/VPA
objects or thresholds), and R5 (a one-voter shard promises nothing under
unexpected node loss).

A rejection is only useful if it names where it happened, so the last row holds
every refusal in this case to a non-empty field path -- an admission that
refuses without saying which field was wrong is one a user cannot act on.
"""

from __future__ import annotations

from lumen.topology.admission import (
    decide_placement,
    decide_topology_mutation,
    decide_topology_spec,
)
from lumen.topology.availability import availability_promise
from lumen.topology.spec import TopologySpec
from lumen.topology.verdict import Rejection

MINIMUM_CHECKS = 15

TOPOLOGY_CONTRACT_SECURITY_MATRIX = (
    ("zero_shard_minimum_is_rejected", ("zero_shard_minimum", "shard_minimum")),
    ("zero_voters_is_rejected", ("zero_voters", "voters")),
    ("even_voter_count_is_rejected", ("even_voter_count", "voters")),
    ("five_voters_is_rejected_as_unsupported", ("unsupported_voter_count", "voters")),
    ("legacy_flattened_replica_field_is_rejected", ("legacy_replica_vocabulary", "legacy_replicas")),
    ("negative_read_replica_count_is_rejected", ("negative_read_replicas", "read_replicas")),
    ("hpa_knob_is_rejected_rather_than_honoured", ("hpa_knob_not_owned", "hpa_knobs")),
    ("one_to_three_voter_mutation_is_rejected_without_a_safe_controller", ("no_safe_topology_mutation", "voters")),
    ("two_instances_sharing_one_node_is_rejected", ("data_member_node_conflict", "placement.node_name")),
    ("two_instances_on_distinct_nodes_are_admitted", "admitted"),
    ("one_voter_makes_no_unexpected_loss_promise", "no_promise_on_unexpected_node_loss"),
    ("zero_voters_raise_value_error_for_availability", (0, "ValueError")),
    ("even_voters_raise_value_error_for_availability", (2, "ValueError")),
    ("unsupported_voters_raise_value_error_for_availability", (5, "ValueError")),
    ("every_refusal_names_a_non_empty_field_path", True),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def _rejection(verdict) -> tuple[str, str]:
    if isinstance(verdict, Rejection):
        return (verdict.reason.value, verdict.field_path)
    return ("admitted", "")


def _availability_outcome(voters: int) -> tuple[int, str]:
    try:
        availability_promise(voters)
    except ValueError as error:
        return (voters, type(error).__name__)
    return (voters, "returned")


def verify_topology_contract_security() -> dict:
    checks = []
    refusals = []

    # 1. R6 — zero shards is not "no shards", it is an unserviceable cluster.
    v1 = decide_topology_spec(TopologySpec(shard_minimum=0, voters=1, read_replicas=0))
    refusals.append(v1)
    obs1 = _rejection(v1)
    exp1 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[0][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R6 — zero voters is a shard that can never commit a write.
    v2 = decide_topology_spec(TopologySpec(shard_minimum=1, voters=0, read_replicas=0))
    refusals.append(v2)
    obs2 = _rejection(v2)
    exp2 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[1][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R6 — an even voter count has no majority and deadlocks on a tie.
    v3 = decide_topology_spec(TopologySpec(shard_minimum=1, voters=2, read_replicas=0))
    refusals.append(v3)
    obs3 = _rejection(v3)
    exp3 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[2][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R3/R6 — odd is necessary but not sufficient: the contract offers one
    #    or three, and five must be refused as unsupported rather than accepted
    #    because it happens to be odd.
    v4 = decide_topology_spec(TopologySpec(shard_minimum=1, voters=5, read_replicas=0))
    refusals.append(v4)
    obs4 = _rejection(v4)
    exp4 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[3][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R6 — the old flattened vocabulary is ambiguous next to voters and read
    #    replicas, so a CR still carrying it is refused, not reinterpreted.
    v5 = decide_topology_spec(
        TopologySpec(shard_minimum=1, voters=1, read_replicas=0, legacy_replicas=3)
    )
    refusals.append(v5)
    obs5 = _rejection(v5)
    exp5 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[4][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R6 — a negative count is a zero count that passed a `!= 0` test.
    v6 = decide_topology_spec(TopologySpec(shard_minimum=1, voters=1, read_replicas=-1))
    refusals.append(v6)
    obs6 = _rejection(v6)
    exp6 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[5][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 — Lumen does not own HPA/VPA thresholds. Silently ignoring the knob
    #    is worse than refusing it: the user keeps believing it is in effect.
    v7 = decide_topology_spec(
        TopologySpec(
            shard_minimum=1,
            voters=1,
            read_replicas=0,
            hpa_knobs=("targetCPUUtilizationPercentage",),
        )
    )
    refusals.append(v7)
    obs7 = _rejection(v7)
    exp7 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[6][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R6 — voter membership change is exactly the mutation this issue has no
    #    safe controller for; #3089 introduces the learner path that admits it.
    v8 = decide_topology_mutation(
        TopologySpec(shard_minimum=1, voters=1, read_replicas=0),
        TopologySpec(shard_minimum=1, voters=3, read_replicas=0),
    )
    refusals.append(v8)
    obs8 = _rejection(v8)
    exp8 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[7][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R2/AC3 — two *different* Lumen instances landing data members on one
    #    node invalidates the one-member-per-node failure domain just as surely
    #    as one instance doing it, and is the case a per-instance anti-affinity
    #    rule misses.
    v9 = decide_placement((("lumen-a", "node-1"), ("lumen-b", "node-1")))
    refusals.append(v9)
    obs9 = _rejection(v9)
    exp9 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[8][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R2/AC3 — distinct nodes preserve the one-member-per-node placement
    #     model, so a fail-closed placement decider must also admit this shape.
    v10 = decide_placement((("lumen-a", "node-1"), ("lumen-b", "node-2")))
    obs10 = _reason(v10)
    exp10 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[9][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R5/AC4 — the one-voter default must not be describable as highly
    #     available. This is the overclaim the issue exists to close.
    obs11 = availability_promise(1)
    exp11 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[10][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12-14. R5 — availability is only defined for admitted voter counts.
    obs12 = _availability_outcome(0)
    exp12 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[11][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    obs13 = _availability_outcome(2)
    exp13 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[12][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    obs14 = _availability_outcome(5)
    exp14 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[13][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. AC1 — every refusal above must land on a precise field, so a user can
    #     see which part of their spec was wrong.
    obs15 = all(isinstance(r, Rejection) and bool(r.field_path) for r in refusals)
    exp15 = TOPOLOGY_CONTRACT_SECURITY_MATRIX[14][1]
    checks.append({"name": TOPOLOGY_CONTRACT_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "topology-contract-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
