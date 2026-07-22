"""SQLite-backed Todo domain operations."""

from __future__ import annotations

import sqlite3
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

VALID_PRIORITIES = {"low", "medium", "high"}


class ValidationError(ValueError):
    """Raised when a request payload does not describe a valid task."""


class NotFoundError(LookupError):
    """Raised when a requested task does not exist."""


@dataclass(frozen=True)
class Todo:
    id: int
    title: str
    completed: bool
    priority: str
    due_date: str | None
    created_at: str

    def to_dict(self) -> dict[str, Any]:
        return asdict(self)


class TodoStore:
    """Owns the SQLite schema and the task CRUD contract."""

    def __init__(self, db_path: str | Path) -> None:
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._initialize()

    def _connect(self) -> sqlite3.Connection:
        connection = sqlite3.connect(self.db_path)
        connection.row_factory = sqlite3.Row
        return connection

    def _initialize(self) -> None:
        with self._connect() as connection:
            connection.execute(
                """
                CREATE TABLE IF NOT EXISTS todos (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    title TEXT NOT NULL,
                    completed INTEGER NOT NULL DEFAULT 0,
                    priority TEXT NOT NULL DEFAULT 'medium',
                    due_date TEXT,
                    created_at TEXT NOT NULL
                )
                """
            )

    def list_todos(self) -> list[dict[str, Any]]:
        with self._connect() as connection:
            rows = connection.execute(
                """
                SELECT id, title, completed, priority, due_date, created_at
                FROM todos
                ORDER BY completed ASC,
                         CASE priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END,
                         created_at DESC
                """
            ).fetchall()
        return [self._row_to_todo(row).to_dict() for row in rows]

    def create_todo(self, payload: dict[str, Any]) -> dict[str, Any]:
        title = self._validate_title(payload.get("title"))
        priority = self._validate_priority(payload.get("priority", "medium"))
        due_date = self._validate_due_date(payload.get("due_date"))
        created_at = datetime.now(timezone.utc).isoformat()
        with self._connect() as connection:
            cursor = connection.execute(
                "INSERT INTO todos (title, priority, due_date, created_at) VALUES (?, ?, ?, ?)",
                (title, priority, due_date, created_at),
            )
            row = connection.execute(
                "SELECT id, title, completed, priority, due_date, created_at FROM todos WHERE id = ?",
                (cursor.lastrowid,),
            ).fetchone()
        return self._row_to_todo(row).to_dict()

    def update_todo(self, todo_id: int, payload: dict[str, Any]) -> dict[str, Any]:
        existing = self._get_row(todo_id)
        updates: dict[str, Any] = {}
        if "title" in payload:
            updates["title"] = self._validate_title(payload["title"])
        if "priority" in payload:
            updates["priority"] = self._validate_priority(payload["priority"])
        if "due_date" in payload:
            updates["due_date"] = self._validate_due_date(payload["due_date"])
        if "completed" in payload:
            if not isinstance(payload["completed"], bool):
                raise ValidationError("completed must be true or false")
            updates["completed"] = int(payload["completed"])

        if not updates:
            return self._row_to_todo(existing).to_dict()

        assignments = ", ".join(f"{column} = ?" for column in updates)
        with self._connect() as connection:
            connection.execute(
                f"UPDATE todos SET {assignments} WHERE id = ?",
                (*updates.values(), todo_id),
            )
        return self._row_to_todo(self._get_row(todo_id)).to_dict()

    def delete_todo(self, todo_id: int) -> None:
        with self._connect() as connection:
            cursor = connection.execute("DELETE FROM todos WHERE id = ?", (todo_id,))
        if cursor.rowcount == 0:
            raise NotFoundError(f"todo {todo_id} was not found")

    def _get_row(self, todo_id: int) -> sqlite3.Row:
        with self._connect() as connection:
            row = connection.execute(
                "SELECT id, title, completed, priority, due_date, created_at FROM todos WHERE id = ?",
                (todo_id,),
            ).fetchone()
        if row is None:
            raise NotFoundError(f"todo {todo_id} was not found")
        return row

    @staticmethod
    def _row_to_todo(row: sqlite3.Row) -> Todo:
        return Todo(
            id=row["id"],
            title=row["title"],
            completed=bool(row["completed"]),
            priority=row["priority"],
            due_date=row["due_date"],
            created_at=row["created_at"],
        )

    @staticmethod
    def _validate_title(value: Any) -> str:
        if not isinstance(value, str):
            raise ValidationError("title is required")
        title = value.strip()
        if not title:
            raise ValidationError("title cannot be empty")
        if len(title) > 160:
            raise ValidationError("title must be at most 160 characters")
        return title

    @staticmethod
    def _validate_priority(value: Any) -> str:
        if value not in VALID_PRIORITIES:
            raise ValidationError("priority must be low, medium, or high")
        return value

    @staticmethod
    def _validate_due_date(value: Any) -> str | None:
        if value in (None, ""):
            return None
        if not isinstance(value, str):
            raise ValidationError("due_date must be an ISO date")
        try:
            return datetime.strptime(value, "%Y-%m-%d").date().isoformat()
        except ValueError as error:
            raise ValidationError("due_date must use YYYY-MM-DD") from error
