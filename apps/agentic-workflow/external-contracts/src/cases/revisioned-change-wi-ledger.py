"""Python EC: causal lifecycle snapshot for a Change WI (#3347)."""

from __future__ import annotations

import hashlib
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
    "list evidence/invalidations, integer iteration, next.command with next.owner==wi, "
    "terminal false; every aw wi show call leaves the fixture tree byte-identical",
    "after aw wi update --body-file the first post-update show returns a strictly "
    "advanced head_event_id AND epoch with next.owner==wi; a second independent show "
    "returns a byte-identical snapshot; both shows are read-only",
    "a Change WI with a legacy <!-- aw:loop-state ... --> body has a parsed loop_state "
    "red last_result dict; its causal snapshot has schema aw.change-lifecycle.v1, "
    "terminal false, next.owner==migration, and next.command starting with aw ",
)

_CHANGE_BODY = (
    "## Goal\n\n"
    "When aw wi create is called, it creates a change work item with causal ledger snapshot.\n\n"
    "## How\n\n"
    "### Verified premises\n\n"
    "- apps/agentic-workflow/external-contracts/src/cases/revisioned-change-wi-ledger.py:41 proves the causal ledger snapshot.\n\n"
    "### Change points\n\n"
    "- apps/agentic-workflow/external-contracts/src/cases/revisioned-change-wi-ledger.py — rewrite inline change body to GHAN.\n\n"
    "### Frozen decisions\n\n"
    "Ledger verification and phase 2 body updates operate on causal lifecycle.\n\n"
    "## Acceptance\n\n"
    "| # | command | current | target | why it cannot hold by accident |\n"
    "|---|---------|---------|--------|--------------------------------|\n"
    "| 1 | `aw wi show` | null lifecycle | valid causal lifecycle snapshot | validates ledger state |\n\n"
    "### Negative control\n\n"
    "Under line 42 mutation the gate must go red restoring to sha256 0000000000000000000000000000000000000000000000000000000000000000\n\n"
    "## Never\n\n"
    "This addresses the worker implementing this work item, not the controller reviewing it.\n\n"
    "### Must not touch\n\n"
    "- apps/agentic-workflow/src/issues/ghan.rs — validator is fixed.\n\n"
    "### Must not do\n\n"
    "- Do not alter causal lifecycle snapshot checks.\n"
)

_LEGACY_LOOP_BODY = (
    "<!-- aw:loop-state\n"
    "version: 1\n"
    'issue_id: "placeholder"\n'
    "iterations:\n"
    '  - {n: 1, action: ec, outcome: "red:behavior", summary: "prior round"}\n'
    "last_result: !red {dimension: behavior, why: \"prior round\"}\n"
    "status: iterating\n"
    'next_action: "aw cb gen placeholder"\n'
    "tried: []\n"
    "-->\n\n"
    "## Goal\n\n"
    "When aw wi create is called with a legacy loop-state header, it creates a change work item with parsed loop state.\n\n"
    "## How\n\n"
    "### Verified premises\n\n"
    "- apps/agentic-workflow/external-contracts/src/cases/revisioned-change-wi-ledger.py:57 verifies legacy loop-state body.\n\n"
    "### Change points\n\n"
    "- apps/agentic-workflow/external-contracts/src/cases/revisioned-change-wi-ledger.py — rewrite legacy loop body to GHAN.\n\n"
    "### Frozen decisions\n\n"
    "HTML comment containing aw:loop-state is preserved.\n\n"
    "## Acceptance\n\n"
    "| # | command | current | target | why it cannot hold by accident |\n"
    "|---|---------|---------|--------|--------------------------------|\n"
    "| 1 | `aw wi validate` | unvalidated | validated legacy loop-state change | validates loop-state parsing |\n\n"
    "### Negative control\n\n"
    "Under line 58 mutation the gate must go red restoring to sha256 0000000000000000000000000000000000000000000000000000000000000000\n\n"
    "## Never\n\n"
    "This addresses the worker implementing this work item, not the controller reviewing it.\n\n"
    "### Must not touch\n\n"
    "- apps/agentic-workflow/src/issues/ghan.rs — validator is fixed.\n\n"
    "### Must not do\n\n"
    "- Do not remove the aw:loop-state HTML comment.\n"
)


def _tree_fp(root: Path) -> str:
    h = hashlib.sha256()
    for p in sorted(root.rglob("*")):
        if p.is_file():
            h.update(p.relative_to(root).as_posix().encode())
            h.update(p.read_bytes())
    return h.hexdigest()


def _show_readonly(root: Path, slug: str) -> dict:
    before = _tree_fp(root)
    result = show(root, slug)
    assert _tree_fp(root) == before, f"aw wi show {slug!r} mutated the fixture tree"
    return result


def _lifecycle(root: Path, slug: str) -> dict:
    result = _show_readonly(root, slug)
    assert "causal_lifecycle" in result, (
        f"causal_lifecycle absent from aw wi show {slug!r} -- "
        "expected failure on pre-#3347 binary"
    )
    return result["causal_lifecycle"]


def _assert_revision(rev: object, name: str) -> None:
    assert isinstance(rev, dict), f"{name} not a dict: {rev!r}"
    assert rev.get("id"), f"{name}.id empty: {rev!r}"
    assert rev.get("digest"), f"{name}.digest empty: {rev!r}"
    assert isinstance(rev.get("parents"), list), f"{name}.parents not a list: {rev!r}"


def verify() -> list[str]:
    with project_fixture() as root:
        # Phase 1: fresh Change WI has the approved v1 shape.
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

        # Phase 2: body update; read lifecycle only from show, never from update.
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
        assert ledger2.get("head_event_id") != ledger1.get("head_event_id"), snap2a
        assert ledger2.get("epoch", 0) > ledger1.get("epoch", 0), snap2a
        assert snap2a.get("next", {}).get("owner") == "wi", snap2a

        snap2b = _lifecycle(root, slug)
        assert snap2b == snap2a, (snap2b, snap2a)

        # Phase 3: legacy loop-state body — prove parsed red record then causal snap.
        legacy = create(
            root, "Legacy loop-state Change", "change", "--body", _LEGACY_LOOP_BODY
        )
        legacy_slug = legacy["slug"]
        final_json(run_aw(root, "wi", "validate", legacy_slug))

        legacy_shown = _show_readonly(root, legacy_slug)
        loop_st = legacy_shown.get("loop_state")
        assert isinstance(loop_st, dict), f"loop_state not a dict: {loop_st!r}"
        assert loop_st.get("last_result") == {
            "red": {"dimension": "behavior", "why": "prior round"}
        }, loop_st

        snap_leg = legacy_shown.get("causal_lifecycle")
        assert snap_leg is not None, legacy_shown
        assert snap_leg.get("schema") == "aw.change-lifecycle.v1", snap_leg
        assert snap_leg.get("terminal") is False, snap_leg
        for _rev in ("wi_revision", "ec_revision", "td_revision", "cb_revision"):
            assert snap_leg.get(_rev) is None, (_rev, snap_leg)
        assert snap_leg.get("ledger") == {"head_event_id": None, "epoch": 0}, snap_leg
        assert snap_leg.get("evidence") == [] and snap_leg.get("invalidations") == [] and snap_leg.get("iteration") == 1, snap_leg
        nxt_leg = snap_leg.get("next", {})
        assert nxt_leg.get("owner") == "migration", snap_leg
        assert nxt_leg.get("command") == f"aw wi validate {legacy_slug}", snap_leg

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify_case(CASE_ID, verify)
