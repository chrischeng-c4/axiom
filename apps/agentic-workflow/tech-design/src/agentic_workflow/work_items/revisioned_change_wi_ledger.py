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
      LifecycleEvent, InvalidationRecord, ChangeLifecycle, and persistent record serde layout
      at .aw/causal-lifecycle/<slug>.json with atomic local file storage helpers in Rust.
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    description: >
      Integrate root-aware causal_lifecycle JSON load/render into read-only aw wi show <id>,
      run_create handler order (backend create success precedes fold_wi_create build/reduce/save),
      and run_update handler order (retain prior body, backend update, compare canonical old/new digest,
      no-op returns old carrier bytes without writes; changed body builds wi_change event with persisted predecessor, reduces, and saves).
      Add the named issues-handler tests in RUST_ISSUES_TEST_SEAMS: actual LocalBackend-backed public
      create/update dispatch proves create-after-backend-success, equal-body carrier byte preservation,
      changed-body predecessor/epoch progression and a reopened backend; the actual JSON show handler
      fingerprints carrier bytes before and after every show; and a fresh aw binary process hydrates
      the saved carrier. Do not test only an extracted pure reducer or serializer.
  - path: apps/agentic-workflow/src/cli/mod.rs
    action: modify
    description: >
      Register change_lifecycle module and expose persistence/hydration helper bindings for issues CLI.
"""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/revisioned-change-wi-ledger-wi-3347"
__aw_work_item__ = "3347"


# These are implementation-obligatory Rust test seams, deliberately pinned in
# the TD because the Python contracts below exercise only the model. They
# bind the CB to the real `issues.rs` handlers and live CLI parser rather than
# permitting a false-green pure-helper test.
RUST_ISSUES_TEST_SEAMS: dict[str, str] = {
    "revisioned_change_wi_ec_draft_command_round_trips_live_cli_parser": (
        "call cli::run::ec_draft_command(project, wi) and pass its exact output to "
        "crate::cli::chain::validate_aw_command_string"
    ),
    "revisioned_change_wi_local_create_update_persists_and_noop_is_byte_stable": (
        "drive public IssuesCommand::Create and IssuesCommand::Update against an isolated "
        "LocalBackend; observe the carrier after backend create, preserve its exact bytes for an "
        "equal --body-file update, advance head and epoch for a changed update, then reopen LocalBackend"
    ),
    "revisioned_change_wi_show_json_handler_is_carrier_byte_readonly": (
        "invoke the root-aware JSON show handler used by aw wi show --json twice and fingerprint "
        ".aw/causal-lifecycle/<slug>.json before and after each invocation"
    ),
    "revisioned_change_wi_fresh_binary_show_hydrates_existing_carrier": (
        "invoke a second aw wi show <slug> --json process against the same local project and require "
        "the saved head, epoch and active WI revision without rewriting the carrier"
    ),
}


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

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> CausalParent:
        return cls(revision_id=str(data["id"]), digest=str(data["digest"]))


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
            "kind": self.kind.value if isinstance(self.kind, ArtifactKind) else str(self.kind),
            "digest": self.digest,
            "parents": [p.to_dict() for p in self.parents],
            "iteration": self.iteration,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ArtifactRevision:
        parents = tuple(
            CausalParent.from_dict(p)
            for p in data.get("parents", [])
            if isinstance(p, dict)
        )
        return cls(
            id=str(data["id"]),
            kind=ArtifactKind(data.get("kind", "wi")),
            digest=str(data["digest"]),
            parents=parents,
            iteration=int(data.get("iteration", 1)),
        )


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

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> ActiveDigestTuple:
        return cls(
            wi_digest=data.get("wi_digest"),
            ec_digest=data.get("ec_digest"),
            td_digest=data.get("td_digest"),
            cb_digest=data.get("cb_digest"),
        )


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

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> EvidenceBinding:
        bound_tuple = ActiveDigestTuple.from_dict(data.get("bound_tuple", {}))
        return cls(
            verifier=str(data["verifier"]),
            bound_tuple=bound_tuple,
            passed=bool(data.get("passed", True)),
            summary=str(data.get("summary", "")),
        )


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
            "trigger_kind": self.trigger_kind.value if isinstance(self.trigger_kind, ArtifactKind) else str(self.trigger_kind),
            "invalidated_kinds": [k.value if isinstance(k, ArtifactKind) else str(k) for k in self.invalidated_kinds],
            "invalidated_revision_ids": list(self.invalidated_revision_ids),
            "evicted_evidence_verifiers": list(self.evicted_evidence_verifiers),
            "reason": self.reason,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> InvalidationRecord:
        return cls(
            trigger_revision_id=str(data.get("trigger_revision_id", "")),
            trigger_kind=ArtifactKind(data.get("trigger_kind", "wi")),
            invalidated_kinds=tuple(ArtifactKind(k) for k in data.get("invalidated_kinds", [])),
            invalidated_revision_ids=tuple(data.get("invalidated_revision_ids", [])),
            evicted_evidence_verifiers=tuple(data.get("evicted_evidence_verifiers", [])),
            reason=str(data.get("reason", "")),
        )


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

    def to_dict(self) -> dict[str, Any]:
        return {
            "event_id": self.event_id,
            "predecessor_id": self.predecessor_id,
            "kind": self.kind.value if isinstance(self.kind, LifecycleEventKind) else str(self.kind),
            "candidate_revision": self.candidate_revision.to_dict(),
            "bound_tuple": self.bound_tuple.to_dict(),
            "next_command": self.next_command,
            "next_owner": self.next_owner.value if isinstance(self.next_owner, OwnerVocabulary) else str(self.next_owner),
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> LifecycleEvent:
        cand_dict = data.get("candidate_revision", {})
        cand_rev = ArtifactRevision.from_dict(cand_dict)
        bound_tuple = ActiveDigestTuple.from_dict(data.get("bound_tuple", {}))
        return cls(
            event_id=str(data["event_id"]),
            predecessor_id=data.get("predecessor_id"),
            kind=LifecycleEventKind(data["kind"]),
            candidate_revision=cand_rev,
            bound_tuple=bound_tuple,
            next_command=str(data["next_command"]),
            next_owner=OwnerVocabulary(data["next_owner"]),
        )


@dataclass(frozen=True)
class NextObligation:
    """Exactly one resumable next command and stage owner."""

    command: str
    owner: OwnerVocabulary

    def to_dict(self) -> dict[str, str]:
        return {"command": self.command, "owner": self.owner.value if isinstance(self.owner, OwnerVocabulary) else str(self.owner)}

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> NextObligation:
        return cls(
            command=str(data["command"]),
            owner=OwnerVocabulary(data.get("owner", "wi")),
        )


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

    def to_persistent_dict(self) -> dict[str, Any]:
        """Serde serializable persistent-record schema at .aw/causal-lifecycle/<slug>.json."""
        active_dict = {}
        for kind in ArtifactKind:
            rev = self.active_revisions.get(kind)
            active_dict[kind.value] = rev.to_dict() if rev else None

        return {
            "schema": "aw.change-lifecycle.v1",
            "slug": self.slug,
            "epoch": self.epoch,
            "head_event_id": self.head_event_id,
            "iteration": self.iteration,
            "terminal": self.terminal,
            "next": self.next_obligation.to_dict(),
            "active_revisions": active_dict,
            "events": [evt.to_dict() for evt in self.events],
            "evidence": [ev.to_dict() for ev in self.evidence_bindings],
            "invalidations": [inv.to_dict() for inv in self.invalidations],
        }

    @classmethod
    def from_persistent_dict(cls, payload: dict[str, Any]) -> ChangeLifecycle:
        """Hydrate ChangeLifecycle from deserialized .aw/causal-lifecycle/<slug>.json dictionary."""
        slug = str(payload.get("slug", "3347"))
        epoch = int(payload.get("epoch", 1))
        head_event_id = payload.get("head_event_id")
        iteration = int(payload.get("iteration", 1))
        terminal = bool(payload.get("terminal", False))

        next_data = payload.get("next", {})
        next_ob = NextObligation.from_dict(next_data)

        active_revs: dict[ArtifactKind, ArtifactRevision | None] = {}
        raw_active = payload.get("active_revisions", {})
        for kind in ArtifactKind:
            rev_raw = raw_active.get(kind.value)
            active_revs[kind] = ArtifactRevision.from_dict(rev_raw) if rev_raw else None

        events = tuple(
            LifecycleEvent.from_dict(e)
            for e in payload.get("events", [])
            if isinstance(e, dict)
        )
        evidence = tuple(
            EvidenceBinding.from_dict(ev)
            for ev in payload.get("evidence", [])
            if isinstance(ev, dict)
        )
        invalidations = tuple(
            InvalidationRecord.from_dict(inv)
            for inv in payload.get("invalidations", [])
            if isinstance(inv, dict)
        )

        return cls(
            slug=slug,
            epoch=epoch,
            head_event_id=head_event_id,
            active_revisions=active_revs,
            events=events,
            evidence_bindings=evidence,
            invalidations=invalidations,
            iteration=iteration,
            terminal=terminal,
            next_obligation=next_ob,
        )


def get_ledger_path(project_root: Any, slug: str) -> Any:
    """Return root-relative persistent record path .aw/causal-lifecycle/<slug>.json."""
    from pathlib import Path
    return Path(project_root) / ".aw" / "causal-lifecycle" / f"{slug}.json"


def save_ledger_record(project_root: Any, lifecycle: ChangeLifecycle) -> Any:
    """Atomic local persistence adapter for .aw/causal-lifecycle/<slug>.json.

    Creates directory if missing, writes to temporary replacement file, and renames atomically.
    Raises IOError if write fails.
    """
    import json
    import os
    from pathlib import Path

    ledger_path = get_ledger_path(project_root, slug=lifecycle.slug)
    ledger_dir = ledger_path.parent
    ledger_dir.mkdir(parents=True, exist_ok=True)

    tmp_path = ledger_dir / f"{ledger_path.name}.tmp.{os.getpid()}"
    payload = lifecycle.to_persistent_dict()
    tmp_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    tmp_path.replace(ledger_path)
    return ledger_path


def compute_digest(content: str) -> str:
    """Compute sha256 content digest."""
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def build_ec_draft_command(slug: str, project: str) -> str:
    """Mirror `cli::run::ec_draft_command(project, wi)` exactly.

    The Rust implementation must call that existing canonical builder rather
    than hand-formatting a new grammar.  This executable TD mirror preserves
    its emitted `aw ec draft <id> --project <project> --wi <wi>` order.
    """
    return f"aw ec draft {slug} --project {project} --wi {slug}"


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
    import tempfile
    from pathlib import Path

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

    rev_wi_same_id = compute_revision_identity(ArtifactKind.WI, digest_wi, ())
    assert rev_wi.id == rev_wi_same_id

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

    content_wi_v2 = content_wi + "\n- Updated requirement"
    digest_wi_v2 = compute_digest(content_wi_v2)
    rev_wi_2 = ArtifactRevision(
        id=compute_revision_identity(ArtifactKind.WI, digest_wi_v2, ()),
        kind=ArtifactKind.WI,
        digest=digest_wi_v2,
        parents=(),
        iteration=2,
    )

    parent_wi_2 = CausalParent(revision_id=rev_wi_2.id, digest=rev_wi_2.digest)
    rebind_ec_id = compute_revision_identity(ArtifactKind.EC, digest_ec, (parent_wi_2,))
    assert rebind_ec_id != rev_ec_1.id

    cmd_plain = build_ec_draft_command("3347", "agentic-workflow")
    assert cmd_plain == "aw ec draft 3347 --project agentic-workflow --wi 3347"
    assert (
        RUST_ISSUES_TEST_SEAMS[
            "revisioned_change_wi_ec_draft_command_round_trips_live_cli_parser"
        ]
        == "call cli::run::ec_draft_command(project, wi) and pass its exact output to "
        "crate::cli::chain::validate_aw_command_string"
    )
    assert "LocalBackend" in RUST_ISSUES_TEST_SEAMS[
        "revisioned_change_wi_local_create_update_persists_and_noop_is_byte_stable"
    ]

    event_1 = LifecycleEvent(
        event_id="evt-001",
        predecessor_id=None,
        kind=LifecycleEventKind.WI_CREATE,
        candidate_revision=rev_wi,
        bound_tuple=ActiveDigestTuple(wi_digest=digest_wi),
        next_command=cmd_plain,
        next_owner=OwnerVocabulary.WI,
    )

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
        next_obligation=NextObligation(cmd_plain, OwnerVocabulary.WI),
    )

    assert lifecycle.epoch == 1
    assert lifecycle.head_event_id == "evt-001"
    assert lifecycle.active_digest_tuple().wi_digest == digest_wi
    assert lifecycle.next_obligation.owner == OwnerVocabulary.WI

    # Test serde serialization and deserialization
    p_dict = lifecycle.to_persistent_dict()
    assert p_dict["schema"] == "aw.change-lifecycle.v1"
    assert p_dict["slug"] == "3347"
    assert p_dict["active_revisions"]["wi"]["id"] == rev_wi.id

    rehydrated = ChangeLifecycle.from_persistent_dict(p_dict)
    assert rehydrated.slug == lifecycle.slug
    assert rehydrated.epoch == lifecycle.epoch
    assert rehydrated.head_event_id == lifecycle.head_event_id
    assert rehydrated.active_revisions[ArtifactKind.WI].id == rev_wi.id

    # Test atomic write to path
    with tempfile.TemporaryDirectory() as tmpdir:
        written_path = save_ledger_record(tmpdir, lifecycle)
        assert written_path.exists()
        assert written_path.name == "3347.json"
        assert written_path.parent.name == "causal-lifecycle"

    # Named Rust test selectors and Python EC falsifiable gates
    # - Rust test: revisioned_change_wi_ledger
    # - Rust test: revisioned_change_wi_parent_rebind
    # - Rust test: revisioned_change_wi_invalidation
    # - Python EC: revisioned-change-wi-ledger
    return "ok"
