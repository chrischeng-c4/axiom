#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[2] / "scripts" / "_agy_assignment_guard.py"
SPEC = importlib.util.spec_from_file_location("agy_assignment_guard", SCRIPT)
assert SPEC and SPEC.loader
guard = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(guard)


class AssignmentGuardTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        (self.root / "src").mkdir()
        (self.root / "src" / "allowed.txt").write_text("safe\n")
        self.policy = {
            "schema": guard.POLICY_SCHEMA,
            "assignment_digest": "a" * 64,
            "task_key": "T-1",
            "worktree_root": str(self.root),
            "workspace_roots": [str(self.root)],
            "executor_role": "qa",
            "model": "gemini-3.8-flash-high",
            "task_commands": {
                "allow": ["pwd", guard.SAFE_DENIAL_CANARY],
                "deny": ["git push origin main"],
            },
            "allowed_repo_writes": ["src/allowed.txt"],
            "expected_denied_commands": [guard.SAFE_DENIAL_CANARY],
        }
        self.policy_digest = guard.canonical_digest(self.policy)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def payload(
        self,
        name: str,
        args: dict,
        *,
        step: int = 1,
        workspace_paths: list[str] | None = None,
        model: str | None = None,
    ) -> dict:
        return {
            "conversationId": "conversation-1",
            "stepIdx": step,
            "workspacePaths": workspace_paths or [str(self.root)],
            "modelName": model or self.policy["model"],
            "toolCall": {"name": name, "args": args},
        }

    def decision(self, name: str, args: dict, **kwargs: object) -> tuple[dict, dict]:
        return guard.decide(
            self.policy,
            self.policy_digest,
            self.payload(name, args, **kwargs),
        )

    def test_exact_shell_command_and_cwd_are_allowed(self) -> None:
        decision, event = self.decision(
            "run_command",
            {"CommandLine": "pwd", "Cwd": str(self.root)},
        )
        self.assertEqual(decision["decision"], "allow")
        self.assertEqual(event["decision"], "allow")

    def test_fixed_canary_is_denied_before_execution(self) -> None:
        decision, event = self.decision(
            "run_command",
            {
                "CommandLine": guard.SAFE_DENIAL_CANARY,
                "Cwd": str(self.root),
            },
        )
        self.assertEqual(decision["decision"], "deny")
        self.assertEqual(event["reason"], "expected assignment isolation denial")

    def test_unlisted_denied_wrong_cwd_and_persistent_commands_fail_closed(self) -> None:
        cases = (
            {"CommandLine": "printf surprise", "Cwd": str(self.root)},
            {"CommandLine": "git push origin main", "Cwd": str(self.root)},
            {"CommandLine": "pwd", "Cwd": str(self.root.parent)},
            {
                "CommandLine": "pwd",
                "Cwd": str(self.root),
                "RunPersistent": True,
            },
        )
        for args in cases:
            with self.subTest(args=args):
                self.assertEqual(
                    self.decision("run_command", args)[0]["decision"],
                    "deny",
                )

    def test_context_identity_must_match_the_frozen_policy(self) -> None:
        cases = (
            {"workspace_paths": [str(self.root.parent)]},
            {"model": "different-model"},
        )
        for kwargs in cases:
            with self.subTest(kwargs=kwargs):
                self.assertEqual(
                    self.decision(
                        "run_command",
                        {"CommandLine": "pwd", "Cwd": str(self.root)},
                        **kwargs,
                    )[0]["decision"],
                    "deny",
                )

    def test_read_tools_stay_inside_the_worktree(self) -> None:
        allowed, _ = self.decision(
            "view_file",
            {"AbsolutePath": str(self.root / "src" / "allowed.txt")},
        )
        denied, _ = self.decision(
            "view_file",
            {"AbsolutePath": str(self.root.parent / "outside.txt")},
        )
        self.assertEqual(allowed["decision"], "allow")
        self.assertEqual(denied["decision"], "deny")

    def test_qa_write_is_denied_and_dev_exact_write_is_allowed(self) -> None:
        args = {"TargetFile": str(self.root / "src" / "allowed.txt")}
        self.assertEqual(
            self.decision("write_to_file", args)[0]["decision"],
            "deny",
        )
        self.policy["executor_role"] = "dev"
        self.policy["model"] = "gemini-3.8-flash-medium"
        self.policy_digest = guard.canonical_digest(self.policy)
        self.assertEqual(
            self.decision("write_to_file", args)[0]["decision"],
            "allow",
        )
        outside, _ = self.decision(
            "write_to_file",
            {"TargetFile": str(self.root / "src" / "other.txt")},
        )
        self.assertEqual(outside["decision"], "deny")

    def test_symlink_escape_and_unknown_tool_are_denied(self) -> None:
        outside = self.root.parent / "outside.txt"
        outside.write_text("outside\n")
        (self.root / "src" / "escape.txt").symlink_to(outside)
        self.assertEqual(
            self.decision(
                "view_file",
                {"AbsolutePath": str(self.root / "src" / "escape.txt")},
            )[0]["decision"],
            "deny",
        )
        self.assertEqual(self.decision("browser", {})[0]["decision"], "deny")

    def test_policy_loader_rejects_overlap_and_unsafe_paths(self) -> None:
        path = self.root / "policy.json"
        overlap = json.loads(json.dumps(self.policy))
        overlap["task_commands"]["deny"].append("pwd")
        path.write_text(json.dumps(overlap))
        with self.assertRaisesRegex(guard.PolicyError, "must not overlap"):
            guard.load_policy(path)

        relative = json.loads(json.dumps(self.policy))
        relative["worktree_root"] = "relative"
        path.write_text(json.dumps(relative))
        with self.assertRaisesRegex(guard.PolicyError, "must be absolute"):
            guard.load_policy(path)

    def test_cli_without_active_policy_never_grants_new_authority(self) -> None:
        env = os.environ.copy()
        env.pop(guard.POLICY_ENV, None)
        env.pop(guard.AUDIT_ENV, None)
        result = subprocess.run(
            [sys.executable, str(SCRIPT)],
            input="{}",
            text=True,
            capture_output=True,
            env=env,
            check=True,
        )
        self.assertEqual(
            json.loads(result.stdout)["decision"],
            "deny",
        )

    def test_cli_writes_one_audit_record_for_each_decision(self) -> None:
        policy_path = self.root / "policy.json"
        audit_path = self.root / "audit.jsonl"
        policy_path.write_text(json.dumps(self.policy))
        env = {
            **os.environ,
            guard.POLICY_ENV: str(policy_path),
            guard.AUDIT_ENV: str(audit_path),
        }
        result = subprocess.run(
            [sys.executable, str(SCRIPT)],
            input=json.dumps(
                self.payload(
                    "run_command",
                    {"CommandLine": "pwd", "Cwd": str(self.root)},
                )
            ),
            text=True,
            capture_output=True,
            env=env,
            check=True,
        )
        self.assertEqual(json.loads(result.stdout)["decision"], "allow")
        records = audit_path.read_text().splitlines()
        self.assertEqual(len(records), 1)
        self.assertEqual(json.loads(records[0])["decision"], "allow")


if __name__ == "__main__":
    unittest.main()
