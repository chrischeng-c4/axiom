from __future__ import annotations

from collections.abc import Callable, Mapping
from enum import Enum

from cli_std.domain.errors import ChainableViolation


class NextLine(Enum):
    DONE = "done"
    COMMAND = "command"


JsonValue = Mapping[str, object]
ParseJson = Callable[[str], JsonValue | None]


def has_terminal_marker(value: JsonValue) -> bool:
    completion = value.get("completion")
    if isinstance(completion, Mapping):
        wf = completion.get("workflow_complete")
        if type(wf) is bool and wf is True:
            return True

    if value.get("status") == "done":
        return True

    if value.get("next") == "done":
        return True

    return False


def has_runnable_command(value: JsonValue) -> bool:
    nxt = value.get("next")
    if isinstance(nxt, str) and nxt.strip() != "" and nxt != "done":
        return True
    if isinstance(nxt, Mapping):
        cmd = nxt.get("command")
        if isinstance(cmd, str) and cmd.strip() != "":
            return True

    inv = value.get("invoke")
    if isinstance(inv, Mapping):
        cmd = inv.get("command")
        if isinstance(cmd, str) and cmd.strip() != "":
            return True

    return False


def trailing_next_line(text: str) -> NextLine | None:
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    if not lines:
        return None
    last = lines[-1]
    if last == "next: done":
        return NextLine.DONE
    if last.startswith("next: "):
        remainder = last[6:].strip()
        if remainder != "":
            return NextLine.COMMAND
    return None


def assert_chainable(
    output: str, parse_json: ParseJson
) -> ChainableViolation | None:
    trimmed = output.strip()
    if trimmed == "":
        return ChainableViolation(
            "output is empty - no JSON payload and no `next:` line"
        )

    value = parse_json(trimmed)
    if value is not None:
        if has_terminal_marker(value):
            return None
        if has_runnable_command(value):
            return None
        return ChainableViolation(
            "JSON payload has neither runnable command nor terminal marker"
        )

    match trailing_next_line(trimmed):
        case NextLine.DONE | NextLine.COMMAND:
            return None
        case _:
            return ChainableViolation(
                "output is not valid JSON and has no trailing `next:` marker"
            )
