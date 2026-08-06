"""Black-box contract for the WorkItem loop-state model (#3307).

Drives the real `aw ec record` verb -- the first-class CLI mechanism that
writes the `<!-- aw:loop-state ... -->` block into a WorkItem's body -- twice
in a row against a real local-backend WorkItem, then independently
cross-reads `aw wi show` after each write. Proves the loop state round-trips
without loss: a later verification round appends a new iteration while the
prior round's iteration survives byte-identical, and `status`/`next_action`
transition according to the real red/green decision rule.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import create, final_json, project_fixture, run_aw, show

CASE_ID = "aw-core-client-workitem-loop-state-model"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "workitem-loop-state-model"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-workitem-loop-state-model"
)
ASSERTIONS = (
    "a WorkItem with no `aw ec record` history reports loop_state: null from "
    "`aw wi show`, then a real `aw ec record --result red --dimension "
    "behavior` writes a genuine `<!-- aw:loop-state -->` block whose "
    "iterations/last_result/status/next_action are independently readable, "
    "byte-identical, from a separate `aw wi show` process, proving the loop "
    "state is durable WorkItem-body state rather than command-local output",
    "a second `aw ec record --result green` on the same WorkItem appends "
    "iteration n=2 while leaving iteration n=1 completely unchanged "
    "(identical action/outcome/summary across both the record response and "
    "an independent `aw wi show` re-read), proving the round-trip loses no "
    "prior history, and status/next_action transition from "
    "iterating/`aw cb gen <id>` (red) to converged/`aw cb check <id>` "
    "(green) exactly as the real decision rule prescribes",
)


def verify() -> list[str]:
    with project_fixture() as root:
        change = create(root, "Loop state round trip target", "change")
        slug = change["slug"]

        before = show(root, slug)
        assert before.get("loop_state") is None, before.get("loop_state")

        rec1 = final_json(
            run_aw(
                root,
                "ec",
                "record",
                "--project",
                "demo",
                "--wi",
                slug,
                "--result",
                "red",
                "--dimension",
                "behavior",
                "--summary",
                "round 1",
                "--json",
            )
        )
        state1 = rec1["loop_state"]
        assert state1["version"] == 1, state1
        assert state1["issue_id"] == slug, state1
        assert state1["iterations"] == [
            {"n": 1, "action": "ec", "outcome": "red:behavior", "summary": "round 1"}
        ], state1["iterations"]
        assert state1["last_result"] == {"red": {"dimension": "behavior", "why": "round 1"}}, state1
        assert state1["status"] == "iterating", state1
        assert state1["next_action"] == f"aw cb gen {slug}", state1
        assert state1["tried"] == [], state1

        shown1 = show(root, slug)
        assert shown1["loop_state"] == state1, (shown1["loop_state"], state1)

        rec2 = final_json(
            run_aw(
                root,
                "ec",
                "record",
                "--project",
                "demo",
                "--wi",
                slug,
                "--result",
                "green",
                "--summary",
                "round 2",
                "--json",
            )
        )
        state2 = rec2["loop_state"]
        assert state2["version"] == 1, state2
        assert state2["issue_id"] == slug, state2
        assert state2["iterations"][0] == state1["iterations"][0], (
            state2["iterations"][0],
            state1["iterations"][0],
        )
        assert state2["iterations"][1] == {
            "n": 2,
            "action": "ec",
            "outcome": "green",
            "summary": "round 2",
        }, state2["iterations"]
        assert len(state2["iterations"]) == 2, state2["iterations"]
        assert state2["last_result"] == "green", state2
        assert state2["status"] == "converged", state2
        assert state2["next_action"] == f"aw cb check {slug}", state2

        shown2 = show(root, slug)
        assert shown2["loop_state"] == state2, (shown2["loop_state"], state2)
        assert shown2["loop_state"]["iterations"][0] == shown1["loop_state"]["iterations"][0]

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
