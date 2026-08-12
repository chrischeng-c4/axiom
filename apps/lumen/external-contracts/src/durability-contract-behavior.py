"""EC behavior case for #2938 -- durable Raft admission.

Every expected value below is an EC-owned literal transcribed from #2938:
R1 requires the ``raft`` backend and a deterministic named group even for one
voter; R2 requires ``durable_raft_commit_apply`` for every listed mutation;
R3 admits Raft in production; R5 preserves group identity and durable format
through membership extension; and R7 reports a structured
``durability-unavailable`` failure with degraded, unready status.
"""

from __future__ import annotations

from lumen.topology.durability_admission import (
    decide_membership_extension,
    decide_mutation_route,
    decide_new_shard_durability,
    decide_profile_durability,
    map_persistence_failure,
)
from lumen.topology.durability_spec import (
    DurabilityProfile,
    GroupIdentity,
    LegacyState,
    MembershipTarget,
    PersistenceFailure,
    ProfileClass,
    ShardDurabilityIntent,
)
from lumen.topology.durability_verdict import AdmittedDurability, Rejection

MINIMUM_CHECKS = 12

DURABILITY_CONTRACT_BEHAVIOR_MATRIX = (
    ("production_one_voter_uses_raft", "raft"),
    ("production_one_voter_has_a_named_group", True),
    ("group_name_is_deterministic", True),
    ("index_mutations_use_durable_raft_commit_apply", "durable_raft_commit_apply"),
    ("delete_mutations_use_durable_raft_commit_apply", "durable_raft_commit_apply"),
    ("schema_mutations_use_durable_raft_commit_apply", "durable_raft_commit_apply"),
    ("admin_mutations_use_durable_raft_commit_apply", "durable_raft_commit_apply"),
    ("production_raft_profile_is_admitted", "admitted"),
    ("learner_extension_preserves_group_name", "lumen-orders-0"),
    ("voter_extension_preserves_durable_format", "raft-runtime-v1"),
    ("persistence_failure_returns_durability_unavailable", "durability-unavailable"),
    ("persistence_failure_marks_status_degraded", "degraded"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_durability_contract_behavior() -> dict:
    checks = []
    production_one_voter = ShardDurabilityIntent(
        profile=ProfileClass.PRODUCTION,
        shard_name="orders",
        shard_ordinal=0,
        voters=1,
    )
    plan = decide_new_shard_durability(production_one_voter)

    # 1. R1 -- production's one-voter case still selects Raft.
    obs1 = plan.backend if isinstance(plan, AdmittedDurability) else _outcome(plan)
    exp1 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- its group has an operator-visible name rather than an anonymous
    #    single-replica fallback.
    obs2 = bool(plan.group_name) if isinstance(plan, AdmittedDurability) else False
    exp2 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- repeating an identical shard intent cannot mint a new group.
    repeat_plan = decide_new_shard_durability(production_one_voter)
    obs3 = (plan.group_name == repeat_plan.group_name) if isinstance(plan, AdmittedDurability) and isinstance(repeat_plan, AdmittedDurability) else False
    exp3 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- index mutation follows the durable commit-and-apply path.
    obs4 = decide_mutation_route(plan, "index")
    exp4 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R2 -- delete is also a state mutation, never an embedded side path.
    obs5 = decide_mutation_route(plan, "delete")
    exp5 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- schema changes must cross the same durable boundary.
    obs6 = decide_mutation_route(plan, "schema")
    exp6 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- privileged admin writes do not escape the durable route.
    obs7 = decide_mutation_route(plan, "admin")
    exp7 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- Raft remains an admitted production selection.
    raft_profile = decide_profile_durability(
        DurabilityProfile(profile=ProfileClass.PRODUCTION, backend="raft"),
        LegacyState.absent(),
    )
    obs8 = _outcome(raft_profile)
    exp8 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    group = GroupIdentity(group_name="lumen-orders-0", durable_format="raft-runtime-v1", voters=1)

    # 9. R5 -- learner growth retains the existing durable group identity.
    learner_extension = decide_membership_extension(group, MembershipTarget(learners=("lumen-orders-1",), voters=1))
    obs9 = learner_extension.group_name if isinstance(learner_extension, AdmittedDurability) else _outcome(learner_extension)
    exp9 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R5 -- voter growth likewise keeps the original on-disk format.
    voter_extension = decide_membership_extension(group, MembershipTarget(learners=(), voters=3))
    obs10 = voter_extension.durable_format if isinstance(voter_extension, AdmittedDurability) else _outcome(voter_extension)
    exp10 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    failure = map_persistence_failure(PersistenceFailure(kind="io_error", detail="EIO"))

    # 11. R7 -- a typed persistence error becomes a structured unavailable result.
    obs11 = failure.write_result
    exp11 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R7 -- independently, status is degraded rather than ready.
    obs12 = failure.status
    exp12 = DURABILITY_CONTRACT_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": DURABILITY_CONTRACT_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "durability-contract-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
