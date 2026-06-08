"""Postgres-shaped preview of Cue's future Mamba persistence surface."""

from __future__ import annotations

from typing import Any, Literal, Protocol, TypedDict


PgCommandKind = Literal["select", "insert", "update", "delete", "execute"]


class PgConfig(TypedDict, total=False):
    dsn: str
    schema: str
    application_name: str


class PgCommand(TypedDict):
    kind: PgCommandKind
    sql: str
    params: dict[str, Any]


class PgResult(TypedDict):
    rows: list[dict[str, Any]]
    row_count: int
    status: Literal["ok", "unavailable", "error"]
    error: str | None


class PgKit(Protocol):
    """Minimal Postgres boundary that future Mamba bindings must satisfy."""

    def execute(self, command: PgCommand) -> PgResult:
        """Execute one structured Postgres command."""


class RecordingPgKit:
    """Fixture PgKit that records structured commands for contract tests."""

    def __init__(self, config: PgConfig | None = None) -> None:
        self.config = config or {}
        self.commands: list[PgCommand] = []

    def execute(self, command: PgCommand) -> PgResult:
        self.commands.append(command)
        return {
            "rows": [],
            "row_count": 1 if command["kind"] in {"insert", "update", "delete", "execute"} else 0,
            "status": "ok",
            "error": None,
        }


class UnavailablePgKit:
    """Placeholder until the real Postgres-backed Mamba library is built in."""

    def __init__(self, config: PgConfig | None = None) -> None:
        self.config = config or {}

    def execute(self, command: PgCommand) -> PgResult:
        return {
            "rows": [],
            "row_count": 0,
            "status": "unavailable",
            "error": "pgkit is a contract preview; Postgres execution is not wired yet",
        }


def pg_insert(table: str, values: dict[str, Any]) -> PgCommand:
    columns = list(values.keys())
    placeholders = [f":{column}" for column in columns]
    return {
        "kind": "insert",
        "sql": f"insert into {table} ({', '.join(columns)}) values ({', '.join(placeholders)})",
        "params": values,
    }


def pg_update(table: str, values: dict[str, Any], where: dict[str, Any]) -> PgCommand:
    set_clause = ", ".join(f"{column} = :{column}" for column in values)
    where_params = {f"where_{column}": value for column, value in where.items()}
    where_clause = " and ".join(f"{column} = :where_{column}" for column in where)
    return {
        "kind": "update",
        "sql": f"update {table} set {set_clause} where {where_clause}",
        "params": {**values, **where_params},
    }
