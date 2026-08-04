"""Tech design for WI #3347: Change WI lifecycle hydration, legacy fail-closed handling, and causal read model.

@spec #3347
"""

from __future__ import annotations

from dataclasses import dataclass
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
)


__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/src/cli/change_lifecycle.rs
    action: modify
    description: >
      Implement hydration reconstruction from append-only event ledger, legacy
      <!-- aw:loop-state ... --> zero-head migration detection, and causal_lifecycle JSON serializer.
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    description: >
      Expose causal_lifecycle (schema aw.change-lifecycle.v1) on read-only aw wi show.
"""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/change-wi-lifecycle-hydration-wi-3347"
__aw_work_item__ = "3347"


def hydrate_from_raw_payload(payload: dict[str, Any]) -> tuple[ChangeLifecycle | None, dict[str, Any]]:
    """R9: Hydrate ChangeLifecycle from persisted raw payload or handle legacy loop-state.

    Returns (hydrated_lifecycle, causal_lifecycle_projection).

    Note on Migration & Zero-Head Boundary (Defect 4 Repair):
    The legacy <!-- aw:loop-state ... --> case has no causal ledger, so its read model
    MUST use ledger.head_event_id: null and ledger.epoch: 0. It returns a read-only, non-terminal
    migration remediation snapshot with null artifact revisions, owner='migration', and existing
    diagnostic command 'aw wi validate <slug>'. Any actual migration write or ledger generation
    belongs to a later bounded leaf work-item (#3359-#3361); #3347 does not introduce any new public
    mutation grammar (e.g. 'aw wi migrate').
    """
    slug = str(payload.get("slug", payload.get("id", "3347")))
    body_text = str(payload.get("body", ""))

    # Defect 4 Repair: Real legacy loop-state detection with zero-head (null head_event_id and epoch 0)
    if "<!-- aw:loop-state" in body_text:
        legacy_projection = {
            "schema": "aw.change-lifecycle.v1",
            "wi_revision": None,
            "ec_revision": None,
            "td_revision": None,
            "cb_revision": None,
            "ledger": {
                "head_event_id": None,
                "epoch": 0,
            },
            "evidence": [],
            # There is no causal event or revision to name yet.  The next
            # migration-owned diagnostic is the sole remediation witness.
            "invalidations": [],
            "iteration": 1,
            "next": {
                "command": f"aw wi validate {slug}",
                "owner": OwnerVocabulary.MIGRATION.value,
            },
            "terminal": False,
        }
        return (None, legacy_projection)

    # Malformed or missing ledger payload fails closed to zero-head remediation
    ledger_data = payload.get("causal_ledger")
    if not isinstance(ledger_data, dict) or ledger_data.get("schema") != "aw.change-lifecycle.v1":
        fail_closed_projection = {
            "schema": "aw.change-lifecycle.v1",
            "wi_revision": None,
            "ec_revision": None,
            "td_revision": None,
            "cb_revision": None,
            "ledger": {
                "head_event_id": None,
                "epoch": 0,
            },
            "evidence": [],
            "invalidations": [],
            "iteration": 1,
            "next": {
                "command": f"aw wi validate {slug}",
                "owner": OwnerVocabulary.WI.value,
            },
            "terminal": False,
        }
        return (None, fail_closed_projection)

    # Parse hydrated active revisions and typed CausalParent (id, digest)
    active_revs: dict[ArtifactKind, ArtifactRevision | None] = {
        ArtifactKind.WI: None,
        ArtifactKind.EC: None,
        ArtifactKind.TD: None,
        ArtifactKind.CB: None,
    }

    for kind_enum in ArtifactKind:
        kind_str = kind_enum.value
        rev_dict = ledger_data.get(f"{kind_str}_revision")
        if isinstance(rev_dict, dict):
            parents_list: list[CausalParent] = []
            for p_dict in rev_dict.get("parents", []):
                if isinstance(p_dict, dict):
                    parents_list.append(CausalParent(revision_id=str(p_dict["id"]), digest=str(p_dict["digest"])))

            active_revs[kind_enum] = ArtifactRevision(
                id=str(rev_dict["id"]),
                kind=kind_enum,
                digest=str(rev_dict["digest"]),
                parents=tuple(parents_list),
                iteration=int(ledger_data.get("iteration", 1)),
            )

    events: list[LifecycleEvent] = []
    for evt_dict in ledger_data.get("events", []):
        t_dict = evt_dict.get("bound_tuple", {})
        b_tuple = ActiveDigestTuple(
            wi_digest=t_dict.get("wi_digest"),
            ec_digest=t_dict.get("ec_digest"),
            td_digest=t_dict.get("td_digest"),
            cb_digest=t_dict.get("cb_digest"),
        )
        cand_dict = evt_dict.get("candidate_revision", {})
        cand_parents = tuple(
            CausalParent(revision_id=str(p["id"]), digest=str(p["digest"]))
            for p in cand_dict.get("parents", [])
            if isinstance(p, dict)
        )
        cand_rev = ArtifactRevision(
            id=str(cand_dict.get("id", evt_dict.get("revision_id", ""))),
            kind=ArtifactKind(cand_dict.get("kind", evt_dict.get("kind", "wi").split("_")[0])),
            digest=str(cand_dict.get("digest", "")),
            parents=cand_parents,
            iteration=int(cand_dict.get("iteration", 1)),
        )
        events.append(
            LifecycleEvent(
                event_id=str(evt_dict["event_id"]),
                predecessor_id=evt_dict.get("predecessor_id"),
                kind=LifecycleEventKind(evt_dict["kind"]),
                candidate_revision=cand_rev,
                bound_tuple=b_tuple,
                next_command=str(evt_dict["next_command"]),
                next_owner=OwnerVocabulary(evt_dict["next_owner"]),
            )
        )

    evidence_list: list[EvidenceBinding] = []
    for ev_dict in ledger_data.get("evidence", []):
        t_dict = ev_dict.get("bound_tuple", {})
        b_tuple = ActiveDigestTuple(
            wi_digest=t_dict.get("wi_digest"),
            ec_digest=t_dict.get("ec_digest"),
            td_digest=t_dict.get("td_digest"),
            cb_digest=t_dict.get("cb_digest"),
        )
        evidence_list.append(
            EvidenceBinding(
                verifier=str(ev_dict["verifier"]),
                bound_tuple=b_tuple,
                passed=bool(ev_dict.get("passed", True)),
                summary=str(ev_dict.get("summary", "")),
            )
        )

    invalidation_records: list[InvalidationRecord] = []
    for inv_dict in ledger_data.get("invalidations", []):
        if isinstance(inv_dict, dict):
            invalidation_records.append(
                InvalidationRecord(
                    trigger_revision_id=str(inv_dict.get("trigger_revision_id", "")),
                    trigger_kind=ArtifactKind(inv_dict.get("trigger_kind", "wi")),
                    invalidated_kinds=tuple(ArtifactKind(k) for k in inv_dict.get("invalidated_kinds", [])),
                    invalidated_revision_ids=tuple(inv_dict.get("invalidated_revision_ids", [])),
                    evicted_evidence_verifiers=tuple(inv_dict.get("evicted_evidence_verifiers", [])),
                    reason=str(inv_dict.get("reason", "")),
                )
            )

    next_dict = ledger_data.get("next", {})
    next_ob = NextObligation(
        command=str(next_dict.get("command", f"aw wi validate {slug}")),
        owner=OwnerVocabulary(next_dict.get("owner", OwnerVocabulary.WI.value)),
    )

    hydrated_lifecycle = ChangeLifecycle(
        slug=slug,
        epoch=int(ledger_data["ledger"]["epoch"]),
        head_event_id=ledger_data["ledger"]["head_event_id"],
        active_revisions=active_revs,
        events=tuple(events),
        evidence_bindings=tuple(evidence_list),
        invalidations=tuple(invalidation_records),
        iteration=int(ledger_data.get("iteration", 1)),
        terminal=bool(ledger_data.get("terminal", False)),
        next_obligation=next_ob,
    )

    projection = render_causal_lifecycle_projection(hydrated_lifecycle)
    return (hydrated_lifecycle, projection)


def render_causal_lifecycle_projection(lifecycle: ChangeLifecycle) -> dict[str, Any]:
    """Render the official aw.change-lifecycle.v1 JSON read model projection on aw wi show."""

    def format_rev(kind: ArtifactKind) -> dict[str, Any] | None:
        rev = lifecycle.active_revisions.get(kind)
        if rev is None:
            return None
        return rev.to_dict()

    return {
        "schema": "aw.change-lifecycle.v1",
        "wi_revision": format_rev(ArtifactKind.WI),
        "ec_revision": format_rev(ArtifactKind.EC),
        "td_revision": format_rev(ArtifactKind.TD),
        "cb_revision": format_rev(ArtifactKind.CB),
        "ledger": {
            "head_event_id": lifecycle.head_event_id,
            "epoch": lifecycle.epoch,
        },
        "evidence": [e.to_dict() for e in lifecycle.evidence_bindings],
        "invalidations": [inv.to_dict() for inv in lifecycle.invalidations],
        "iteration": lifecycle.iteration,
        "next": lifecycle.next_obligation.to_dict(),
        "terminal": lifecycle.terminal,
    }


def design_contract() -> str:
    """Express executable design contract for hydration, legacy handling, and read model."""

    # Vector 7 (Defect 4 Repair): Legacy loop-state body block returns zero-head migration remediation
    legacy_payload = {
        "slug": "3347",
        "body": "<!-- aw:loop-state\nversion: 1\nstatus: iterating\n-->\n\n## Problem\nLegacy body",
        "loop_state": {"version": 1, "status": "iterating"},
    }
    lifecycle_leg, snap_leg = hydrate_from_raw_payload(legacy_payload)
    assert lifecycle_leg is None
    assert snap_leg["schema"] == "aw.change-lifecycle.v1"
    assert snap_leg["wi_revision"] is None
    assert snap_leg["ec_revision"] is None
    # Zero-head verification
    assert snap_leg["ledger"]["head_event_id"] is None
    assert snap_leg["ledger"]["epoch"] == 0
    assert snap_leg["terminal"] is False
    assert snap_leg["next"]["owner"] == OwnerVocabulary.MIGRATION.value
    assert snap_leg["next"]["command"] == "aw wi validate 3347"

    # Vector 8 (Defect 5 Repair): Valid payload hydration round-trip with CausalParent (id, digest)
    valid_payload = {
        "slug": "3347",
        "causal_ledger": {
            "schema": "aw.change-lifecycle.v1",
            "slug": "3347",
            "iteration": 1,
            "wi_revision": {"id": "rev-wi-1", "digest": "dig-wi-1", "parents": []},
            "ec_revision": {
                "id": "rev-ec-1",
                "digest": "dig-ec-1",
                "parents": [{"id": "rev-wi-1", "digest": "dig-wi-1"}],
            },
            "td_revision": None,
            "cb_revision": None,
            "ledger": {"head_event_id": "evt-001", "epoch": 1},
            "events": [
                {
                    "event_id": "evt-001",
                    "predecessor_id": None,
                    "kind": "wi_create",
                    "candidate_revision": {
                        "id": "rev-wi-1",
                        "kind": "wi",
                        "digest": "dig-wi-1",
                        "parents": [],
                        "iteration": 1,
                    },
                    "bound_tuple": {"wi_digest": "dig-wi-1"},
                    "next_command": "aw ec scaffold --wi 3347",
                    "next_owner": "wi",
                }
            ],
            "evidence": [],
            "invalidations": [],
            "next": {"command": "aw ec scaffold --wi 3347", "owner": "wi"},
            "terminal": False,
        },
    }

    lifecycle_valid, snap_valid = hydrate_from_raw_payload(valid_payload)
    assert lifecycle_valid is not None
    assert lifecycle_valid.head_event_id == "evt-001"
    assert lifecycle_valid.epoch == 1
    assert snap_valid["schema"] == "aw.change-lifecycle.v1"
    assert snap_valid["wi_revision"]["id"] == "rev-wi-1"
    assert snap_valid["ec_revision"]["parents"][0] == {"id": "rev-wi-1", "digest": "dig-wi-1"}
    assert snap_valid["td_revision"] is None
    assert snap_valid["next"]["owner"] == "wi"
    assert snap_valid["terminal"] is False

    # Named Rust test selector & Python EC
    # - Rust test: revisioned_change_wi_hydration
    # - Python EC: revisioned-change-wi-ledger
    return "ok"
