#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).with_name("codex_dispatch.py")
SPEC = importlib.util.spec_from_file_location("codex_dispatch", SCRIPT)
assert SPEC and SPEC.loader
codex_dispatch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(codex_dispatch)


class CodexDispatchTest(unittest.TestCase):
    def test_unwrapped_concatenated_quoting(self) -> None:
        cmd = '/bin/zsh -lc \'rg foo\'" bar"'
        resolved = codex_dispatch.unwrapped(cmd)
        self.assertTrue(
            resolved.startswith("rg"),
            f"Expected unwrapped command to start with 'rg', got {resolved!r}",
        )

    def test_measurement_two_expression(self) -> None:
        q = chr(39)
        d = chr(34)
        c1 = codex_dispatch.unwrapped('/bin/zsh -lc ' + q + 'rg foo' + q + d + ' bar' + d)
        c2 = codex_dispatch.unwrapped('/bin/zsh -lc ' + q + 'rg foo' + q)
        self.assertTrue(c1.startswith('rg'))
        self.assertEqual(c2, 'rg foo')

    def test_unwrapped_balanced_pair(self) -> None:
        cmd = "/bin/zsh -lc 'rg foo'"
        resolved = codex_dispatch.unwrapped(cmd)
        self.assertEqual(resolved, "rg foo")

    def test_unauthorized_commands_authorizes_concatenated_and_balanced_rejects_out_of_family(
        self,
    ) -> None:
        cmd_concat = '/bin/zsh -lc \'rg foo\'" bar"'
        cmd_balanced = "/bin/zsh -lc 'rg foo'"
        cmd_unauthorized = "/bin/zsh -lc 'forbidden_cmd arg'"

        with tempfile.TemporaryDirectory() as tmpdir:
            state_dir = Path(tmpdir)
            runs_dir = state_dir / "runs"
            runs_dir.mkdir(parents=True)

            log_path = runs_dir / "task-1.jsonl"
            log_contents = "\n".join(
                [
                    json.dumps(
                        {
                            "item": {
                                "type": "command_execution",
                                "id": "1",
                                "command": cmd_concat,
                            }
                        }
                    ),
                    json.dumps(
                        {
                            "item": {
                                "type": "command_execution",
                                "id": "2",
                                "command": cmd_balanced,
                            }
                        }
                    ),
                    json.dumps(
                        {
                            "item": {
                                "type": "command_execution",
                                "id": "3",
                                "command": cmd_unauthorized,
                            }
                        }
                    ),
                ]
            )
            log_path.write_text(log_contents)

            profile = {
                "state_dir": str(state_dir),
                "task_commands": {
                    "allow": [],
                    "allow_prefix": ["rg"],
                },
            }

            stray = codex_dispatch.unauthorized_commands(profile, "task-1")
            self.assertNotIn(cmd_concat, stray)
            self.assertNotIn(cmd_balanced, stray)
            self.assertIn(cmd_unauthorized, stray)
            self.assertEqual(stray, [cmd_unauthorized])

    def test_unwrapped_same_quote_concatenation(self) -> None:
        cmd = "/bin/zsh -lc 'rg' 'foo'"
        resolved = codex_dispatch.unwrapped(cmd)
        self.assertEqual(resolved, "rg foo")
        self.assertTrue(resolved.startswith("rg"))

    def test_unauthorized_commands_same_quote_concatenation(self) -> None:
        cmd_same_quote = "/bin/zsh -lc 'rg' 'foo'"
        with tempfile.TemporaryDirectory() as tmpdir:
            state_dir = Path(tmpdir)
            runs_dir = state_dir / "runs"
            runs_dir.mkdir(parents=True)
            log_path = runs_dir / "task-1.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "item": {
                            "type": "command_execution",
                            "id": "1",
                            "command": cmd_same_quote,
                        }
                    }
                )
            )
            profile = {
                "state_dir": str(state_dir),
                "task_commands": {
                    "allow": [],
                    "allow_prefix": ["rg"],
                },
            }
            stray = codex_dispatch.unauthorized_commands(profile, "task-1")
            self.assertNotIn(cmd_same_quote, stray)

    def test_unauthorized_commands_prefix_with_space(self) -> None:
        cmd_space_prefix = "/bin/zsh -lc 'git log -n 5'"
        with tempfile.TemporaryDirectory() as tmpdir:
            state_dir = Path(tmpdir)
            runs_dir = state_dir / "runs"
            runs_dir.mkdir(parents=True)
            log_path = runs_dir / "task-1.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "item": {
                            "type": "command_execution",
                            "id": "1",
                            "command": cmd_space_prefix,
                        }
                    }
                )
            )
            profile = {
                "state_dir": str(state_dir),
                "task_commands": {
                    "allow": [],
                    "allow_prefix": ["git log"],
                },
            }
            stray = codex_dispatch.unauthorized_commands(profile, "task-1")
            self.assertNotIn(cmd_space_prefix, stray)

    def test_unauthorized_commands_word_begins_with_family_prefix(self) -> None:
        cmd_rgrep = "/bin/zsh -lc 'rgrep foo'"
        with tempfile.TemporaryDirectory() as tmpdir:
            state_dir = Path(tmpdir)
            runs_dir = state_dir / "runs"
            runs_dir.mkdir(parents=True)
            log_path = runs_dir / "task-1.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "item": {
                            "type": "command_execution",
                            "id": "1",
                            "command": cmd_rgrep,
                        }
                    }
                )
            )
            profile = {
                "state_dir": str(state_dir),
                "task_commands": {
                    "allow": [],
                    "allow_prefix": ["rg"],
                },
            }
            stray = codex_dispatch.unauthorized_commands(profile, "task-1")
            self.assertIn(cmd_rgrep, stray)

    def test_unauthorized_commands_unparseable_fallback(self) -> None:
        cmd_unclosed = "/bin/zsh -lc 'rg foo"
        with tempfile.TemporaryDirectory() as tmpdir:
            state_dir = Path(tmpdir)
            runs_dir = state_dir / "runs"
            runs_dir.mkdir(parents=True)
            log_path = runs_dir / "task-1.jsonl"
            log_path.write_text(
                json.dumps(
                    {
                        "item": {
                            "type": "command_execution",
                            "id": "1",
                            "command": cmd_unclosed,
                        }
                    }
                )
            )
            profile = {
                "state_dir": str(state_dir),
                "task_commands": {
                    "allow": [],
                    "allow_prefix": ["rg"],
                },
            }
            stray = codex_dispatch.unauthorized_commands(profile, "task-1")
            self.assertNotIn(cmd_unclosed, stray)


if __name__ == "__main__":
    unittest.main()
