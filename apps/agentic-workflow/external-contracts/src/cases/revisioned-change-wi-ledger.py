"""Python EC: causal lifecycle snapshot for a Change WI (#3347)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import (
    create,
    final_json,
    project_fixture,
    run_aw,
    show,
    verify_case,
)

CASE_ID = "revisioned-change-wi-ledger"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-loop-state-model"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case revisioned-change-wi-ledger"
)
ASSERTIONS = (
    "a fresh Change WI shows schema==aw.change-lifecycle.v1, a typed wi_revision "
    "with id/digest/parents (list), null ec/td/cb revisions, ledger head_event_id/epoch, "
    "list evidence/invalidations, integer iteration, one next.command and next.owner==wi, terminal false",
    "after aw wi update --body-file the first post-update show returns an updated "
    "wi_revision digest and ledger head_event_id/epoch; a second independent show "
    "returns a byte-identical snapshot; next.owner is exactly wi",
    "a Change WI with a legacy <!-- aw:loop-state ... --> body has a real loop_state "
    "dict from show, and its causal snapshot has terminal false and one next.command",
)

_CHANGE_BODY = (
    "## Problem\n\nProve the causal ledger snapshot.\n\n"
    "## Capability Alignment\n\nCapability: WI lifecycle\n"
    "Capability Gap: none\nProgress Evidence: causal_lifecycle snapshot\n\n"
    "## Requirements\n\n- R1: ledger snapshot present.\n\n"
    "## Scope\n\n### In Scope\n- ledger verification\n\n"
    "### Out of Scope\n- unrelated lifecycle stages\n\n"
    "## Acceptance Criteria\n\n- AC1: causal_lifecycle non-null.\n\n"
    "## Reference Context\n\n### Related Specs\n"
    "| Spec | Relevance |\n|------|-----------|\n"
    "| complete-platform.md | environment |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
    "|---------|--------|---------------|\n"
    "| ledger-trace | update | complete-platform.md |\n"
)

_LEGACY_LOOP_BODY = (
    "<!-- aw:loop-state\n"
    "version: 1\n"
    'issue_id: "placeholder"\n'
    "iterations:\n"
    '  - {n: 1, action: ec, outcome: "red:behavior", summary: "prior round"}\n'
    "last_result: {red: {dimension: behavior, why: \"prior round\"}}\n"
    "status: iterating\n"
    'next_action: "aw cb gen placeholder"\n'
    "tried: []\n"
    "-->\n\n"
    "## Problem\n\nLegacy loop-state Change.\n\n"
    "## Capability Alignment\n\nCapability: WI lifecycle\n"
    "Capability Gap: none\nProgress Evidence: loop-state body\n\n"
    "## Requirements\n\n- R1: legacy body accepted.\n\n"
    "## Scope\n\n### In Scope\n- legacy body\n\n"
    "### Out of Scope\n- new ledger\n\n"
    "## Acceptance Criteria\n\n- AC1: snapshot non-null.\n\n"
    "## Reference Context\n\n### Related Specs\n"
    "| Spec | Relevance |\n|------|-----------|\n"
    "| complete-platform.md | environment |\n\n"
    "### Spec Plan\n| Spec ID | Action | Main Spec Ref |\n"
    "|---------|--------|---------------|\n"
    "| legacy-trace | update | complete-platform.md |\n"
)


def _lifecycle(root: Path, slug: str) -> dict:
    snapshot = show(root, slug)
    assert "causal_lifecycle" in snapshot, (
        f"causal_lifecycle absent from aw wi show {slug!r} -- "
        "expected failure on pre-#3347 binary"
    )
    return snapshot["causal_lifecycle"]


def _assert_revision(rev: object, name: str) -> None:
    assert isinstance(rev, dict), f"{name} not a dict: {rev!r}"
    assert rev.get("id"), f"{name}.id empty: {rev!r}"
    assert rev.get("digest"), f"{name}.digest empty: {rev!r}"
    assert isinstance(rev.get("parents"), list), f"{name}.parents not a list: {rev!r}"


def verify() -> list[str]:
    with project_fixture() as root:
        change = create(root, "Causal ledger baseline", "change", "--body", _CHANGE_BODY)
        slug = change["slug"]
        final_json(run_aw(root, "wi", "validate", slug))

        snap1 = _lifecycle(root, slug)
        assert snap1.get("schema") == "aw.change-lifecycle.v1", snap1
        _assert_revision(snap1.get("wi_revision"), "wi_revision")
        assert snap1.get("ec_revision") is None, snap1
        assert snap1.get("td_revision") is None, snap1
        assert snap1.get("cb_revision") is None, snap1
        ledger1 = snap1.get("ledger", {})
        assert ledger1.get("head_event_id"), ledger1
        assert isinstance(ledger1.get("epoch"), int), ledger1
        assert isinstance(snap1.get("evidence"), list), snap1
        assert isinstance(snap1.get("invalidations"), list), snap1
        assert isinstance(snap1.get("iteration"), int), snap1
        nxt1 = snap1.get("next", {})
        assert isinstance(nxt1.get("command"), str) and nxt1["command"], snap1
        assert nxt1.get("owner") == "wi", snap1
        assert snap1.get("terminal") is False, snap1

        updated_path = root / "updated.md"
        updated_path.write_text(
            _CHANGE_BODY + "\n<!-- updated for ledger phase 2 -->\n",
            encoding="utf-8",
        )
        run_aw(root, "wi", "update", slug, "--body-file", str(updated_path))

        snap2a = _lifecycle(root, slug)
        wi_rev2 = snap2a.get("wi_revision", {})
        assert wi_rev2.get("digest") != snap1["wi_revision"].get("digest"), snap2a
        ledger2 = snap2a.get("ledger", {})
        assert (
            ledger2.get("head_event_id") != ledger1.get("head_event_id")
            or ledger2.get("epoch", 0) > ledger1.get("epoch", 0)
        ), snap2a
        assert snap2a.get("next", {}).get("owner") == "wi", snap2a

        snap2b = _lifecycle(root, slug)
        assert snap2b == snap2a, (snap2b, snap2a)

        legacy = create(
            root, "Legacy loop-state Change", "change", "--body", _LEGACY_LOOP_BODY
        )
        legacy_slug = legacy["slug"]
        final_json(run_aw(root, "wi", "validate", legacy_slug))
        assert isinstance(show(root, legacy_slug).get("loop_state"), dict), (
            "loop_state not a dict for legacy WI"
        )
        snap_leg = _lifecycle(root, legacy_slug)
        assert snap_leg.get("terminal") is False, snap_leg
        nxt_leg = snap_leg.get("next", {})
        assert isinstance(nxt_leg.get("command"), str) and nxt_leg["command"], snap_leg

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
