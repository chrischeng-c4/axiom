"""Black-box contract for the CLI-owned `aw goal` verifiable-condition loop.

Drives the real `aw goal set` / `aw goal check` / `aw goal show` cycle against
a fixture project and a real, file-backed gate command (`test -f <marker>`),
proving the loop deterministically reports `blocked` while the marker is
absent, `done` once it appears (with the goal state then auto-cleared), and
`gave_up` once a separately recorded, permanently-failing goal exhausts its
`--budget-checks` -- the three terminal states the capability promises.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import final_json, project_fixture, run_aw

CASE_ID = "aw-core-client-aw-goal-verifiable-condition-loop"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "aw-goal-verifiable-condition-loop"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-aw-goal-verifiable-condition-loop"
)
ASSERTIONS = (
    "`aw goal set <intent> --gate <cmd>` records workspace-scoped state and "
    "`aw goal check <id>` deterministically reports `blocked` with the real "
    "failing gate's command/success/output while its file-backed condition "
    "is unmet, then reports `done` with workflow_complete=true and clears "
    "the goal the instant the same real gate command starts passing",
    "a goal recorded with `--budget-checks 1` against a permanently-failing "
    "gate reports `blocked` on its first check and deterministically "
    "reports `gave_up` (goal cleared, no gate re-run) on its second, "
    "proving budget exhaustion -- not just gate success -- is a real "
    "terminal outcome of the loop",
    "once a goal reaches a terminal state (done or gave_up) its state is "
    "actually discarded: `aw goal show` for that id fails afterward instead "
    "of silently reporting stale recorded state",
)


def verify() -> list[str]:
    with project_fixture() as root:
        # -- blocked -> done ------------------------------------------------
        set_result = final_json(
            run_aw(
                root,
                "goal",
                "set",
                "the ready marker file exists in the fixture root",
                "--gate",
                "test -f ready.marker",
                "--budget-checks",
                "5",
            )
        )
        assert set_result["status"] == "recorded", set_result
        assert set_result["action"] == "goal_set", set_result
        goal = set_result["goal"]
        assert goal["gates"] == ["test -f ready.marker"], goal
        assert goal["checks_run"] == 0, goal
        assert goal["budget_checks"] == 5, goal
        goal_id = goal["id"]
        assert goal_id, set_result

        blocked = final_json(run_aw(root, "goal", "check", goal_id))
        assert blocked["status"] == "blocked", blocked
        assert blocked["action"] == "goal_check_blocked", blocked
        assert blocked["completion"]["workflow_complete"] is False, blocked
        assert blocked["completion"]["missing"] == [
            "at least one gate still failing"
        ], blocked
        assert len(blocked["gates"]) == 1, blocked
        gate_report = blocked["gates"][0]
        assert gate_report["command"] == "test -f ready.marker", gate_report
        assert gate_report["success"] is False, gate_report
        assert blocked["goal"]["checks_run"] == 1, blocked
        assert blocked["next"]["command"] == f"aw goal check {goal_id}", blocked

        (root / "ready.marker").write_text("", encoding="utf-8")

        done = final_json(run_aw(root, "goal", "check", goal_id))
        assert done["status"] == "done", done
        assert done["action"] == "goal_check_done", done
        assert done["completion"]["workflow_complete"] is True, done
        assert done["completion"]["criteria"] == ["all recorded gates passed"], done
        assert len(done["gates"]) == 1, done
        assert done["gates"][0]["success"] is True, done
        assert done["next"]["kind"] == "done", done

        vanished = run_aw(root, "goal", "show", goal_id, expect_success=False)
        assert vanished.returncode != 0, vanished

        # -- blocked -> gave_up on exhausted budget --------------------------
        set_result_2 = final_json(
            run_aw(
                root,
                "goal",
                "set",
                "a condition that can never become true in this fixture",
                "--gate",
                "false",
                "--budget-checks",
                "1",
            )
        )
        goal_id_2 = set_result_2["goal"]["id"]
        assert goal_id_2 != goal_id, (goal_id, goal_id_2)

        first_check = final_json(run_aw(root, "goal", "check", goal_id_2))
        assert first_check["status"] == "blocked", first_check
        assert first_check["goal"]["checks_run"] == 1, first_check

        gave_up = final_json(run_aw(root, "goal", "check", goal_id_2))
        assert gave_up["status"] == "gave_up", gave_up
        assert gave_up["action"] == "goal_gave_up", gave_up
        assert gave_up["gates"] == [], gave_up
        assert gave_up["completion"]["workflow_complete"] is False, gave_up
        assert gave_up["completion"]["missing"] == ["budget/expiry exhausted"], gave_up
        assert gave_up["next"]["kind"] == "none", gave_up

        vanished_2 = run_aw(root, "goal", "show", goal_id_2, expect_success=False)
        assert vanished_2.returncode != 0, vanished_2

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
