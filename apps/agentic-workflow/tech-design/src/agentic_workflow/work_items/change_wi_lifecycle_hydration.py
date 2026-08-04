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
    RUST_ISSUES_TEST_SEAMS,
)


__aw_changes__ = """
changes:
  - path: apps/agentic-workflow/src/cli/change_lifecycle.rs
    action: modify
    description: >
      Implement hydration reconstruction from append-only event ledger at .aw/causal-lifecycle/<slug>.json,
      legacy <!-- aw:loop-state ... --> zero-head migration detection, loader error mapping (absent, malformed, conflicting),
      and causal_lifecycle JSON serializer in Rust.
  - path: apps/agentic-workflow/src/cli/issues.rs
    action: modify
    description: >
      Expose causal_lifecycle (schema aw.change-lifecycle.v1) on read-only aw wi show <id> using a
      root-aware loader. Implement revisioned_change_wi_show_json_handler_is_carrier_byte_readonly
      against the same root-aware JSON projection call used by run_show: fingerprint the exact carrier
      before and after each of two handler calls. Implement revisioned_change_wi_fresh_binary_show_hydrates_existing_carrier
      by invoking a second aw wi show <slug> --json process on the same local root and proving it sees
      the stored head, epoch and active WI revision without a carrier rewrite.
  - path: apps/agentic-workflow/src/cli/mod.rs
    action: modify
    description: >
      Register change_lifecycle hydration and read-model rendering exports.
"""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/change-wi-lifecycle-hydration-wi-3347"
__aw_work_item__ = "3347"


def validate_persisted_lifecycle(
    lifecycle: ChangeLifecycle, requested_slug: str
) -> bool:
    """Concrete persisted-state validation before rendering valid state.

    Returns True if valid, False if conflicting.
    Asserts:
      1. requested_slug == lifecycle.slug
      2. If events non-empty, events[0].predecessor_id is None
      3. For each subsequent event i > 0, events[i].predecessor_id == events[i-1].event_id
      4. If events non-empty, lifecycle.head_event_id == events[-1].event_id
      5. Active WI revision matches candidate_revision of latest WI event (wi_create or wi_change)
    """
    if str(requested_slug) != str(lifecycle.slug):
        return False

    events = lifecycle.events
    if events:
        if events[0].predecessor_id is not None:
            return False
        for i in range(1, len(events)):
            if events[i].predecessor_id != events[i - 1].event_id:
                return False
        if lifecycle.head_event_id != events[-1].event_id:
            return False

        wi_events = [
            e for e in events
            if e.kind in (LifecycleEventKind.WI_CREATE, LifecycleEventKind.WI_CHANGE)
        ]
        if wi_events:
            latest_wi_event = wi_events[-1]
            active_wi = lifecycle.active_revisions.get(ArtifactKind.WI)
            if active_wi is None or active_wi != latest_wi_event.candidate_revision:
                return False
    else:
        if lifecycle.head_event_id is not None:
            return False

    return True


def load_and_hydrate_lifecycle(
    project_root: Any, slug: str, issue_body: str
) -> tuple[ChangeLifecycle | None, dict[str, Any], str]:
    """Root-aware loader that reads local ledger state at .aw/causal-lifecycle/<slug>.json.

    Classifies carrier state into:
      - 'legacy': body contains <!-- aw:loop-state ... --> (takes precedence, returns zero-head migration remediation)
      - 'absent': ledger file missing on disk (returns zero-head nonterminal wi remediation)
      - 'malformed': JSON corrupted or schema != aw.change-lifecycle.v1 (returns zero-head nonterminal wi remediation)
      - 'conflicting': syntactically valid record violating causal chain invariants (returns zero-head nonterminal wi remediation)
      - 'valid': valid JSON persistent record loaded and hydrated successfully
    """
    import json
    from pathlib import Path
    from .revisioned_change_wi_ledger import get_ledger_path

    slug_str = str(slug)

    # Legacy loop-state precedence check (zero-head migration remediation)
    if "<!-- aw:loop-state" in str(issue_body):
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
            "invalidations": [],
            "iteration": 1,
            "next": {
                "command": f"aw wi validate {slug_str}",
                "owner": OwnerVocabulary.MIGRATION.value,
            },
            "terminal": False,
        }
        return (None, legacy_projection, "legacy")

    ledger_path = get_ledger_path(project_root, slug_str)
    if not ledger_path.exists():
        fail_absent_projection = {
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
                "command": f"aw wi validate {slug_str}",
                "owner": OwnerVocabulary.WI.value,
            },
            "terminal": False,
        }
        return (None, fail_absent_projection, "absent")

    try:
        content = ledger_path.read_text(encoding="utf-8")
        payload = json.loads(content)
        if not isinstance(payload, dict) or payload.get("schema") != "aw.change-lifecycle.v1":
            raise ValueError("Invalid schema or payload shape")

        hydrated = ChangeLifecycle.from_persistent_dict(payload)
    except Exception:
        fail_malformed_projection = {
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
                "command": f"aw wi validate {slug_str}",
                "owner": OwnerVocabulary.WI.value,
            },
            "terminal": False,
        }
        return (None, fail_malformed_projection, "malformed")

    if not validate_persisted_lifecycle(hydrated, slug_str):
        fail_conflicting_projection = {
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
                "command": f"aw wi validate {slug_str}",
                "owner": OwnerVocabulary.WI.value,
            },
            "terminal": False,
        }
        return (None, fail_conflicting_projection, "conflicting")

    projection = render_causal_lifecycle_projection(hydrated)
    return (hydrated, projection, "valid")


def show_causal_lifecycle(
    project_root: Any, slug: str, issue_body: str
) -> dict[str, Any]:
    """Read-only helper for aw wi show <slug>.

    Loads, hydrates, and renders the causal_lifecycle projection without creating, updating,
    or writing to any disk file or carrier. Fresh-process show sees the exact same record.
    """
    _, projection, _ = load_and_hydrate_lifecycle(project_root, slug, issue_body)
    return projection


def hydrate_from_raw_payload(payload: dict[str, Any]) -> tuple[ChangeLifecycle | None, dict[str, Any]]:
    """R9: Hydrate ChangeLifecycle from persisted raw payload or handle legacy loop-state.

    Returns (hydrated_lifecycle, causal_lifecycle_projection).
    """
    slug = str(payload.get("slug", payload.get("id", "3347")))
    body_text = str(payload.get("body", ""))

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
            "invalidations": [],
            "iteration": 1,
            "next": {
                "command": f"aw wi validate {slug}",
                "owner": OwnerVocabulary.MIGRATION.value,
            },
            "terminal": False,
        }
        return (None, legacy_projection)

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

    if not validate_persisted_lifecycle(hydrated_lifecycle, slug):
        fail_conflicting_projection = {
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
        return (None, fail_conflicting_projection)

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
    import json
    import tempfile
    from pathlib import Path
    from .revisioned_change_wi_ledger import compute_digest, get_ledger_path, save_ledger_record

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
    assert snap_leg["ledger"]["head_event_id"] is None
    assert snap_leg["ledger"]["epoch"] == 0
    assert snap_leg["terminal"] is False
    assert snap_leg["next"]["owner"] == OwnerVocabulary.MIGRATION.value
    assert snap_leg["next"]["command"] == "aw wi validate 3347"

    # Test load_and_hydrate_lifecycle for legacy body
    with tempfile.TemporaryDirectory() as tmpdir:
        lc_leg, proj_leg, status_leg = load_and_hydrate_lifecycle(tmpdir, "3347", legacy_payload["body"])
        assert lc_leg is None
        assert status_leg == "legacy"
        assert proj_leg["next"]["owner"] == "migration"

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
                    "next_command": "aw ec draft 3347 --project agentic-workflow --wi 3347",
                    "next_owner": "wi",
                }
            ],
            "evidence": [],
            "invalidations": [],
            "next": {
                "command": "aw ec draft 3347 --project agentic-workflow --wi 3347",
                "owner": "wi",
            },
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
    valid_persistent_payload = lifecycle_valid.to_persistent_dict()

    # Test load_and_hydrate_lifecycle for absent file
    with tempfile.TemporaryDirectory() as tmpdir:
        lc_abs, proj_abs, status_abs = load_and_hydrate_lifecycle(tmpdir, "3347", "Ordinary body")
        assert lc_abs is None
        assert status_abs == "absent"
        assert proj_abs["next"]["owner"] == "wi"
        assert proj_abs["ledger"]["epoch"] == 0

    # Test load_and_hydrate_lifecycle for malformed file
    with tempfile.TemporaryDirectory() as tmpdir:
        bad_file = Path(tmpdir) / ".aw" / "causal-lifecycle" / "3347.json"
        bad_file.parent.mkdir(parents=True, exist_ok=True)
        bad_file.write_text("{corrupt json", encoding="utf-8")

        lc_mal, proj_mal, status_mal = load_and_hydrate_lifecycle(tmpdir, "3347", "Ordinary body")
        assert lc_mal is None
        assert status_mal == "malformed"
        assert proj_mal["next"]["owner"] == "wi"

    # Test load_and_hydrate_lifecycle for conflicting file (syntactically valid JSON with causal chain break)
    with tempfile.TemporaryDirectory() as tmpdir:
        bad_chain_file = get_ledger_path(tmpdir, "3347")
        bad_chain_file.parent.mkdir(parents=True, exist_ok=True)
        conflict_payload = json.loads(json.dumps(valid_persistent_payload))
        conflict_payload["head_event_id"] = "evt-wrong-head"
        bad_chain_file.write_text(json.dumps(conflict_payload, indent=2), encoding="utf-8")

        lc_conf, proj_conf, status_conf = load_and_hydrate_lifecycle(tmpdir, "3347", "Ordinary body")
        assert lc_conf is None
        assert status_conf == "conflicting"
        assert proj_conf["next"]["command"] == "aw wi validate 3347"
        assert proj_conf["next"]["owner"] == "wi"
        assert proj_conf["ledger"]["epoch"] == 0
        assert proj_conf["ledger"]["head_event_id"] is None
        assert proj_conf["terminal"] is False

    # A same revision id cannot mask divergent persisted candidate content.
    # The active WI remains rev-wi-1/dig-wi-1, but the latest WI event claims
    # that same id with a distinct digest.  This is a syntactically valid
    # record and must classify as conflicting, never valid.
    with tempfile.TemporaryDirectory() as tmpdir:
        mismatch_file = get_ledger_path(tmpdir, "3347")
        mismatch_file.parent.mkdir(parents=True, exist_ok=True)
        # Baseline must be a real carrier accepted by the exact loader under
        # test; a read-model-shaped fixture could fail before the equality
        # check and would not prove this invariant.
        mismatch_file.write_text(json.dumps(valid_persistent_payload, indent=2), encoding="utf-8")
        lc_baseline, _, status_baseline = load_and_hydrate_lifecycle(
            tmpdir, "3347", "Ordinary body"
        )
        assert lc_baseline is not None
        assert status_baseline == "valid"

        mismatch_payload = json.loads(json.dumps(valid_persistent_payload))
        mismatch_payload["events"][0]["candidate_revision"]["digest"] = "dig-wi-tampered"
        mismatch_file.write_text(json.dumps(mismatch_payload, indent=2), encoding="utf-8")

        lc_mismatch, proj_mismatch, status_mismatch = load_and_hydrate_lifecycle(
            tmpdir, "3347", "Ordinary body"
        )
        assert lc_mismatch is None
        assert status_mismatch == "conflicting"
        assert proj_mismatch["ledger"] == {"head_event_id": None, "epoch": 0}
        assert proj_mismatch["next"] == {"command": "aw wi validate 3347", "owner": "wi"}

    # Test load_and_hydrate_lifecycle for valid file & read-only show_causal_lifecycle with carrier byte fingerprinting
    with tempfile.TemporaryDirectory() as tmpdir:
        save_ledger_record(tmpdir, lifecycle_valid)
        carrier_path = get_ledger_path(tmpdir, "3347")
        bytes_before = carrier_path.read_bytes()
        digest_before = compute_digest(bytes_before.decode("utf-8"))

        lc_val, proj_val, status_val = load_and_hydrate_lifecycle(tmpdir, "3347", "Ordinary body")
        assert lc_val is not None
        assert status_val == "valid"
        assert proj_val["wi_revision"]["id"] == "rev-wi-1"

        # show_causal_lifecycle is byte-identical across multiple calls and fingerprints exact carrier bytes
        show_1 = show_causal_lifecycle(tmpdir, "3347", "Ordinary body")
        bytes_after1 = carrier_path.read_bytes()
        digest_after1 = compute_digest(bytes_after1.decode("utf-8"))
        assert digest_before == digest_after1

        show_2 = show_causal_lifecycle(tmpdir, "3347", "Ordinary body")
        bytes_after2 = carrier_path.read_bytes()
        digest_after2 = compute_digest(bytes_after2.decode("utf-8"))
        assert digest_before == digest_after2
        assert show_1 == show_2 == proj_val

    # Named actual-handler Rust test selectors & independent Python EC.
    # The pure helper checks above are intentionally supplemental: the CB is
    # required to prove both handler and fresh-process behavior from the
    # corresponding RUST_ISSUES_TEST_SEAMS entries.
    assert "revisioned_change_wi_show_json_handler_is_carrier_byte_readonly" in RUST_ISSUES_TEST_SEAMS
    assert "revisioned_change_wi_fresh_binary_show_hydrates_existing_carrier" in RUST_ISSUES_TEST_SEAMS
    # - Rust test: revisioned_change_wi_hydration
    # - Python EC: revisioned-change-wi-ledger
    return "ok"
