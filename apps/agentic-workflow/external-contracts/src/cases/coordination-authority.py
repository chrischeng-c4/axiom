"""Black-box contract for AW-only coordination advancement authority."""

from __future__ import annotations

import json
from copy import deepcopy
from pathlib import Path

from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "coordination-authority"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "aw-only-coordination-authority"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case coordination-authority"
)
ASSERTIONS = (
    "only an event submitted against durably active AW dispatch state can advance",
    "all required gates must be satisfied, evidenced, cited, and durably retained",
    "identity mismatch and client-forged authority or gate satisfaction fail closed",
    "human decision evidence is durably recorded by AW before decision advancement",
)


def _documents(
    task_id: str,
    *,
    required_gates: list[str] | None = None,
    decision_authority: str = "aw",
) -> tuple[dict[str, object], dict[str, object], list[dict[str, object]]]:
    gate_ids = required_gates or ["gate:tests"]
    task = {
        "schema_version": "aw.coordination.v1",
        "kind": "task",
        "task_id": task_id,
        "workflow_root": "change:#2587",
        "dependencies": [],
        "required_gates": gate_ids,
    }
    dispatch = {
        "schema_version": "aw.coordination.v1",
        "kind": "dispatch",
        "task_id": task_id,
        "dispatch_id": f"dispatch:{task_id}:1",
        "attempt": 1,
        "assignee": "agent:worker",
        "authority": "aw",
        "status": "active",
    }
    gates = [
        {
            "schema_version": "aw.coordination.v1",
            "kind": "gate",
            "gate_id": gate_id,
            "task_id": task_id,
            "gate_type": (
                "decision" if gate_id.startswith("gate:approval") else "evidence"
            ),
            "status": "pending",
            "authority": (
                decision_authority
                if gate_id.startswith("gate:approval")
                else "aw"
            ),
            "evidence": [],
        }
        for gate_id in gate_ids
    ]
    return task, dispatch, gates


def _event(
    task_id: str,
    message_type: str,
    evidence: list[str],
    *,
    dispatch_id: str | None = None,
    body: dict[str, object] | None = None,
) -> dict[str, object]:
    return {
        "schema_version": "aw.coordination.v1",
        "kind": "message",
        "event_id": f"event:{task_id}:{message_type}",
        "task_id": task_id,
        "dispatch_id": dispatch_id or f"dispatch:{task_id}:1",
        "sequence": 1,
        "sender": "agent:worker",
        "message_type": message_type,
        "evidence": evidence,
        "body": body or {},
    }


def _write(root: Path, name: str, value: object) -> Path:
    path = root / f"{name}.json"
    path.write_text(json.dumps(value), encoding="utf-8")
    return path


def _open(
    root: Path,
    name: str,
    task: dict[str, object],
    dispatch: dict[str, object],
    gates: list[dict[str, object]],
    *,
    expect_success: bool = True,
) -> dict[str, object]:
    completed = run_aw(
        root,
        "coordination",
        "open",
        "--task",
        str(_write(root, f"{name}-task", task)),
        "--dispatch",
        str(_write(root, f"{name}-dispatch", dispatch)),
        "--gates",
        str(_write(root, f"{name}-gates", gates)),
        expect_success=expect_success,
    )
    return final_json(completed)


def _submit(
    root: Path,
    name: str,
    task_id: str,
    event: dict[str, object],
    *,
    expect_success: bool = True,
) -> dict[str, object]:
    return final_json(
        run_aw(
            root,
            "coordination",
            "submit",
            task_id,
            "--event",
            str(_write(root, f"{name}-event", event)),
            expect_success=expect_success,
        )
    )


def _satisfy(root: Path, task_id: str, gate_id: str, evidence: str) -> None:
    result = final_json(
        run_aw(
            root,
            "coordination",
            "satisfy-gate",
            task_id,
            "--gate",
            gate_id,
            "--evidence",
            evidence,
        )
    )
    assert result["status"] == "satisfied"


def _show(root: Path, task_id: str) -> dict[str, object]:
    return final_json(run_aw(root, "coordination", "show", task_id))


def verify() -> list[str]:
    with project_fixture() as root:
        task, dispatch, gates = _documents("task:complete")
        opened = _open(root, "complete", task, dispatch, gates)
        assert opened["authority"] == "aw"
        assert opened["status"] == "open"

        # Mutating client-owned source files after open cannot alter AW state.
        dispatch["status"] = "interrupted"
        gates[0]["status"] = "satisfied"
        gates[0]["evidence"] = ["forged"]

        pending = _submit(
            root,
            "pending",
            "task:complete",
            _event("task:complete", "completion", ["gate:tests"]),
        )
        assert pending["completion_advanced"] is False
        assert "not satisfied" in pending["reason"]

        _satisfy(root, "task:complete", "gate:tests", "evidence:test-run")
        completed = _submit(
            root,
            "complete",
            "task:complete",
            _event("task:complete", "completion", ["gate:tests"]),
        )
        assert completed["authority"] == "aw"
        assert completed["completion_advanced"] is True
        assert completed["decision_advanced"] is False
        assert completed["terminal"] is True
        assert _show(root, "task:complete")["completion_advanced"] is True

        inactive_task, inactive_dispatch, inactive_gates = _documents(
            "task:inactive"
        )
        _open(
            root,
            "inactive",
            inactive_task,
            inactive_dispatch,
            inactive_gates,
        )
        _satisfy(
            root,
            "task:inactive",
            "gate:tests",
            "evidence:inactive-test-run",
        )
        interrupted = final_json(
            run_aw(
                root,
                "coordination",
                "interrupt",
                "task:inactive",
                "--reason",
                "worker-lost",
            )
        )
        assert interrupted["dispatch_status"] == "interrupted"
        assert _show(root, "task:inactive")["dispatch"]["status"] == "interrupted"
        rejected_inactive = _submit(
            root,
            "inactive",
            "task:inactive",
            _event("task:inactive", "completion", ["gate:tests"]),
        )
        assert rejected_inactive["completion_advanced"] is False
        assert "active dispatch" in rejected_inactive["reason"]

        # A client cannot establish a non-AW or already-satisfied dispatch.
        forged_task, forged_dispatch, forged_gates = _documents("task:forged")
        forged_dispatch["authority"] = "human"
        rejected_authority = _open(
            root,
            "forged-authority",
            forged_task,
            forged_dispatch,
            forged_gates,
            expect_success=False,
        )
        assert rejected_authority["status"] == "rejected"

        forged_dispatch["authority"] = "aw"
        forged_gates[0]["status"] = "satisfied"
        forged_gates[0]["evidence"] = ["client:forged"]
        rejected_gate = _open(
            root,
            "forged-gate",
            forged_task,
            forged_dispatch,
            forged_gates,
            expect_success=False,
        )
        assert rejected_gate["status"] == "rejected"

        # Identity mismatch and incomplete evidence matrices remain blocked.
        matrix_task, matrix_dispatch, matrix_gates = _documents(
            "task:matrix", required_gates=["gate:tests", "gate:lint"]
        )
        _open(root, "matrix", matrix_task, matrix_dispatch, matrix_gates)
        empty_evidence = final_json(
            run_aw(
                root,
                "coordination",
                "satisfy-gate",
                "task:matrix",
                "--gate",
                "gate:tests",
                "--evidence",
                "",
                expect_success=False,
            )
        )
        assert empty_evidence["status"] == "rejected"
        _satisfy(root, "task:matrix", "gate:tests", "evidence:test-run")

        for name, event, reason, expect_success in (
            (
                "wrong-task",
                _event("task:other", "completion", ["gate:tests"]),
                "task identity",
                False,
            ),
            (
                "wrong-dispatch",
                _event(
                    "task:matrix",
                    "completion",
                    ["gate:tests"],
                    dispatch_id="dispatch:stale:1",
                ),
                "active dispatch",
                False,
            ),
            (
                "missing",
                _event("task:matrix", "completion", []),
                "required gate",
                True,
            ),
            (
                "wrong-gate",
                _event("task:matrix", "completion", ["gate:other"]),
                "required gate",
                True,
            ),
            (
                "partial",
                _event("task:matrix", "completion", ["gate:tests"]),
                "gate:lint",
                True,
            ),
        ):
            rejected = _submit(
                root,
                name,
                "task:matrix",
                event,
                expect_success=expect_success,
            )
            assert rejected["completion_advanced"] is False
            assert reason in rejected["reason"]
            if not expect_success:
                assert rejected["code"] == "stale_event"

        _satisfy(root, "task:matrix", "gate:lint", "evidence:lint-run")
        all_gates = _submit(
            root,
            "all-gates",
            "task:matrix",
            _event(
                "task:matrix",
                "completion",
                ["gate:tests", "gate:lint"],
            ),
        )
        assert all_gates["completion_advanced"] is True
        matrix_state = _show(root, "task:matrix")
        durable_evidence = {
            gate["gate_id"]: gate["evidence"] for gate in matrix_state["gates"]
        }
        assert durable_evidence == {
            "gate:tests": ["evidence:test-run"],
            "gate:lint": ["evidence:lint-run"],
        }

        # A blocked question cannot self-authorize a decision. Only AW's
        # decision command records concrete human evidence and advances it.
        decision_task, decision_dispatch, decision_gates = _documents(
            "task:decision",
            required_gates=["gate:approval"],
            decision_authority="human",
        )
        _open(
            root,
            "decision",
            decision_task,
            decision_dispatch,
            decision_gates,
        )
        blocked = _submit(
            root,
            "question",
            "task:decision",
            _event(
                "task:decision",
                "blocked_question",
                [],
                body={"question": "Approve deployment?"},
            ),
        )
        assert blocked["decision_advanced"] is False
        assert blocked["requires_hitl"] is True

        for name, choice, evidence in (
            ("empty-choice", "", "human:review-42"),
            ("empty-decision-evidence", "approved", ""),
        ):
            rejected_decision = final_json(
                run_aw(
                    root,
                    "coordination",
                    "decide",
                    "task:decision",
                    "--gate",
                    "gate:approval",
                    "--choice",
                    choice,
                    "--evidence",
                    evidence,
                    expect_success=False,
                )
            )
            assert rejected_decision["decision_advanced"] is False, name
            assert rejected_decision["status"] == "rejected", name
            assert _show(root, "task:decision")["decision"] is None, name

        decided = final_json(
            run_aw(
                root,
                "coordination",
                "decide",
                "task:decision",
                "--gate",
                "gate:approval",
                "--choice",
                "approved",
                "--evidence",
                "human:review-42",
            )
        )
        assert decided["authority"] == "aw"
        assert decided["decision_advanced"] is True
        durable = _show(root, "task:decision")
        assert durable["decision"]["choice"] == "approved"
        assert durable["decision"]["evidence"] == "human:review-42"

        aw_decision_task, aw_decision_dispatch, aw_decision_gates = _documents(
            "task:forged-decision",
            required_gates=["gate:approval"],
            decision_authority="aw",
        )
        _open(
            root,
            "forged-decision",
            aw_decision_task,
            aw_decision_dispatch,
            aw_decision_gates,
        )
        forged_decision = final_json(
            run_aw(
                root,
                "coordination",
                "decide",
                "task:forged-decision",
                "--gate",
                "gate:approval",
                "--choice",
                "approved",
                "--evidence",
                "client:forged",
            )
        )
        assert forged_decision["decision_advanced"] is False
        assert "human authority" in forged_decision["reason"]

    return list(ASSERTIONS)
