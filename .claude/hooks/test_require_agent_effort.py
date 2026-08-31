#!/usr/bin/env python3
"""Tests for the Agent dispatch effort gate."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import unittest
from pathlib import Path
from typing import Optional

from require_agent_effort import (
    DispatchPolicyError,
    VALID_EFFORTS,
    load_project_agent_efforts,
    validate_agent_call,
)


PROJECT_ROOT = Path(__file__).resolve().parents[2]
HOOK = Path(__file__).with_name("require_agent_effort.py")


def payload(agent: str, effort: Optional[str]) -> dict[str, object]:
    description = "Inspect the assigned scope"
    if effort is not None:
        description = f"[effort={effort}] {description}"
    return {
        "hook_event_name": "PreToolUse",
        "tool_name": "Agent",
        "tool_input": {
            "description": description,
            "prompt": "Inspect the assigned scope and report evidence.",
            "subagent_type": agent,
        },
    }


class AgentEffortHookTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.agents = load_project_agent_efforts(PROJECT_ROOT)

    def test_every_project_agent_has_one_valid_effort(self) -> None:
        self.assertTrue(self.agents)
        self.assertTrue(set(self.agents.values()).issubset(VALID_EFFORTS))

    def test_project_settings_wire_the_pre_tool_use_gate(self) -> None:
        settings = json.loads(
            (PROJECT_ROOT / ".claude" / "settings.json").read_text(encoding="utf-8")
        )
        groups = settings["hooks"]["PreToolUse"]
        agent_groups = [group for group in groups if group.get("matcher") == "Agent"]
        self.assertEqual(len(agent_groups), 1)
        handlers = agent_groups[0]["hooks"]
        self.assertEqual(len(handlers), 1)
        self.assertEqual(handlers[0]["type"], "command")
        self.assertEqual(handlers[0]["command"], "python3")
        self.assertEqual(
            handlers[0]["args"],
            ["${CLAUDE_PROJECT_DIR}/.claude/hooks/require_agent_effort.py"],
        )

    def test_matching_explicit_effort_is_accepted(self) -> None:
        agent, effort = next(iter(self.agents.items()))
        validate_agent_call(payload(agent, effort), PROJECT_ROOT)

    def test_missing_effort_marker_is_rejected(self) -> None:
        agent = next(iter(self.agents))
        with self.assertRaisesRegex(DispatchPolicyError, "description must start"):
            validate_agent_call(payload(agent, None), PROJECT_ROOT)

    def test_mismatched_effort_is_rejected(self) -> None:
        agent, actual = next(iter(self.agents.items()))
        wrong = next(effort for effort in sorted(VALID_EFFORTS) if effort != actual)
        with self.assertRaisesRegex(DispatchPolicyError, "does not match"):
            validate_agent_call(payload(agent, wrong), PROJECT_ROOT)

    def test_unknown_or_builtin_agent_is_rejected(self) -> None:
        with self.assertRaisesRegex(DispatchPolicyError, "not a project Agent"):
            validate_agent_call(payload("Explore", "low"), PROJECT_ROOT)

    def test_command_returns_exit_two_for_a_missing_marker(self) -> None:
        agent = next(iter(self.agents))
        environment = dict(os.environ)
        environment["CLAUDE_PROJECT_DIR"] = str(PROJECT_ROOT)
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps(payload(agent, None)),
            text=True,
            capture_output=True,
            check=False,
            env=environment,
        )
        self.assertEqual(result.returncode, 2, result)
        self.assertIn("Agent dispatch blocked", result.stderr)

    def test_command_returns_zero_for_a_matching_marker(self) -> None:
        agent, effort = next(iter(self.agents.items()))
        environment = dict(os.environ)
        environment["CLAUDE_PROJECT_DIR"] = str(PROJECT_ROOT)
        result = subprocess.run(
            [sys.executable, str(HOOK)],
            input=json.dumps(payload(agent, effort)),
            text=True,
            capture_output=True,
            check=False,
            env=environment,
        )
        self.assertEqual(result.returncode, 0, result)
        self.assertEqual(result.stdout, "")
        self.assertEqual(result.stderr, "")


if __name__ == "__main__":
    unittest.main()
