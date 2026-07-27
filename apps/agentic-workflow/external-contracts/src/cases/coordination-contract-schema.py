"""Black-box contract for AW's client-independent coordination schemas."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


CASE_ID = "coordination-contract-schema"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "versioned-client-independent-coordination-contract"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case coordination-contract-schema"
)
ASSERTIONS = (
    "published task, dispatch, message, and gate fixtures round-trip",
    "every coordination document rejects an unknown protocol version",
    "every required field and every declared field type fail closed when mutated",
    "every const, enum, and unknown field fails closed when mutated",
    "messages represent heartbeat, completion, escalation, and blocked questions",
)

REPOSITORY_ROOT = Path(__file__).resolve().parents[5]
SCHEMA_ROOT = (
    REPOSITORY_ROOT
    / "apps"
    / "agentic-workflow"
    / "schemas"
    / "coordination"
)

FIXTURES: dict[str, dict[str, Any]] = {
    "task": {
        "schema_version": "aw.coordination.v1",
        "kind": "task",
        "task_id": "task:2586",
        "workflow_root": "change:#2586",
        "dependencies": ["task:2585"],
        "required_gates": ["gate:contract"],
    },
    "dispatch": {
        "schema_version": "aw.coordination.v1",
        "kind": "dispatch",
        "task_id": "task:2586",
        "dispatch_id": "dispatch:2586:1",
        "attempt": 1,
        "assignee": "agent:worker",
        "authority": "aw",
        "status": "active",
    },
    "message": {
        "schema_version": "aw.coordination.v1",
        "kind": "message",
        "event_id": "event:2586:1",
        "task_id": "task:2586",
        "dispatch_id": "dispatch:2586:1",
        "sequence": 1,
        "sender": "agent:worker",
        "message_type": "heartbeat",
        "evidence": [],
        "body": {},
    },
    "gate": {
        "schema_version": "aw.coordination.v1",
        "kind": "gate",
        "gate_id": "gate:contract",
        "task_id": "task:2586",
        "gate_type": "evidence",
        "status": "pending",
        "authority": "aw",
        "evidence": [],
    },
}

REQUIRED_FIELDS = {
    "task": {
        "schema_version",
        "kind",
        "task_id",
        "workflow_root",
        "dependencies",
        "required_gates",
    },
    "dispatch": {
        "schema_version",
        "kind",
        "task_id",
        "dispatch_id",
        "attempt",
        "assignee",
        "authority",
        "status",
    },
    "message": {
        "schema_version",
        "kind",
        "event_id",
        "task_id",
        "dispatch_id",
        "sequence",
        "sender",
        "message_type",
        "evidence",
        "body",
    },
    "gate": {
        "schema_version",
        "kind",
        "gate_id",
        "task_id",
        "gate_type",
        "status",
        "authority",
        "evidence",
    },
}


def _matches_type(value: Any, expected: str) -> bool:
    return {
        "object": isinstance(value, dict),
        "array": isinstance(value, list),
        "string": isinstance(value, str),
        "integer": isinstance(value, int) and not isinstance(value, bool),
        "number": isinstance(value, (int, float)) and not isinstance(value, bool),
        "boolean": isinstance(value, bool),
        "null": value is None,
    }[expected]


def _validate(schema: dict[str, Any], value: Any, path: str = "$") -> None:
    expected_type = schema.get("type")
    if isinstance(expected_type, list):
        assert any(_matches_type(value, item) for item in expected_type), path
    elif isinstance(expected_type, str):
        assert _matches_type(value, expected_type), path

    if "const" in schema:
        assert value == schema["const"], f"{path}: expected const"
    if "enum" in schema:
        assert value in schema["enum"], f"{path}: invalid enum"

    if isinstance(value, dict):
        properties = schema.get("properties", {})
        for required in schema.get("required", []):
            assert required in value, f"{path}: missing {required}"
        if schema.get("additionalProperties") is False:
            unknown = set(value) - set(properties)
            assert not unknown, f"{path}: unknown {sorted(unknown)}"
        for key, child in value.items():
            if key in properties:
                _validate(properties[key], child, f"{path}.{key}")

    if isinstance(value, list):
        if schema.get("uniqueItems"):
            encoded = [json.dumps(item, sort_keys=True) for item in value]
            assert len(encoded) == len(set(encoded)), f"{path}: duplicates"
        for index, child in enumerate(value):
            if "items" in schema:
                _validate(schema["items"], child, f"{path}[{index}]")

    if isinstance(value, str) and "minLength" in schema:
        assert len(value) >= schema["minLength"], f"{path}: too short"
    if isinstance(value, int) and "minimum" in schema:
        assert value >= schema["minimum"], f"{path}: below minimum"


def _rejected(schema: dict[str, Any], fixture: dict[str, Any]) -> bool:
    try:
        _validate(schema, fixture)
    except AssertionError:
        return True
    return False


def _wrong_type(expected: str | list[str]) -> Any:
    types = {expected} if isinstance(expected, str) else set(expected)
    for candidate, value in (
        ("null", None),
        ("boolean", True),
        ("integer", 7),
        ("string", "wrong-type"),
        ("array", []),
        ("object", {}),
    ):
        if candidate not in types:
            return value
    raise AssertionError(f"no wrong-type fixture for {expected}")


def verify() -> list[str]:
    schemas: dict[str, dict[str, Any]] = {}
    for kind, fixture in FIXTURES.items():
        path = SCHEMA_ROOT / f"{kind}.schema.json"
        schema = json.loads(path.read_text(encoding="utf-8"))
        assert schema["$schema"] == "https://json-schema.org/draft/2020-12/schema"
        assert schema["$id"].endswith(f"/{kind}.schema.json")
        assert set(schema["required"]) == REQUIRED_FIELDS[kind]
        assert set(schema["properties"]) == REQUIRED_FIELDS[kind]
        assert schema["additionalProperties"] is False
        _validate(schema, fixture)
        round_trip = json.loads(json.dumps(fixture, sort_keys=True))
        assert round_trip == fixture
        _validate(schema, round_trip)
        schemas[kind] = schema

        for required in REQUIRED_FIELDS[kind]:
            missing = dict(fixture)
            missing.pop(required)
            assert _rejected(schema, missing), f"{kind}.{required} was optional"

        for field, field_schema in schema["properties"].items():
            wrong_type = fixture | {
                field: _wrong_type(field_schema["type"])
            }
            assert _rejected(schema, wrong_type), f"{kind}.{field} was untyped"

            if "const" in field_schema:
                invalid_const = fixture | {field: "__unknown_const__"}
                assert _rejected(schema, invalid_const), (
                    f"{kind}.{field} ignored const"
                )
            if "enum" in field_schema:
                invalid_enum = fixture | {field: "__unknown_enum__"}
                assert _rejected(schema, invalid_enum), (
                    f"{kind}.{field} ignored enum"
                )

        unknown_field = fixture | {"client_private_state": True}
        assert _rejected(schema, unknown_field)

    message_types = set(
        schemas["message"]["properties"]["message_type"]["enum"]
    )
    assert message_types == {
        "heartbeat",
        "completion",
        "escalation",
        "blocked_question",
    }
    return list(ASSERTIONS)
