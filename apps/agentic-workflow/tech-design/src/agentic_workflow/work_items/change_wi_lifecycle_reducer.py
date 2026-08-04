"""Tech design for WI #3347: Pure deterministic reducer for ChangeLifecycle forward, repair, rebind, and terminal transitions.

@spec #3347
"""

from __future__ import annotations

from dataclasses import dataclass, replace
from enum import StrEnum
from typing import Any

from .revisioned_change_wi_ledger import (
    ActiveDigestTuple,
    ArtifactKind,
    ArtifactRevision,
    CausalParent,
    ChangeLifecycle,
    EvidenceBinding,
    InvalidationRecord,
    LifecycleEvent,
    LifecycleEventKind,
    NextObligation,
    OwnerVocabulary,
    RUST_ISSUES_TEST_SEAMS,
    build_ec_draft_command,
    compute_transitive_invalidation,
)


__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/src/cli/change_lifecycle.rs
    action: modify
    description: >
      Implement deterministic ChangeLifecycle reducer, active-tuple evidence invalidation,
      parent-only rebind reduction, evidence eviction, CB commit terminal check, and existing-command folds.
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    description: >
      Fold wi_create (backend success precedes fold_wi_create build/reduce/save) and wi_update
      (retain prior body, backend update, compare canonical old/new digest; on equal body return old carrier bytes without writes; on change construct wi_change event, reduce, and save).
      Add revisioned_change_wi_local_create_update_persists_and_noop_is_byte_stable using public
      IssuesCommand create/update dispatch and an isolated LocalBackend; no pure-fold-only substitute.
  - path: apps/agentic-workflow/src/cli/mod.rs
    action: modify
    description: >
      Expose fold_wi_create and fold_wi_update reducer functions for issue creation and update workflows.
"""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/change-wi-lifecycle-reducer-wi-3347"
__aw_work_item__ = "3347"


class FailureOwnership(StrEnum):
    WI_DRIFT = "wi_drift"
    CONTRACT = "contract"
    DESIGN = "design"
    IMPLEMENTATION = "implementation"
    INFRASTRUCTURE = "infrastructure"


@dataclass(frozen=True)
class ReducerResult:
    lifecycle: ChangeLifecycle
    accepted: bool
    rejection_reason: str | None = None


def route_failure(failure_type: FailureOwnership, current_command: str) -> NextObligation:
    """R6 & Defect 3: Route red/blocked evidence through pure reducer stage ownership with explicit NextObligation."""
    if failure_type == FailureOwnership.WI_DRIFT:
        return NextObligation("aw wi validate", OwnerVocabulary.WI)
    elif failure_type == FailureOwnership.CONTRACT:
        return NextObligation("aw ec check", OwnerVocabulary.EC)
    elif failure_type == FailureOwnership.DESIGN:
        return NextObligation("aw td check", OwnerVocabulary.TD)
    elif failure_type == FailureOwnership.IMPLEMENTATION:
        return NextObligation("aw cb check", OwnerVocabulary.CB)
    elif failure_type == FailureOwnership.INFRASTRUCTURE:
        # Infrastructure failure retains same blocked obligation for retry
        return NextObligation(current_command, OwnerVocabulary.CB)
    return NextObligation("aw health", OwnerVocabulary.WI)


def expected_parent_set(
    lifecycle: ChangeLifecycle, kind: ArtifactKind
) -> tuple[CausalParent, ...] | None:
    """Return the exact active causal parent set a candidate must attest to.

    This is deliberately revision-aware rather than digest-only: a same-content
    rebind must still observe the new upstream revision identity.
    """
    upstream = {
        ArtifactKind.WI: (),
        ArtifactKind.EC: (ArtifactKind.WI,),
        ArtifactKind.TD: (ArtifactKind.WI, ArtifactKind.EC),
        ArtifactKind.CB: (ArtifactKind.WI, ArtifactKind.EC, ArtifactKind.TD),
    }[kind]
    parents: list[CausalParent] = []
    for upstream_kind in upstream:
        revision = lifecycle.active_revisions.get(upstream_kind)
        if revision is None:
            return None
        parents.append(CausalParent(revision.id, revision.digest))
    return tuple(parents)


def fold_wi_create(slug: str, body: str, project: str) -> ChangeLifecycle:
    """Post-success fold for aw wi create: constructs epoch 1 wi_create event and initial WI revision.

    Uses `cli::run::ec_draft_command(project, slug)`, not a local command
    template.  `issues.rs::run_create` calls this only after backend creation
    succeeds, atomically saves the record, then returns the successful result.
    """
    from .revisioned_change_wi_ledger import (
        compute_digest,
        compute_revision_identity,
    )

    digest = compute_digest(body)
    rev_wi = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.WI, digest, ()),
        kind=ArtifactKind.WI,
        digest=digest,
        parents=(),
        iteration=1,
    )

    next_cmd = build_ec_draft_command(slug, project=project)

    initial_event = LifecycleEvent(
        event_id="evt-001",
        predecessor_id=None,
        kind=LifecycleEventKind.WI_CREATE,
        candidate_revision=rev_wi,
        bound_tuple=ActiveDigestTuple(wi_digest=digest),
        next_command=next_cmd,
        next_owner=OwnerVocabulary.WI,
    )

    return ChangeLifecycle(
        slug=slug,
        epoch=1,
        head_event_id="evt-001",
        active_revisions={
            ArtifactKind.WI: rev_wi,
            ArtifactKind.EC: None,
            ArtifactKind.TD: None,
            ArtifactKind.CB: None,
        },
        events=(initial_event,),
        evidence_bindings=(),
        invalidations=(),
        iteration=1,
        terminal=False,
        next_obligation=NextObligation(next_cmd, OwnerVocabulary.WI),
    )


def fold_wi_update(
    prior_lifecycle: ChangeLifecycle,
    new_body: str,
    pre_update_body: str | None = None,
) -> ReducerResult:
    """Post-success fold for aw wi update --body-file: compares canonical WI digests.

    If old digest equals new digest: returns event-free no-op (head, epoch, events unchanged),
    writing NO event or carrier bytes to disk.
    If changed: uses prior persisted head as predecessor, constructs candidate wi_change, and reduces.
    Must be called only after backend update success in issues.rs run_update.
    """
    from .revisioned_change_wi_ledger import (
        compute_digest,
        compute_revision_identity,
    )

    new_digest = compute_digest(new_body)
    active_wi = prior_lifecycle.active_revisions.get(ArtifactKind.WI)
    old_digest = (
        active_wi.digest
        if active_wi
        else (compute_digest(pre_update_body) if pre_update_body else None)
    )

    if old_digest is not None and old_digest == new_digest:
        return ReducerResult(
            lifecycle=prior_lifecycle,
            accepted=False,
            rejection_reason="No-op transition: unchanged WI content digest",
        )

    cand_rev = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.WI, new_digest, ()),
        kind=ArtifactKind.WI,
        digest=new_digest,
        parents=(),
        iteration=prior_lifecycle.iteration + 1,
    )

    event_id = f"evt-{len(prior_lifecycle.events) + 1:03d}"
    update_event = LifecycleEvent(
        event_id=event_id,
        predecessor_id=prior_lifecycle.head_event_id,
        kind=LifecycleEventKind.WI_CHANGE,
        candidate_revision=cand_rev,
        bound_tuple=ActiveDigestTuple(wi_digest=new_digest),
        next_command=f"aw wi validate {prior_lifecycle.slug}",
        next_owner=OwnerVocabulary.WI,
    )

    return reduce_event(prior_lifecycle, update_event)


def reduce_event(lifecycle: ChangeLifecycle, event: LifecycleEvent) -> ReducerResult:
    """R4, R6, R7, R8 & Defect 1-3: Pure deterministic lifecycle transition reducer."""
    slug = lifecycle.slug

    # Defect 3: Predecessor check: fail closed with explicit NextObligation
    if event.predecessor_id != lifecycle.head_event_id:
        fail_closed_ob = NextObligation(
            command=f"aw wi validate {slug}",
            owner=OwnerVocabulary.WI,
        )
        return ReducerResult(
            lifecycle=replace(lifecycle, next_obligation=fail_closed_ob),
            accepted=False,
            rejection_reason=(
                f"Conflicting/stale predecessor_id {event.predecessor_id!r} "
                f"does not match current head_event_id {lifecycle.head_event_id!r}"
            ),
        )

    cand_rev = event.candidate_revision
    target_kind = cand_rev.kind
    active_rev = lifecycle.active_revisions.get(target_kind)
    expected_parents = expected_parent_set(lifecycle, target_kind)
    if expected_parents is None or cand_rev.parents != expected_parents:
        invalid_parent_ob = NextObligation(
            command=f"aw wi validate {slug}", owner=OwnerVocabulary.WI
        )
        return ReducerResult(
            lifecycle=replace(lifecycle, next_obligation=invalid_parent_ob),
            accepted=False,
            rejection_reason="candidate parent set does not match the active causal predecessor set",
        )

    # A commit does not revise CB source.  It records terminal evidence against
    # the already-active CB revision, so it must bypass the ordinary no-op and
    # invalidation path below.
    if event.kind == LifecycleEventKind.CB_COMMIT:
        if active_rev != cand_rev:
            blocked_ob = NextObligation("aw cb check", OwnerVocabulary.CB)
            return ReducerResult(
                lifecycle=replace(lifecycle, next_obligation=blocked_ob),
                accepted=False,
                rejection_reason="cb_commit candidate is not the current active CB revision",
            )
        updated_active_revisions = dict(lifecycle.active_revisions)
        new_active_tuple = lifecycle.active_digest_tuple()
        retained_evidence = lifecycle.evidence_bindings
        inval_rec = None
    else:
        # Exact same content and exact same parents is a NO-OP.  Changed
        # parents with identical content deliberately reach the rebind path.
        if active_rev and active_rev.digest == cand_rev.digest and active_rev.parents == cand_rev.parents:
            noop_ob = NextObligation(
                command=f"aw wi validate {slug}", owner=OwnerVocabulary.WI
            )
            return ReducerResult(
                lifecycle=replace(lifecycle, next_obligation=noop_ob),
                accepted=False,
                rejection_reason="No-op transition: unchanged content and unchanged causal parents",
            )

        updated_active_revisions = dict(lifecycle.active_revisions)
        updated_active_revisions[target_kind] = cand_rev
        inval_rec = compute_transitive_invalidation(
            cand_rev, lifecycle.active_revisions, lifecycle.evidence_bindings
        )
        for inv_kind in inval_rec.invalidated_kinds:
            updated_active_revisions[inv_kind] = None
        new_active_tuple = ActiveDigestTuple(
            wi_digest=updated_active_revisions[ArtifactKind.WI].digest if updated_active_revisions[ArtifactKind.WI] else None,
            ec_digest=updated_active_revisions[ArtifactKind.EC].digest if updated_active_revisions[ArtifactKind.EC] else None,
            td_digest=updated_active_revisions[ArtifactKind.TD].digest if updated_active_revisions[ArtifactKind.TD] else None,
            cb_digest=updated_active_revisions[ArtifactKind.CB].digest if updated_active_revisions[ArtifactKind.CB] else None,
        )
        # Any accepted revision event evicts all older witness records.  This
        # includes parent-only rebinds whose content-digest tuple is unchanged.
        retained_evidence = ()

    # Defect 3: Terminal check for CB_COMMIT
    if event.kind == LifecycleEventKind.CB_COMMIT:
        # Check 4D digest completeness
        if not (new_active_tuple.wi_digest and new_active_tuple.ec_digest and new_active_tuple.td_digest and new_active_tuple.cb_digest):
            failed_commit_ob = NextObligation(
                command="aw ec verify --stage cb",
                owner=OwnerVocabulary.CB,
            )
            return ReducerResult(
                lifecycle=replace(lifecycle, next_obligation=failed_commit_ob),
                accepted=False,
                rejection_reason="cb_commit rejected: active digest tuple is incomplete across WI/EC/TD/CB",
            )

        # Evidence witness check matching FULL 4D active tuple
        required_verifiers = {"cb_test", "cb_review", "td_reconcile", "ec_verify_cb"}
        matching_verifiers = set()
        for eb in retained_evidence:
            if eb.passed and eb.bound_tuple.matches(new_active_tuple) and eb.verifier in required_verifiers:
                matching_verifiers.add(eb.verifier)

        if matching_verifiers != required_verifiers:
            missing = sorted(required_verifiers - matching_verifiers)
            failed_commit_ob = NextObligation(
                command="aw ec verify --stage cb",
                owner=OwnerVocabulary.CB,
            )
            return ReducerResult(
                lifecycle=replace(lifecycle, next_obligation=failed_commit_ob),
                accepted=False,
                rejection_reason=f"cb_commit rejected: missing valid 4D active-tuple evidence for {missing}",
            )

        # Defect 3 Repair: Terminal observable check MUST be aw wi show <slug>!
        terminal_ob = NextObligation(
            command=f"aw wi show {slug}",
            owner=OwnerVocabulary.CB,
        )
        new_lifecycle = replace(
            lifecycle,
            epoch=lifecycle.epoch + 1,
            head_event_id=event.event_id,
            active_revisions=updated_active_revisions,
            events=(*lifecycle.events, event),
            evidence_bindings=retained_evidence,
            invalidations=(
                lifecycle.invalidations
                if inval_rec is None
                else (*lifecycle.invalidations, inval_rec)
            ),
            terminal=True,
            next_obligation=terminal_ob,
        )
        return ReducerResult(lifecycle=new_lifecycle, accepted=True)

    # Canonical forward, repair, or parent-rebind transition
    next_ob = NextObligation(command=event.next_command, owner=event.next_owner)
    new_lifecycle = replace(
        lifecycle,
        epoch=lifecycle.epoch + 1,
        head_event_id=event.event_id,
        active_revisions=updated_active_revisions,
        events=(*lifecycle.events, event),
        evidence_bindings=retained_evidence,
        invalidations=(*lifecycle.invalidations, inval_rec),
        next_obligation=next_ob,
    )
    return ReducerResult(lifecycle=new_lifecycle, accepted=True)


def design_contract() -> str:
    """Express executable design contract for ChangeLifecycle deterministic reducer."""

    # Set up active revisions and parent bindings
    parent_wi = CausalParent("rev-wi-1", "dig-wi-1")
    parent_ec = CausalParent("rev-ec-1", "dig-ec-1")
    parent_td = CausalParent("rev-td-1", "dig-td-1")

    rev_wi = ArtifactRevision("rev-wi-1", ArtifactKind.WI, "dig-wi-1", (), 1)
    rev_ec = ArtifactRevision("rev-ec-1", ArtifactKind.EC, "dig-ec-1", (parent_wi,), 1)
    rev_td = ArtifactRevision("rev-td-1", ArtifactKind.TD, "dig-td-1", (parent_wi, parent_ec), 1)
    rev_cb = ArtifactRevision("rev-cb-1", ArtifactKind.CB, "dig-cb-1", (parent_wi, parent_ec, parent_td), 1)

    draft_cmd = build_ec_draft_command("3347", "agentic-workflow")
    initial_event = LifecycleEvent(
        event_id="evt-001",
        predecessor_id=None,
        kind=LifecycleEventKind.WI_CREATE,
        candidate_revision=rev_wi,
        bound_tuple=ActiveDigestTuple("dig-wi-1", "dig-ec-1", "dig-td-1", "dig-cb-1"),
        next_command=draft_cmd,
        next_owner=OwnerVocabulary.WI,
    )

    lifecycle = ChangeLifecycle(
        slug="3347",
        epoch=1,
        head_event_id="evt-001",
        active_revisions={
            ArtifactKind.WI: rev_wi,
            ArtifactKind.EC: rev_ec,
            ArtifactKind.TD: rev_td,
            ArtifactKind.CB: rev_cb,
        },
        events=(initial_event,),
        evidence_bindings=(),
        invalidations=(),
        iteration=1,
        terminal=False,
        next_obligation=NextObligation(draft_cmd, OwnerVocabulary.WI),
    )

    # Test fold_wi_create via the same canonical EC draft command builder.
    created_lc = fold_wi_create("3347", "## Problem\nNew WI", "agentic-workflow")
    assert created_lc.slug == "3347"
    assert created_lc.epoch == 1
    assert created_lc.head_event_id == "evt-001"
    assert created_lc.events[0].kind == LifecycleEventKind.WI_CREATE
    assert (
        created_lc.next_obligation.command
        == "aw ec draft 3347 --project agentic-workflow --wi 3347"
    )
    assert (
        "IssuesCommand::Create" in RUST_ISSUES_TEST_SEAMS[
            "revisioned_change_wi_local_create_update_persists_and_noop_is_byte_stable"
        ]
    )

    # Test fold_wi_update with equal body digest (no-op)
    noop_fold = fold_wi_update(created_lc, "## Problem\nNew WI")
    assert not noop_fold.accepted
    assert noop_fold.lifecycle == created_lc

    # Test fold_wi_update with changed body digest
    update_fold = fold_wi_update(created_lc, "## Problem\nUpdated WI content")
    assert update_fold.accepted
    assert update_fold.lifecycle.epoch == 2
    assert update_fold.lifecycle.head_event_id == "evt-002"
    assert update_fold.lifecycle.events[-1].predecessor_id == "evt-001"

    # Vector 1: Event predecessor validation failure
    conflict_evt = LifecycleEvent(
        event_id="evt-002",
        predecessor_id="evt-stale",
        kind=LifecycleEventKind.EC_CHANGE,
        candidate_revision=ArtifactRevision("rev-ec-2", ArtifactKind.EC, "dig-ec-2", (parent_wi,), 2),
        bound_tuple=ActiveDigestTuple("dig-wi-1", "dig-ec-2"),
        next_command="aw ec check",
        next_owner=OwnerVocabulary.EC,
    )
    res_conflict = reduce_event(lifecycle, conflict_evt)
    assert not res_conflict.accepted
    assert res_conflict.rejection_reason is not None
    assert res_conflict.lifecycle.next_obligation.command == "aw wi validate 3347"
    assert res_conflict.lifecycle.next_obligation.owner == OwnerVocabulary.WI

    # Vector 2: No-op transition (equal content and equal parents)
    noop_evt = LifecycleEvent(
        event_id="evt-001",
        predecessor_id="evt-001",
        kind=LifecycleEventKind.WI_CHANGE,
        candidate_revision=rev_wi,
        bound_tuple=ActiveDigestTuple("dig-wi-1"),
        next_command="aw wi validate",
        next_owner=OwnerVocabulary.WI,
    )
    res_noop = reduce_event(lifecycle, noop_evt)
    assert not res_noop.accepted
    assert "No-op transition" in res_noop.rejection_reason
    assert res_noop.lifecycle.next_obligation.command == "aw wi validate 3347"

    # Vector 3: WI Invalidation fan-out and evidence eviction
    rev_wi_v2 = ArtifactRevision("rev-wi-2", ArtifactKind.WI, "dig-wi-2", (), 2)
    wi_update_evt = LifecycleEvent(
        event_id="evt-002",
        predecessor_id="evt-001",
        kind=LifecycleEventKind.WI_CHANGE,
        candidate_revision=rev_wi_v2,
        bound_tuple=ActiveDigestTuple("dig-wi-2"),
        next_command="aw wi validate 3347",
        next_owner=OwnerVocabulary.WI,
    )
    stale_ev = EvidenceBinding("wi_validate", ActiveDigestTuple("dig-wi-1"), True, "Old WI pass")
    lifecycle_stale = replace(lifecycle, evidence_bindings=(stale_ev,))

    res_wi_update = reduce_event(lifecycle_stale, wi_update_evt)
    assert res_wi_update.accepted
    assert res_wi_update.lifecycle.active_revisions[ArtifactKind.WI].id == "rev-wi-2"
    assert res_wi_update.lifecycle.active_revisions[ArtifactKind.EC] is None
    assert res_wi_update.lifecycle.active_revisions[ArtifactKind.TD] is None
    assert res_wi_update.lifecycle.active_revisions[ArtifactKind.CB] is None
    assert len(res_wi_update.lifecycle.evidence_bindings) == 0

    # Vector 4: Parent-only rebind transition
    parent_wi_v2 = CausalParent("rev-wi-2", "dig-wi-2")
    rebind_ec = ArtifactRevision("rev-ec-rebind", ArtifactKind.EC, "dig-ec-1", (parent_wi_v2,), 2)
    rebind_evt = LifecycleEvent(
        event_id="evt-003",
        predecessor_id="evt-002",
        kind=LifecycleEventKind.REBIND,
        candidate_revision=rebind_ec,
        bound_tuple=ActiveDigestTuple("dig-wi-2", "dig-ec-1"),
        next_command="aw ec check",
        next_owner=OwnerVocabulary.EC,
    )
    res_rebind = reduce_event(res_wi_update.lifecycle, rebind_evt)
    assert res_rebind.accepted
    assert res_rebind.lifecycle.active_revisions[ArtifactKind.EC].id == "rev-ec-rebind"

    # Vector 5: Terminal acceptance (cb_commit with valid 4D active-tuple evidence)
    tuple_4d = ActiveDigestTuple("dig-wi-1", "dig-ec-1", "dig-td-1", "dig-cb-1")
    valid_4d_evidence = (
        EvidenceBinding("cb_test", tuple_4d, True, "CB test pass"),
        EvidenceBinding("cb_review", tuple_4d, True, "CB review pass"),
        EvidenceBinding("td_reconcile", tuple_4d, True, "no_change"),
        EvidenceBinding("ec_verify_cb", tuple_4d, True, "4D EC pass"),
    )
    lifecycle_4d = replace(lifecycle, evidence_bindings=valid_4d_evidence)
    commit_evt = LifecycleEvent(
        event_id="evt-002",
        predecessor_id="evt-001",
        kind=LifecycleEventKind.CB_COMMIT,
        candidate_revision=rev_cb,
        bound_tuple=tuple_4d,
        next_command="aw wi show 3347",
        next_owner=OwnerVocabulary.CB,
    )
    res_commit = reduce_event(lifecycle_4d, commit_evt)
    assert res_commit.accepted
    assert res_commit.lifecycle.terminal is True
    assert res_commit.lifecycle.next_obligation.command == "aw wi show 3347"
    assert res_commit.lifecycle.next_obligation.owner == OwnerVocabulary.CB

    # Vector 6: Terminal rejection due to missing/stale 4D evidence
    res_rejected_commit = reduce_event(lifecycle, commit_evt)
    assert not res_rejected_commit.accepted
    assert res_rejected_commit.lifecycle.terminal is False
    assert res_rejected_commit.lifecycle.next_obligation.command == "aw ec verify --stage cb"

    # Named Rust test selectors
    # - Rust test: revisioned_change_wi_reducer
    return "ok"
