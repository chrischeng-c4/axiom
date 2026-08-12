"""EC behavior case for #2951 -- policy-driven shard-split classification.

Every expected literal is transcribed from #2951 R1/R2/R3/R5 and AC1/AC2.
The case drives only the proposed pure capacity model: workload collection,
actuator invocation, Kubernetes reconciliation, and Terraform rendering remain
runtime-stage concerns.
"""

from __future__ import annotations

from lumen.capacity.selection import select_split_target
from lumen.capacity.trigger import decide_split_request
from lumen.capacity.verdict import (
    CapacityAction,
    SplitHeadroomInput,
    SplitSelectionInput,
    SplitTriggerInput,
)

MINIMUM_CHECKS = 14

CAPACITY_2951_BEHAVIOR_MATRIX = (
    ("user_minimum_independently_selects_a_split_request", "split_request"),
    ("user_minimum_split_names_its_reason", "user shard minimum"),
    ("pvc_growth_impossible_independently_selects_a_split_request", "split_request"),
    ("write_saturation_at_maximum_useful_machine_selects_a_split_request", "split_request"),
    ("compaction_saturation_at_maximum_useful_machine_selects_a_split_request", "split_request"),
    ("recovery_time_above_policy_selects_a_split_request", "split_request"),
    ("all_clear_capacity_selects_no_action_and_has_no_split_request", ("no_action", None)),
    ("read_only_pressure_selects_read_replicas", "read_replica"),
    ("read_only_pressure_has_no_shard_split_request", None),
    ("hottest_eligible_shard_is_selected", "shard-b"),
    ("selection_carries_exactly_the_next_generation", 18),
    ("selection_carries_a_different_next_generation", 41),
    ("equal_hottest_shards_use_the_deterministic_tie_break", "shard-a"),
    ("v1_decision_vocabulary_has_no_merge_action", ("no_action", "read_replica", "split_request", "capacity_blocked")),
)


def verify_capacity_2951_behavior() -> dict:
    checks = []

    # 1-2. R1/AC1 -- an explicitly supplied user minimum independently starts
    # a split request and identifies that policy reason.
    user_minimum = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=3,
            pvc_growth_possible=True,
            write_saturated=False,
            compaction_saturated=False,
            at_maximum_useful_machine=False,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs1 = user_minimum.action.value
    exp1 = CAPACITY_2951_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    obs2 = user_minimum.reason
    exp2 = CAPACITY_2951_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- PVC growth impossibility is a separate durable-ceiling input.
    pvc_ceiling = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=False,
            write_saturated=False,
            compaction_saturated=False,
            at_maximum_useful_machine=False,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs3 = pvc_ceiling.action.value
    exp3 = CAPACITY_2951_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- write saturation becomes a split only when the useful-machine
    # ceiling is explicitly present.
    write_ceiling = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=True,
            write_saturated=True,
            compaction_saturated=False,
            at_maximum_useful_machine=True,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs4 = write_ceiling.action.value
    exp4 = CAPACITY_2951_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1 -- compaction saturation has the same independent ceiling path.
    compaction_ceiling = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=True,
            write_saturated=False,
            compaction_saturated=True,
            at_maximum_useful_machine=True,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs5 = compaction_ceiling.action.value
    exp5 = CAPACITY_2951_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R1 -- recovery policy independently supplies the other durable ceiling.
    recovery_ceiling = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=True,
            write_saturated=False,
            compaction_saturated=False,
            at_maximum_useful_machine=False,
            measured_recovery_seconds=61,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs6 = recovery_ceiling.action.value
    exp6 = CAPACITY_2951_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R1 -- when neither a user minimum nor a durable ceiling applies, no
    # split request is proposed.
    all_clear = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=True,
            write_saturated=False,
            compaction_saturated=False,
            at_maximum_useful_machine=False,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=False,
            read_replicas_eligible=False,
        )
    )
    obs7 = (all_clear.action.value, all_clear.split_request)
    exp7 = CAPACITY_2951_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8-9. R2/AC2 -- eligible read replicas service read-only pressure, not a
    # shard split request.
    reads = decide_split_request(
        SplitTriggerInput(
            current_shard_count=2,
            requested_shard_minimum=2,
            pvc_growth_possible=True,
            write_saturated=False,
            compaction_saturated=False,
            at_maximum_useful_machine=False,
            measured_recovery_seconds=10,
            recovery_policy_seconds=60,
            aggregate_read_pressure=True,
            read_replicas_eligible=True,
        )
    )
    obs8 = reads.action.value
    exp8 = CAPACITY_2951_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    obs9 = reads.split_request
    exp9 = CAPACITY_2951_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10-12. R3 -- selection chooses one eligible hottest shard and exactly the
    # caller-supplied next generation.
    selected = select_split_target(
        SplitSelectionInput(
            shard_loads=(("shard-a", 70, True), ("shard-b", 90, True), ("shard-c", 99, False)),
            next_generation=18,
        )
    )
    obs10 = selected.shard_id
    exp10 = CAPACITY_2951_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    obs11 = selected.generation
    exp11 = CAPACITY_2951_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    alternate_selection = select_split_target(
        SplitSelectionInput(
            shard_loads=(("shard-a", 70, True), ("shard-b", 90, True), ("shard-c", 99, False)),
            next_generation=41,
        )
    )
    obs12 = alternate_selection.generation
    exp12 = CAPACITY_2951_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R3 -- equal eligible loads use a deterministic (lexical) tie break.
    tied = select_split_target(
        SplitSelectionInput(shard_loads=(("shard-b", 90, True), ("shard-a", 90, True)), next_generation=18)
    )
    obs13 = tied.shard_id
    exp13 = CAPACITY_2951_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R5 -- v1 has a closed decision vocabulary and never proposes merge.
    obs14 = tuple(action.value for action in CapacityAction)
    exp14 = CAPACITY_2951_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": CAPACITY_2951_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {"case_id": "capacity-2951-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
