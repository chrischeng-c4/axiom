"""Tech design for WI #3347: Model WI, EC, TD, and CB as typed artifact revisions and single-owner ChangeLifecycle.

@spec #3347
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import StrEnum
import hashlib
from typing import Any


__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/src/cli/change_lifecycle.rs
    action: create
    description: >
      Define typed ArtifactRevision, CausalParent, ActiveDigestTuple, LifecycleEventKind,
      LifecycleEvent, InvalidationRecord, and ChangeLifecycle single-owner ledger domain model in Rust.
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    description: >
      Integrate causal_lifecycle JSON projection into aw wi show <id>.
"""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/revisioned-change-wi-ledger-wi-3347"
__aw_work_item__ = "3347"


class ArtifactKind(StrEnum):
    WI = "wi"
    EC = "ec"
    TD = "td"
    CB = "cb"


class OwnerVocabulary(StrEnum):
    WI = "wi"
    EC = "ec"
    TD = "td"
    CB = "cb"
    MIGRATION = "migration"


class LifecycleEventKind(StrEnum):
    WI_CREATE = "wi_create"
    WI_CHANGE = "wi_change"
    EC_CHANGE = "ec_change"
    TD_CHANGE = "td_change"
    CB_CHANGE = "cb_change"
    EC_VERIFY = "ec_verify"
    TD_RECONCILE = "td_reconcile"
    FEEDBACK = "feedback"
    BLOCKED = "blocked"
    REBIND = "rebind"
    STALE_PREDECESSOR = "stale_predecessor"
    MALFORMED = "malformed"
    CB_COMMIT = "cb_commit"


@dataclass(frozen=True)
class CausalParent:
    """Typed causal parent pair containing both revision ID and content digest."""

    revision_id: str
    digest: str

    def to_dict(self) -> dict[str, str]:
        return {"id": self.revision_id, "digest": self.digest}


@dataclass(frozen=True)
class ArtifactRevision:
    """Typed artifact revision bound by content digest and causal parent set."""

    id: str
    kind: ArtifactKind
    digest: str
    parents: tuple[CausalParent, ...]
    iteration: int
    superseded_by: str | None = None
    invalidation_reason: str | None = None

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "digest": self.digest,
            "parents": [p.to_dict() for p in self.parents],
        }


@dataclass(frozen=True)
class ActiveDigestTuple:
    """Active four-dimensional content digest tuple across WI, EC, TD, and CB."""

    wi_digest: str | None = None
    ec_digest: str | None = None
    td_digest: str | None = None
    cb_digest: str | None = None

    def matches(self, other: ActiveDigestTuple) -> bool:
        return (
            self.wi_digest == other.wi_digest
            and self.ec_digest == other.ec_digest
            and self.td_digest == other.td_digest
            and self.cb_digest == other.cb_digest
        )

    def to_dict(self) -> dict[str, str | None]:
        return {
            "wi_digest": self.wi_digest,
            "ec_digest": self.ec_digest,
            "td_digest": self.td_digest,
            "cb_digest": self.cb_digest,
        }


@dataclass(frozen=True)
class EvidenceBinding:
    """Evidence bound strictly to a complete active digest tuple across all stages."""

    verifier: str
    bound_tuple: ActiveDigestTuple
    passed: bool
    summary: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "verifier": self.verifier,
            "bound_tuple": self.bound_tuple.to_dict(),
            "passed": self.passed,
            "summary": self.summary,
        }


@dataclass(frozen=True)
class InvalidationRecord:
    """Typed invalidation record naming the trigger revision, invalidated kinds, and evicted evidence."""

    trigger_revision_id: str
    trigger_kind: ArtifactKind
    invalidated_kinds: tuple[ArtifactKind, ...]
    invalidated_revision_ids: tuple[str, ...]
    evicted_evidence_verifiers: tuple[str, ...]
    reason: str

    def to_dict(self) -> dict[str, Any]:
        return {
            "trigger_revision_id": self.trigger_revision_id,
            "trigger_kind": self.trigger_kind.value,
            "invalidated_kinds": [k.value for k in self.invalidated_kinds],
            "invalidated_revision_ids": list(self.invalidated_revision_ids),
            "evicted_evidence_verifiers": list(self.evicted_evidence_verifiers),
            "reason": self.reason,
        }


@dataclass(frozen=True)
class LifecycleEvent:
    """Immutable append-only lifecycle event carrying a typed candidate ArtifactRevision."""

    event_id: str
    predecessor_id: str | None
    kind: LifecycleEventKind
    candidate_revision: ArtifactRevision
    bound_tuple: ActiveDigestTuple
    next_command: str
    next_owner: OwnerVocabulary


@dataclass(frozen=True)
class NextObligation:
    """Exactly one resumable next command and stage owner."""

    command: str
    owner: OwnerVocabulary

    def to_dict(self) -> dict[str, str]:
        return {"command": self.command, "owner": self.owner.value}


@dataclass(frozen=True)
class ChangeLifecycle:
    """Single typed owner for causal revisions, immutable event ledger, evidence, and next obligation."""

    slug: str
    epoch: int
    head_event_id: str | None
    active_revisions: dict[ArtifactKind, ArtifactRevision | None]
    events: tuple[LifecycleEvent, ...]
    evidence_bindings: tuple[EvidenceBinding, ...]
    invalidations: tuple[InvalidationRecord, ...]
    iteration: int
    terminal: bool
    next_obligation: NextObligation

    def active_digest_tuple(self) -> ActiveDigestTuple:
        wi_rev = self.active_revisions.get(ArtifactKind.WI)
        ec_rev = self.active_revisions.get(ArtifactKind.EC)
        td_rev = self.active_revisions.get(ArtifactKind.TD)
        cb_rev = self.active_revisions.get(ArtifactKind.CB)
        return ActiveDigestTuple(
            wi_digest=wi_rev.digest if wi_rev else None,
            ec_digest=ec_rev.digest if ec_rev else None,
            td_digest=td_rev.digest if td_rev else None,
            cb_digest=cb_rev.digest if cb_rev else None,
        )


def compute_digest(content: str) -> str:
    """Compute sha256 content digest."""
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def causal_parent_sort_key(parent: CausalParent) -> str:
    """Provide the canonical parent ordering used in revision identity."""
    return parent.revision_id


def compute_revision_identity(
    kind: ArtifactKind, content_digest: str, parents: tuple[CausalParent, ...]
) -> str:
    """Revision identity derives from content digest AND causal parent tuple (id + digest)."""
    parent_repr = ",".join(
        f"{parent.revision_id}:{parent.digest}"
        for parent in sorted(parents, key=causal_parent_sort_key)
    )
    raw = f"{kind.value}:{content_digest}:{parent_repr}"
    return f"rev-{hashlib.sha256(raw.encode('utf-8')).hexdigest()[:12]}"


def compute_transitive_invalidation(
    trigger_rev: ArtifactRevision,
    current_revisions: dict[ArtifactKind, ArtifactRevision | None],
    current_evidence: tuple[EvidenceBinding, ...],
) -> InvalidationRecord:
    """Defect 2 Repair: Calculate exact transitive invalidation record for active revisions and evidence eviction."""
    invalidated_kinds: list[ArtifactKind] = []
    if trigger_rev.kind == ArtifactKind.WI:
        invalidated_kinds = [ArtifactKind.EC, ArtifactKind.TD, ArtifactKind.CB]
    elif trigger_rev.kind == ArtifactKind.EC:
        invalidated_kinds = [ArtifactKind.TD, ArtifactKind.CB]
    elif trigger_rev.kind == ArtifactKind.TD:
        invalidated_kinds = [ArtifactKind.CB]
    elif trigger_rev.kind == ArtifactKind.CB:
        invalidated_kinds = []

    invalidated_revision_ids = tuple(
        revision.id
        for kind in invalidated_kinds
        if (revision := current_revisions.get(kind)) is not None
    )
    # A revision update (including a parent-only rebind) makes every existing
    # witness causally stale.  The reducer replaces the active revision before
    # asking the responsible stage to regenerate its evidence; no old green is
    # retained merely because one content digest happens to be unchanged.
    evicted_verifiers = [ev.verifier for ev in current_evidence]

    return InvalidationRecord(
        trigger_revision_id=trigger_rev.id,
        trigger_kind=trigger_rev.kind,
        invalidated_kinds=tuple(invalidated_kinds),
        invalidated_revision_ids=invalidated_revision_ids,
        evicted_evidence_verifiers=tuple(sorted(set(evicted_verifiers))),
        reason=f"Transitive invalidation triggered by {trigger_rev.kind.value} revision {trigger_rev.id}",
    )


def design_contract() -> str:
    """Express executable design contract for revisioned WI-EC-TD-CB ChangeLifecycle."""

    # R1: Typed revision identity with CausalParent (id, digest)
    content_wi = "## Problem\nDefine causal ledger."
    digest_wi = compute_digest(content_wi)

    rev_wi = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.WI, digest_wi, ()),
        kind=ArtifactKind.WI,
        digest=digest_wi,
        parents=(),
        iteration=1,
    )
    assert rev_wi.kind == ArtifactKind.WI
    assert rev_wi.parents == ()

    # R2: Same content + same parents is a no-op identity
    rev_wi_same_id = compute_revision_identity(ArtifactKind.WI, digest_wi, ())
    assert rev_wi.id == rev_wi_same_id

    # R2 & R7: Same content under changed causal parent tuple emits a NEW rebind revision identity
    parent_wi_1 = CausalParent(revision_id=rev_wi.id, digest=rev_wi.digest)
    digest_ec = compute_digest("def verify(): pass")

    rev_ec_1 = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.EC, digest_ec, (parent_wi_1,)),
        kind=ArtifactKind.EC,
        digest=digest_ec,
        parents=(parent_wi_1,),
        iteration=1,
    )
    assert rev_ec_1.parents[0].revision_id == rev_wi.id
    assert rev_ec_1.parents[0].digest == rev_wi.digest

    # WI updates content -> new WI revision
    content_wi_v2 = content_wi + "\n- Updated requirement"
    digest_wi_v2 = compute_digest(content_wi_v2)
    rev_wi_2 = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.WI, digest_wi_v2, ()),
        kind=ArtifactKind.WI,
        digest=digest_wi_v2,
        parents=(),
        iteration=2,
    )

    # Rebind EC with same source content digest_ec but new parent rev_wi_2
    parent_wi_2 = CausalParent(revision_id=rev_wi_2.id, digest=rev_wi_2.digest)
    rebind_ec_id = compute_revision_identity(ArtifactKind.EC, digest_ec, (parent_wi_2,))
    assert rebind_ec_id != rev_ec_1.id

    # Defect 1: Typed LifecycleEvent carrying candidate_revision
    event_1 = LifecycleEvent(
        event_id="evt-001",
        predecessor_id=None,
        kind=LifecycleEventKind.WI_CREATE,
        candidate_revision=rev_wi,
        bound_tuple=ActiveDigestTuple(wi_digest=digest_wi),
        next_command="aw ec scaffold --wi 3347",
        next_owner=OwnerVocabulary.WI,
    )

    # Defect 2: InvalidationRecord fan-out calculation
    inval_rec = compute_transitive_invalidation(rev_wi_2, {ArtifactKind.WI: rev_wi, ArtifactKind.EC: rev_ec_1}, ())
    assert ArtifactKind.EC in inval_rec.invalidated_kinds
    assert ArtifactKind.TD in inval_rec.invalidated_kinds
    assert ArtifactKind.CB in inval_rec.invalidated_kinds

    lifecycle = ChangeLifecycle(
        slug="3347",
        epoch=1,
        head_event_id="evt-001",
        active_revisions={
            ArtifactKind.WI: rev_wi,
            ArtifactKind.EC: None,
            ArtifactKind.TD: None,
            ArtifactKind.CB: None,
        },
        events=(event_1,),
        evidence_bindings=(),
        invalidations=(inval_rec,),
        iteration=1,
        terminal=False,
        next_obligation=NextObligation("aw ec scaffold --wi 3347", OwnerVocabulary.WI),
    )

    assert lifecycle.epoch == 1
    assert lifecycle.head_event_id == "evt-001"
    assert lifecycle.active_digest_tuple().wi_digest == digest_wi
    assert lifecycle.next_obligation.owner == OwnerVocabulary.WI

    # Named Rust test selectors and Python EC falsifiable gates
    # - Rust test: revisioned_change_wi_ledger
    # - Rust test: revisioned_change_wi_parent_rebind
    # - Rust test: revisioned_change_wi_invalidation
    # - Python EC: revisioned-change-wi-ledger
    return "ok"
