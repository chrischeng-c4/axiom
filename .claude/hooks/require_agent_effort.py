#!/usr/bin/env python3
"""Block Claude Code Agent calls that do not name their effective effort."""

from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict


VALID_EFFORTS = frozenset({"low", "medium", "high", "xhigh", "max"})
# The model aliases the Agent tool accepts, plus the one explicit fable id.
# A frontmatter `model:` outside this set silently falls back to the session
# model, so the pin would lie; refuse it instead.
VALID_MODELS = frozenset({"sonnet", "opus", "fable", "haiku", "claude-fable-5-1"})
EFFORT_MARKER = re.compile(
    r"^\[effort=(low|medium|high|xhigh|max)\](?:\s+|$)"
)


class DispatchPolicyError(ValueError):
    """The Agent call does not satisfy the dispatch contract."""


def _plain_yaml_scalar(value: str) -> str:
    value = value.strip()
    if len(value) >= 2 and value[0] == value[-1] and value[0] in {"'", '"'}:
        return value[1:-1]
    return value


def _agent_frontmatter(path: Path) -> Dict[str, str]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeError) as exc:
        raise DispatchPolicyError(f"cannot read Agent definition {path}: {exc}") from exc

    if not lines or lines[0].strip() != "---":
        raise DispatchPolicyError(f"Agent definition has no YAML frontmatter: {path}")

    try:
        end = next(
            index
            for index, line in enumerate(lines[1:], start=1)
            if line.strip() == "---"
        )
    except StopIteration as exc:
        raise DispatchPolicyError(
            f"Agent definition has unterminated YAML frontmatter: {path}"
        ) from exc

    fields: Dict[str, str] = {}
    for line in lines[1:end]:
        match = re.fullmatch(r"(name|effort|model):\s*(.*?)\s*", line)
        if not match:
            continue
        key, raw_value = match.groups()
        if key in fields:
            raise DispatchPolicyError(f"Agent definition repeats {key}: {path}")
        fields[key] = _plain_yaml_scalar(raw_value)
    return fields


def load_project_agent_efforts(project_root: Path) -> Dict[str, str]:
    agent_dir = project_root / ".claude" / "agents"
    if not agent_dir.is_dir():
        raise DispatchPolicyError(f"project Agent directory is missing: {agent_dir}")

    agents: Dict[str, str] = {}
    for path in sorted(agent_dir.rglob("*.md")):
        fields = _agent_frontmatter(path)
        name = fields.get("name")
        effort = fields.get("effort")
        if not name:
            raise DispatchPolicyError(f"Agent definition has no name: {path}")
        if effort not in VALID_EFFORTS:
            raise DispatchPolicyError(
                f"Agent {name!r} has no valid explicit effort in {path}"
            )
        if fields.get("model") not in VALID_MODELS:
            raise DispatchPolicyError(
                f"Agent {name!r} has no valid explicit model in {path}"
            )
        if name in agents:
            raise DispatchPolicyError(f"duplicate project Agent name: {name}")
        agents[name] = effort

    if not agents:
        raise DispatchPolicyError(f"project Agent directory is empty: {agent_dir}")
    return agents


def validate_agent_call(payload: Any, project_root: Path) -> None:
    if not isinstance(payload, dict):
        raise DispatchPolicyError("hook input must be one JSON object")

    if payload.get("tool_name") != "Agent":
        return

    tool_input = payload.get("tool_input")
    if not isinstance(tool_input, dict):
        raise DispatchPolicyError("Agent tool_input must be one JSON object")

    description = tool_input.get("description")
    if not isinstance(description, str):
        raise DispatchPolicyError(
            "description must start with [effort=<level>]"
        )
    marker = EFFORT_MARKER.match(description)
    if marker is None:
        raise DispatchPolicyError(
            "description must start with [effort=low|medium|high|xhigh|max]"
        )
    declared_effort = marker.group(1)

    subagent_type = tool_input.get("subagent_type")
    if not isinstance(subagent_type, str) or not subagent_type:
        raise DispatchPolicyError("subagent_type must name one project Agent")

    agents = load_project_agent_efforts(project_root)
    actual_effort = agents.get(subagent_type)
    if actual_effort is None:
        raise DispatchPolicyError(
            f"{subagent_type!r} is not a project Agent with explicit effort"
        )
    if declared_effort != actual_effort:
        raise DispatchPolicyError(
            f"declared effort {declared_effort!r} does not match "
            f"{subagent_type!r} effort {actual_effort!r}"
        )


def _project_root() -> Path:
    configured = os.environ.get("CLAUDE_PROJECT_DIR")
    if configured:
        return Path(configured).resolve()
    return Path(__file__).resolve().parents[2]


def main() -> int:
    try:
        payload = json.load(sys.stdin)
        validate_agent_call(payload, _project_root())
    except (DispatchPolicyError, json.JSONDecodeError) as exc:
        print(f"Agent dispatch blocked: {exc}", file=sys.stderr)
        return 2
    except Exception as exc:  # Fail closed on validator defects or I/O drift.
        print(f"Agent dispatch blocked: effort validator failed: {exc}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
