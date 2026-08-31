#!/usr/bin/env python3
"""Block Codex spawn_agent calls that omit an explicit reasoning effort."""

from __future__ import annotations

import json
import re
import sys
from typing import Any


VALID_EFFORTS = frozenset({"low", "medium", "high", "xhigh", "max"})
BOUNDED_FORK_TURNS = re.compile(r"^[1-9][0-9]*$")


class DispatchPolicyError(ValueError):
    """The spawn_agent call does not satisfy the dispatch contract."""


def _is_spawn_agent(tool_name: Any) -> bool:
    return isinstance(tool_name, str) and (
        tool_name in {"spawn_agent", "Agent"}
        or tool_name.endswith(".spawn_agent")
    )


def validate_spawn_agent_call(payload: Any) -> None:
    if not isinstance(payload, dict):
        raise DispatchPolicyError("hook input must be one JSON object")

    if not _is_spawn_agent(payload.get("tool_name")):
        return

    tool_input = payload.get("tool_input")
    if not isinstance(tool_input, dict):
        raise DispatchPolicyError("spawn_agent tool_input must be one JSON object")

    effort = tool_input.get("reasoning_effort")
    if effort not in VALID_EFFORTS:
        allowed = "|".join(sorted(VALID_EFFORTS))
        raise DispatchPolicyError(
            f"reasoning_effort must be explicit and one of {allowed}"
        )

    fork_turns = tool_input.get("fork_turns")
    if fork_turns == "none":
        return
    if isinstance(fork_turns, str) and BOUNDED_FORK_TURNS.fullmatch(fork_turns):
        return
    raise DispatchPolicyError(
        "fork_turns must be 'none' or a positive integer string when "
        "reasoning_effort is explicit; full-history forks inherit effort"
    )


def main() -> int:
    try:
        payload = json.load(sys.stdin)
        validate_spawn_agent_call(payload)
    except (DispatchPolicyError, json.JSONDecodeError) as exc:
        print(f"Codex subagent dispatch blocked: {exc}", file=sys.stderr)
        return 2
    except Exception as exc:  # Fail closed on validator defects or input drift.
        print(
            f"Codex subagent dispatch blocked: effort validator failed: {exc}",
            file=sys.stderr,
        )
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
