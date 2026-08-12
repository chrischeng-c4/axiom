"""EC security case for #2945 -- fail-closed replicated split decisions.

Every expected value is an EC-owned literal from #2945 R1 (unsupported voters
and negative replicas), R2 (no skipped checkpoint predecessor), R3 (every
target member caught up), R4 (one active mutation per instance), R5 (no merge,
shard contraction, or PVC shrink), and AC3 (durability and committed cutover
both precede retirement).  Missing split-design imports are intentional: the
external-contract gate must fail closed before that design exists.
"""

from __future__ import annotations

from lumen.topology.split import (
    CatalogCutoverRequest,
    CheckpointTransition,
    ReplicatedSplitRequest,
    SourceRetirementRequest,
    TargetMember,
    TopologyMutationKind,
    TopologyMutationRequest,
    advance_split_checkpoint,
    decide_catalog_cutover,
    decide_replicated_split,
    decide_source_retirement,
    decide_topology_mutation_kind,
    start_topology_mutation,
)

MINIMUM_CHECKS = 19

SPLIT_2945_SECURITY_MATRIX = (
    ("two_voter_source_is_rejected", "unsupported_voter_count"),
    ("two_voter_refusal_names_source_voters", "source_voters"),
    ("negative_target_read_replicas_are_rejected", "negative_read_replicas"),
    ("negative_read_replica_refusal_names_target_field", "target_read_replicas"),
    ("skipped_checkpoint_predecessor_is_rejected", "required_predecessor_skipped"),
    ("skipped_checkpoint_refusal_names_requested_checkpoint", "requested"),
    ("uncaught_target_member_blocks_catalog_cutover", "target_member_not_caught_up"),
    ("uncaught_member_refusal_names_the_member", "target_members[1].caught_up"),
    ("active_instance_mutation_rejects_second_mutation", "active_topology_mutation"),
    ("active_mutation_refusal_names_active_mutation", "active_mutation_id"),
    ("automatic_merge_is_rejected", "automatic_merge_not_supported"),
    ("shard_contraction_is_rejected", "shard_contraction_not_supported"),
    ("pvc_shrink_is_rejected", "pvc_shrink_not_supported"),
    ("undurable_targets_block_source_retirement", "targets_not_durable"),
    ("undurable_target_refusal_names_durability", "targets_durable"),
    ("uncommitted_cutover_blocks_source_retirement", "catalog_cutover_not_committed"),
    ("uncommitted_cutover_refusal_names_cutover", "catalog_cutover_committed"),
    ("split_remains_admitted_next_to_forbidden_kinds", "split"),
    ("three_voter_nonnegative_request_remains_admitted", 3),
)


def verify_split_2945_security() -> dict:
    checks = []

    bad_voters = decide_replicated_split(
        ReplicatedSplitRequest(source_voters=2, target_read_replicas=0)
    )
    # 1. R1 -- an explicit unsupported voter count is rejected.
    obs1 = bad_voters.reason
    exp1 = SPLIT_2945_SECURITY_MATRIX[0][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the refusal identifies the bad split-source field.
    obs2 = bad_voters.field_path
    exp2 = SPLIT_2945_SECURITY_MATRIX[1][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    bad_replicas = decide_replicated_split(
        ReplicatedSplitRequest(source_voters=3, target_read_replicas=-1)
    )
    # 3. R1 -- a negative requested replica count is never normalized away.
    obs3 = bad_replicas.reason
    exp3 = SPLIT_2945_SECURITY_MATRIX[2][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- that refusal points at the explicitly supplied target count.
    obs4 = bad_replicas.field_path
    exp4 = SPLIT_2945_SECURITY_MATRIX[3][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    skipped = advance_split_checkpoint(
        CheckpointTransition(current="prepare", requested="wal_catch_up")
    )
    # 5. R2 -- prepare cannot skip bulk copy on the way to WAL catch-up.
    obs5 = skipped.reason
    exp5 = SPLIT_2945_SECURITY_MATRIX[4][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R2 -- the transition refusal identifies the attempted successor.
    obs6 = skipped.field_path
    exp6 = SPLIT_2945_SECURITY_MATRIX[5][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    uncaught = decide_catalog_cutover(
        CatalogCutoverRequest(
            current_generation=7,
            target_members=(TargetMember(member_id="left-0", caught_up=True), TargetMember(member_id="right-0", caught_up=False)),
        )
    )
    # 7. R3 -- one lagging target member blocks the whole catalog generation.
    obs7 = uncaught.reason
    exp7 = SPLIT_2945_SECURITY_MATRIX[6][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- the failure points to the lagging member rather than a generic gate.
    obs8 = uncaught.field_path
    exp8 = SPLIT_2945_SECURITY_MATRIX[7][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    second = start_topology_mutation(
        TopologyMutationRequest(instance_id="lumen-a", mutation_id="split-2945-b", active_mutation_id="split-2945-a")
    )
    # 9. R4 -- an explicit active mutation refuses a second mutation on that instance.
    obs9 = second.reason
    exp9 = SPLIT_2945_SECURITY_MATRIX[8][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- the user is told which active mutation blocks their request.
    obs10 = second.field_path
    exp10 = SPLIT_2945_SECURITY_MATRIX[9][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    merge = decide_topology_mutation_kind(TopologyMutationKind.MERGE)
    # 11. R5 -- automatic merge is outside the split-only vocabulary.
    obs11 = merge.reason
    exp11 = SPLIT_2945_SECURITY_MATRIX[10][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    contraction = decide_topology_mutation_kind(TopologyMutationKind.SHARD_CONTRACTION)
    # 12. R5 -- shard contraction has its own stable refusal.
    obs12 = contraction.reason
    exp12 = SPLIT_2945_SECURITY_MATRIX[11][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    shrink = decide_topology_mutation_kind(TopologyMutationKind.PVC_SHRINK)
    # 13. R5 -- PVC shrink is equally unavailable from this mutation surface.
    obs13 = shrink.reason
    exp13 = SPLIT_2945_SECURITY_MATRIX[12][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    not_durable = decide_source_retirement(
        SourceRetirementRequest(targets_durable=False, catalog_cutover_committed=True)
    )
    # 14. AC3 -- a requested retirement stops until targets are durable.
    obs14 = not_durable.reason
    exp14 = SPLIT_2945_SECURITY_MATRIX[13][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. AC3 -- the first ordering failure names durability.
    obs15 = not_durable.field_path
    exp15 = SPLIT_2945_SECURITY_MATRIX[14][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    not_committed = decide_source_retirement(
        SourceRetirementRequest(targets_durable=True, catalog_cutover_committed=False)
    )
    # 16. AC3 -- durable targets are still insufficient before routing cutover commits.
    obs16 = not_committed.reason
    exp16 = SPLIT_2945_SECURITY_MATRIX[15][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. AC3 -- the second independent ordering failure names the cutover fact.
    obs17 = not_committed.field_path
    exp17 = SPLIT_2945_SECURITY_MATRIX[16][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    allowed_kind = decide_topology_mutation_kind(TopologyMutationKind.SPLIT)
    # 18. R5 -- neighbouring forbidden kinds do not accidentally refuse split.
    obs18 = allowed_kind.kind
    exp18 = SPLIT_2945_SECURITY_MATRIX[17][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    admitted = decide_replicated_split(
        ReplicatedSplitRequest(source_voters=3, target_read_replicas=0)
    )
    # 19. R1 -- the explicit neighbouring valid replicated input remains admitted.
    obs19 = admitted.right_target.voters
    exp19 = SPLIT_2945_SECURITY_MATRIX[18][1]
    checks.append({"name": SPLIT_2945_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    return {
        "case_id": "split-2945-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
