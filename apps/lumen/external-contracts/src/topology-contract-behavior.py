"""EC behavior case for #2937 — the 1x1-first public topology contract.

Every expected value below is an EC-owned literal transcribed from the issue's
requirements, never a symbol read back out of the design under test: R1 (one
shard, one voter by default), R3 (one or three voters, read replicas a separate
non-voting role), R5 (three voters is the HA option), and R7 (status separates
user policy, current topology, target topology, and generations, and never
reports an uncommitted render as converged).
"""

from __future__ import annotations

from lumen.topology.admission import decide_topology_spec
from lumen.topology.availability import availability_promise
from lumen.topology.spec import TopologySpec
from lumen.topology.status import TopologyStatus
from lumen.topology.verdict import AdmittedTopology, Rejection

MINIMUM_CHECKS = 15

TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX = (
    ("default_shard_minimum_is_one", 1),
    ("default_voter_count_is_one", 1),
    ("default_read_replica_count_is_zero", 0),
    ("default_spec_is_admitted", "admitted"),
    ("default_admission_preserves_the_1x1_topology", (1, 1, 0)),
    ("three_voters_is_admitted", "admitted"),
    ("admitted_shard_count_follows_shard_minimum", 4),
    ("read_replicas_are_excluded_from_the_voter_count", 3),
    ("admitted_topology_carries_the_read_replica_count", 5),
    ("status_separates_policy_current_target_and_generations", True),
    ("uncommitted_render_is_not_converged", False),
    ("committed_matching_status_is_converged", True),
    ("unequal_generations_are_not_converged", False),
    ("unequal_current_and_target_topologies_are_not_converged", False),
    ("three_voters_survive_one_unexpected_node_loss", "survives_one_unexpected_node_loss"),
)


def verify_topology_contract_behavior() -> dict:
    checks = []

    default_spec = TopologySpec.default()

    # 1. R1 — the default physical topology is one data shard.
    obs1 = default_spec.shard_minimum
    exp1 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 — with one Raft voter.
    obs2 = default_spec.voters
    exp2 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R3 — read replicas are opt-in, so the default carries none.
    obs3 = default_spec.read_replicas
    exp3 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R6 — the default is admissible, so fail-closed admission does not
    #    reject the shape the CRD hands every new user.
    verdict_default = decide_topology_spec(default_spec)
    obs4 = verdict_default.reason.value if isinstance(verdict_default, Rejection) else "admitted"
    exp4 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1 — admission preserves the canonical physical default rather than
    #    merely accepting the input and substituting another topology.
    obs5 = (
        (verdict_default.shard_count, verdict_default.voters, verdict_default.read_replicas)
        if isinstance(verdict_default, AdmittedTopology)
        else None
    )
    exp5 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R3 — three voters is the other user-facing availability selection.
    ha_spec = TopologySpec(shard_minimum=1, voters=3, read_replicas=0)
    verdict_ha = decide_topology_spec(ha_spec)
    obs5 = verdict_ha.reason.value if isinstance(verdict_ha, Rejection) else "admitted"
    exp6 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs5, "passed": obs5 == exp6})

    # 7. R4 — the user sets a minimum shard count and the admitted topology
    #    honours it rather than substituting a controller-chosen number.
    wide_spec = TopologySpec(shard_minimum=4, voters=3, read_replicas=0)
    admitted_wide = decide_topology_spec(wide_spec)
    obs7 = admitted_wide.shard_count if isinstance(admitted_wide, AdmittedTopology) else -1
    exp7 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 — a non-voting role. Five read replicas must not become voters,
    #    which is the reading that would silently turn a 3-voter quorum into an
    #    8-member one and change what a majority means.
    replica_spec = TopologySpec(shard_minimum=1, voters=3, read_replicas=5)
    admitted_replicas = decide_topology_spec(replica_spec)
    obs8 = admitted_replicas.voters if isinstance(admitted_replicas, AdmittedTopology) else -1
    exp8 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R3 — and they are still carried, not discarded, by the admitted shape.
    obs9 = admitted_replicas.read_replicas if isinstance(admitted_replicas, AdmittedTopology) else -1
    exp9 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R7 — the four status compartments are distinct named fields. A status
    #    that folds policy into current cannot express "asked for, not yet
    #    running", which is the state the whole epic turns on.
    status_fields = frozenset(TopologyStatus.__dataclass_fields__)
    obs10 = status_fields.issuperset(
        {"policy", "current", "target", "observed_generation", "converged_generation"}
    )
    exp10 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R7 — an uncommitted resource render is never converged serving state,
    #     even when both generations already agree.
    pending = TopologyStatus(
        policy=default_spec,
        current=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        target=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        observed_generation=7,
        converged_generation=7,
        render_committed=False,
    )
    obs11 = pending.is_converged()
    exp11 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7 — the full committed, generation-equal, topology-equal state is
    #     converged; this prevents an always-false implementation from passing.
    converged = TopologyStatus(
        policy=default_spec,
        current=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        target=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        observed_generation=7,
        converged_generation=7,
        render_committed=True,
    )
    obs12 = converged.is_converged()
    exp12 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 — a committed render with stale observed generation is not yet
    #     converged serving state.
    unequal_generations = TopologyStatus(
        policy=default_spec,
        current=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        target=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        observed_generation=6,
        converged_generation=7,
        render_committed=True,
    )
    obs13 = unequal_generations.is_converged()
    exp13 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R7 — a committed, generation-equal render remains unconverged while
    #     the admitted current and target topologies differ.
    unequal_topologies = TopologyStatus(
        policy=default_spec,
        current=AdmittedTopology(shard_count=1, voters=1, read_replicas=0),
        target=AdmittedTopology(shard_count=1, voters=3, read_replicas=0),
        observed_generation=7,
        converged_generation=7,
        render_committed=True,
    )
    obs14 = unequal_topologies.is_converged()
    exp14 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R5 — three voters is the availability option, and the design says so
    #     in its own vocabulary rather than leaving the promise to the README.
    obs15 = availability_promise(3)
    exp15 = TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": TOPOLOGY_CONTRACT_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "topology-contract-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
