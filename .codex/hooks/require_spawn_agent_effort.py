#!/usr/bin/env python3
"""Block Codex spawn_agent calls that omit an explicit reasoning effort,
name an unregistered agent, or claim an effort that does not match the
named role's pinned `model_reasoning_effort`.

This mirrors `.claude/hooks/require_agent_effort.py`: the registry is the
role files themselves — `.codex/agents/*.toml` — so the dispatch marker and
the definition cannot drift apart silently. Parsing is line-regex rather
than tomllib because the system `python3` is 3.9.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, Optional

VALID_EFFORTS = frozenset({"low", "medium", "high", "xhigh", "max"})
BOUNDED_FORK_TURNS = re.compile(r"^[1-9][0-9]*$")
AGENTS_DIR = Path(__file__).resolve().parents[1] / "agents"
_NAME_LINE = re.compile(r'^name = "([^"]+)"$', re.MULTILINE)
_EFFORT_LINE = re.compile(r'^model_reasoning_effort = "([^"]+)"$', re.MULTILINE)


class DispatchPolicyError(ValueError):
    """The spawn_agent call does not satisfy the dispatch contract."""


def _is_spawn_agent(tool_name: Any) -> bool:
    return isinstance(tool_name, str) and (
        tool_name in {"spawn_agent", "Agent"}
        or tool_name.endswith(".spawn_agent")
    )


def load_registry(agents_dir: Path = AGENTS_DIR) -> Dict[str, str]:
    """Map registered agent name -> pinned reasoning effort."""
    registry: Dict[str, str] = {}
    for toml_path in sorted(agents_dir.glob("*.toml")):
        text = toml_path.read_text(encoding="utf-8")
        name = _NAME_LINE.search(text)
        effort = _EFFORT_LINE.search(text)
        if name and effort:
            registry[name.group(1)] = effort.group(1)
    return registry


def validate_spawn_agent_call(
    payload: Any, registry: Optional[Dict[str, str]] = None
) -> None:
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

    if registry is None:
        registry = load_registry()
    if not registry:
        raise DispatchPolicyError(
            f"no agent registry found under {AGENTS_DIR}"
        )
    agent_type = tool_input.get("agent_type")
    if not isinstance(agent_type, str) or agent_type not in registry:
        raise DispatchPolicyError(
            "agent_type must name a registered role in .codex/agents/; "
            f"got {agent_type!r}"
        )
    pinned = registry[agent_type]
    if effort != pinned:
        raise DispatchPolicyError(
            f"reasoning_effort {effort!r} does not match {agent_type!r} "
            f"pinned effort {pinned!r}"
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
