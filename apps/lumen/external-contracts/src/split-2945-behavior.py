"""EC behavior case for #2945 -- replicated split admission and ordering.

Every expected value is an EC-owned literal transcribed from #2945 R1 (one or
three voters and non-negative read replicas), R2 (the ordered split checkpoint
vocabulary), R3 (all target members before catalog cutover), R4 (one active
mutation per instance), R5 (split is the admitted mutation kind), and AC3
(source retirement follows durable committed cutover).  The imports deliberately
fail closed until the pure split design model lands.
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

MINIMUM_CHECKS = 22

SPLIT_2945_BEHAVIOR_MATRIX = (
    ("one_voter_source_is_preserved", 1),
    ("three_voter_source_is_preserved", 3),
    ("three_voter_split_left_target_has_three_voters", 3),
    ("three_voter_split_left_target_preserves_read_replicas", 2),
    ("three_voter_split_right_target_preserves_read_replicas", 2),
    ("split_checkpoint_begins_with_prepare", "prepare"),
    ("prepare_advances_to_bulk_copy", "bulk_copy"),
    ("bulk_copy_advances_to_wal_catch_up", "wal_catch_up"),
    ("wal_catch_up_advances_to_write_fence", "write_fence"),
    ("write_fence_advances_to_catalog_cutover", "catalog_cutover"),
    ("catalog_cutover_advances_to_source_pruning", "source_pruning"),
    ("source_pruning_advances_to_cleanup", "cleanup"),
    ("caught_up_targets_advance_catalog_generation", 8),
    ("catalog_cutover_does_not_retire_sources", "not_retired"),
    ("first_mutation_returns_its_identity", "split-2945-a"),
    ("first_mutation_starts_at_prepare", "prepare"),
    ("split_kind_is_admitted", "split"),
    ("durable_committed_targets_admit_source_retirement", "admitted"),
    ("one_voter_split_left_target_has_one_voter", 1),
    ("one_voter_split_right_target_has_one_voter", 1),
    ("one_voter_split_left_target_preserves_read_replicas", 0),
    ("one_voter_split_right_target_preserves_read_replicas", 0),
)


def verify_split_2945_behavior() -> dict:
    checks = []

    one_voter = decide_replicated_split(
        ReplicatedSplitRequest(source_voters=1, target_read_replicas=0)
    )
    # 1. R1 -- the single-voter v1 shape remains a valid split source.
    obs1 = one_voter.source_voters
    exp1 = SPLIT_2945_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    three_voter = decide_replicated_split(
        ReplicatedSplitRequest(source_voters=3, target_read_replicas=2)
    )
    # 2. R1 -- replicated sources are admitted, not collapsed to one voter.
    obs2 = three_voter.source_voters
    exp2 = SPLIT_2945_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- each resulting group keeps the requested voting membership.
    obs3 = three_voter.left_target.voters
    exp3 = SPLIT_2945_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R1 -- the left target carries the explicit non-voting replica count.
    obs4 = three_voter.left_target.read_replicas
    exp4 = SPLIT_2945_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1 -- the same explicit count is retained independently on the right.
    obs5 = three_voter.right_target.read_replicas
    exp5 = SPLIT_2945_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    first = advance_split_checkpoint(CheckpointTransition(current=None, requested="prepare"))
    # 6. R2 -- a new split begins with prepare.
    obs6 = first.checkpoint
    exp6 = SPLIT_2945_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    bulk = advance_split_checkpoint(CheckpointTransition(current="prepare", requested="bulk_copy"))
    # 7. R2 -- prepare precedes the target bulk copy.
    obs7 = bulk.checkpoint
    exp7 = SPLIT_2945_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    catch_up = advance_split_checkpoint(CheckpointTransition(current="bulk_copy", requested="wal_catch_up"))
    # 8. R2 -- copied data must catch up its WAL before the fence.
    obs8 = catch_up.checkpoint
    exp8 = SPLIT_2945_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    fence = advance_split_checkpoint(CheckpointTransition(current="wal_catch_up", requested="write_fence"))
    # 9. R2 -- WAL catch-up precedes the write fence.
    obs9 = fence.checkpoint
    exp9 = SPLIT_2945_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    cutover = advance_split_checkpoint(CheckpointTransition(current="write_fence", requested="catalog_cutover"))
    # 10. R2 -- the write fence is the direct predecessor of catalog cutover.
    obs10 = cutover.checkpoint
    exp10 = SPLIT_2945_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    pruning = advance_split_checkpoint(CheckpointTransition(current="catalog_cutover", requested="source_pruning"))
    # 11. R2 -- source pruning is only after catalog cutover.
    obs11 = pruning.checkpoint
    exp11 = SPLIT_2945_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    cleanup = advance_split_checkpoint(CheckpointTransition(current="source_pruning", requested="cleanup"))
    # 12. R2 -- cleanup closes the required checkpoint sequence.
    obs12 = cleanup.checkpoint
    exp12 = SPLIT_2945_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    catalog = decide_catalog_cutover(
        CatalogCutoverRequest(
            current_generation=7,
            target_members=(TargetMember(member_id="left-0", caught_up=True), TargetMember(member_id="right-0", caught_up=True)),
        )
    )
    # 13. R3 -- all caught-up members permit exactly the next catalog generation.
    obs13 = catalog.next_catalog_generation
    exp13 = SPLIT_2945_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R3 -- catalog admission alone never retires the old sources.
    obs14 = catalog.source_retirement_status
    exp14 = SPLIT_2945_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    started = start_topology_mutation(
        TopologyMutationRequest(instance_id="lumen-a", mutation_id="split-2945-a", active_mutation_id=None)
    )
    # 15. R4 -- the first mutation returns the identity supplied by the caller.
    obs15 = started.mutation_id
    exp15 = SPLIT_2945_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R4 -- that mutation is explicitly initialized at its durable checkpoint.
    obs16 = started.initial_checkpoint
    exp16 = SPLIT_2945_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    split_kind = decide_topology_mutation_kind(TopologyMutationKind.SPLIT)
    # 17. R5 -- split is the one topology-mutation kind this issue admits.
    obs17 = split_kind.kind
    exp17 = SPLIT_2945_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    retirement = decide_source_retirement(
        SourceRetirementRequest(targets_durable=True, catalog_cutover_committed=True)
    )
    # 18. AC3 -- both supplied predicates admit source retirement.
    obs18 = retirement.outcome
    exp18 = SPLIT_2945_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R1 -- the left one-voter target retains the requested voting membership.
    obs19 = one_voter.left_target.voters
    exp19 = SPLIT_2945_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R1 -- the right one-voter target retains the requested voting membership.
    obs20 = one_voter.right_target.voters
    exp20 = SPLIT_2945_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21. R1 -- the left one-voter target retains the requested non-voting count.
    obs21 = one_voter.left_target.read_replicas
    exp21 = SPLIT_2945_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    # 22. R1 -- the right one-voter target retains the requested non-voting count.
    obs22 = one_voter.right_target.read_replicas
    exp22 = SPLIT_2945_BEHAVIOR_MATRIX[21][1]
    checks.append({"name": SPLIT_2945_BEHAVIOR_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    return {
        "case_id": "split-2945-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(check["passed"] for check in checks) and len(checks) == MINIMUM_CHECKS,
    }
