#!/usr/bin/env python3
"""Tests for the Codex spawn_agent effort gate."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Optional

from require_spawn_agent_effort import (
    DispatchPolicyError,
    VALID_EFFORTS,
    validate_spawn_agent_call,
)


PROJECT_ROOT = Path(__file__).resolve().parents[2]
HOOK = Path(__file__).with_name("require_spawn_agent_effort.py")


def payload(
    effort: Optional[str],
    fork_turns: Optional[str] = "none",
    tool_name: str = "spawn_agent",
) -> dict[str, object]:
    tool_input: dict[str, object] = {
        "task_name": "bounded_check",
        "message": "Inspect the assigned scope and report evidence.",
        "agent_type": "tape-dev",
    }
    if effort is not None:
        tool_input["reasoning_effort"] = effort
    if fork_turns is not None:
        tool_input["fork_turns"] = fork_turns
    return {
        "hook_event_name": "PreToolUse",
        "tool_name": tool_name,
        "tool_input": tool_input,
    }


class SpawnAgentEffortHookTests(unittest.TestCase):
    def test_each_allowed_effort_is_accepted(self) -> None:
        for effort in sorted(VALID_EFFORTS):
            with self.subTest(effort=effort):
                validate_spawn_agent_call(payload(effort))

    def test_positive_history_bound_is_accepted(self) -> None:
        validate_spawn_agent_call(payload("high", fork_turns="3"))

    def test_agent_matcher_alias_is_accepted(self) -> None:
        validate_spawn_agent_call(payload("medium", tool_name="Agent"))

    def test_missing_effort_is_rejected(self) -> None:
        with self.assertRaisesRegex(DispatchPolicyError, "must be explicit"):
            validate_spawn_agent_call(payload(None))

    def test_unapproved_effort_is_rejected(self) -> None:
        with self.assertRaisesRegex(DispatchPolicyError, "must be explicit"):
            validate_spawn_agent_call(payload("ultra"))

    def test_default_or_full_history_is_rejected(self) -> None:
        for fork_turns in (None, "all", "0"):
            with self.subTest(fork_turns=fork_turns):
                with self.assertRaisesRegex(DispatchPolicyError, "fork_turns"):
                    validate_spawn_agent_call(
                        payload("high", fork_turns=fork_turns)
                    )

    def test_other_tools_are_ignored(self) -> None:
        validate_spawn_agent_call(payload(None, tool_name="update_plan"))

    def test_project_hooks_wire_the_spawn_agent_gate(self) -> None:
        settings = json.loads(
            (PROJECT_ROOT / ".codex" / "hooks.json").read_text(encoding="utf-8")
        )
        groups = settings["hooks"]["PreToolUse"]
        self.assertEqual(len(groups), 1)
        self.assertEqual(groups[0]["matcher"], "^(spawn_agent|Agent)$")
        handlers = groups[0]["hooks"]
        self.assertEqual(len(handlers), 1)
        self.assertEqual(handlers[0]["type"], "command")
        self.assertIn("require_spawn_agent_effort.py", handlers[0]["command"])

    def test_command_returns_exit_two_for_missing_effort(self) -> None:
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps(payload(None)),
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 2, result)
        self.assertIn("Codex subagent dispatch blocked", result.stderr)

    def test_command_returns_zero_for_explicit_effort(self) -> None:
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps(payload("xhigh")),
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
