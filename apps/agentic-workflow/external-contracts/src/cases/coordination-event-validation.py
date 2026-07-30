"""Black-box rejection contract for AW coordination events."""

from __future__ import annotations

import json
import shlex
from pathlib import Path

from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "coordination-event-validation"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "fail-closed-coordination-event-validation"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case coordination-event-validation"
)
ASSERTIONS = (
    "stale sequence, duplicate event identity, and stale dispatch fail closed",
    "an event from anyone except the active assignee is unauthorised",
    "unknown version, missing/extra fields, empty identity, zero sequence, and invalid evidence reject structurally",
    "rejections do not persist or consume sequence and every remediation executes",
)

PUBLISHED_MESSAGE_SCHEMA = (
    Path(__file__).resolve().parents[5]
    / "apps"
    / "agentic-workflow"
    / "schemas"
    / "coordination"
    / "message.schema.json"
)


def _write(root: Path, name: str, value: object) -> Path:
    path = root / f"{name}.json"
    path.write_text(json.dumps(value), encoding="utf-8")
    return path


def _event(
    *,
    event_id: str,
    sequence: int,
    dispatch_id: str = "dispatch:events:1",
    sender: str = "agent:worker",
) -> dict[str, object]:
    return {
        "schema_version": "aw.coordination.v1",
        "kind": "message",
        "event_id": event_id,
        "task_id": "task:events",
        "dispatch_id": dispatch_id,
        "sequence": sequence,
        "sender": sender,
        "message_type": "heartbeat",
        "evidence": [],
        "body": {},
    }


def _submit(
    root: Path,
    name: str,
    event: dict[str, object],
    *,
    expect_success: bool,
) -> dict[str, object]:
    return final_json(
        run_aw(
            root,
            "coordination",
            "submit",
            "task:events",
            "--event",
            str(_write(root, name, event)),
            expect_success=expect_success,
        )
    )


def _run_remediation(root: Path, command: str) -> dict[str, object]:
    words = shlex.split(command)
    assert words[0] == "aw"
    return final_json(run_aw(root, *words[1:]))


def _assert_rejected(
    root: Path,
    name: str,
    event: dict[str, object],
    code: str,
    command: str,
) -> None:
    rejected = _submit(root, name, event, expect_success=False)
    assert rejected["status"] == "rejected", rejected
    assert rejected["code"] == code, rejected
    assert rejected["completion_advanced"] is False, rejected
    assert rejected["decision_advanced"] is False, rejected
    assert rejected["next"]["command"] == command, rejected
    remediation = _run_remediation(root, command)
    assert remediation["terminal"] is True


def verify() -> list[str]:
    with project_fixture() as root:
        task = {
            "schema_version": "aw.coordination.v1",
            "kind": "task",
            "task_id": "task:events",
            "workflow_root": "change:#2588",
            "dependencies": [],
            "required_gates": ["gate:tests"],
        }
        dispatch = {
            "schema_version": "aw.coordination.v1",
            "kind": "dispatch",
            "task_id": "task:events",
            "dispatch_id": "dispatch:events:1",
            "attempt": 1,
            "assignee": "agent:worker",
            "authority": "aw",
            "status": "active",
        }
        gates = [
            {
                "schema_version": "aw.coordination.v1",
                "kind": "gate",
                "gate_id": "gate:tests",
                "task_id": "task:events",
                "gate_type": "evidence",
                "status": "pending",
                "authority": "aw",
                "evidence": [],
            }
        ]
        opened = final_json(
            run_aw(
                root,
                "coordination",
                "open",
                "--task",
                str(_write(root, "task", task)),
                "--dispatch",
                str(_write(root, "dispatch", dispatch)),
                "--gates",
                str(_write(root, "gates", gates)),
            )
        )
        assert opened["status"] == "open"

        accepted_first = _submit(
            root,
            "accepted-1",
            _event(event_id="event:1", sequence=1),
            expect_success=True,
        )
        assert accepted_first["status"] == "recorded"

        show_command = "aw coordination show task:events"
        _assert_rejected(
            root,
            "stale-sequence",
            _event(event_id="event:stale", sequence=1),
            "stale_event",
            show_command,
        )
        _assert_rejected(
            root,
            "duplicate-event",
            _event(event_id="event:1", sequence=2),
            "stale_event",
            show_command,
        )
        _assert_rejected(
            root,
            "stale-dispatch",
            _event(
                event_id="event:stale-dispatch",
                sequence=2,
                dispatch_id="dispatch:events:0",
            ),
            "stale_event",
            show_command,
        )
        _assert_rejected(
            root,
            "unauthorised",
            _event(
                event_id="event:intruder",
                sequence=2,
                sender="agent:intruder",
            ),
            "unauthorised_event",
            show_command,
        )

        accepted_second = _submit(
            root,
            "accepted-2",
            _event(event_id="event:2", sequence=2),
            expect_success=True,
        )
        assert accepted_second["status"] == "recorded"

        schema_command = "aw coordination schema message"
        malformed: list[tuple[str, dict[str, object]]] = []

        unknown_version = _event(event_id="event:version", sequence=3)
        unknown_version["schema_version"] = "aw.coordination.v999"
        malformed.append(("unknown-version", unknown_version))

        missing_dispatch = _event(event_id="event:missing", sequence=3)
        missing_dispatch.pop("dispatch_id")
        malformed.append(("missing-field", missing_dispatch))

        extra_field = _event(event_id="event:extra", sequence=3)
        extra_field["client_authority"] = "aw"
        malformed.append(("extra-field", extra_field))

        zero_sequence = _event(event_id="event:zero", sequence=0)
        malformed.append(("zero-sequence", zero_sequence))

        empty_identity = _event(event_id="", sequence=3)
        malformed.append(("empty-event-id", empty_identity))

        empty_sender = _event(event_id="event:sender", sequence=3, sender="")
        malformed.append(("empty-sender", empty_sender))

        empty_evidence = _event(event_id="event:evidence", sequence=3)
        empty_evidence["evidence"] = [""]
        malformed.append(("empty-evidence", empty_evidence))

        duplicate_evidence = _event(event_id="event:duplicate-evidence", sequence=3)
        duplicate_evidence["evidence"] = ["gate:tests", "gate:tests"]
        malformed.append(("duplicate-evidence", duplicate_evidence))

        for name, event in malformed:
            _assert_rejected(
                root,
                name,
                event,
                "invalid_event",
                schema_command,
            )

        accepted_third = _submit(
            root,
            "accepted-3",
            _event(event_id="event:3", sequence=3),
            expect_success=True,
        )
        assert accepted_third["status"] == "recorded"

        schema = _run_remediation(root, schema_command)
        assert schema["document_kind"] == "message"
        assert schema["schema"] == json.loads(
            PUBLISHED_MESSAGE_SCHEMA.read_text(encoding="utf-8")
        )

        durable = _run_remediation(root, show_command)
        assert [event["event_id"] for event in durable["events"]] == [
            "event:1",
            "event:2",
            "event:3",
        ]
        assert [event["sequence"] for event in durable["events"]] == [
            1,
            2,
            3,
        ]
        assert durable["completion_advanced"] is False
        assert durable["decision"] is None

    return list(ASSERTIONS)
