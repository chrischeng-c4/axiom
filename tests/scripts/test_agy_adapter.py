#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import inspect
import io
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest
from contextlib import ExitStack, contextmanager
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).parents[2] / "scripts" / "_agy_adapter.py"
sys.path.insert(0, str(SCRIPT.parent))
SPEC = importlib.util.spec_from_file_location("agy_adapter", SCRIPT)
assert SPEC and SPEC.loader
agy_adapter = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(agy_adapter)


class DispatchControllerTest(unittest.TestCase):
    def setUp(self) -> None:
        state_parent = Path("/tmp/execution/agy")
        state_parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=state_parent)
        self.root = Path(self.temporary.name)
        self.state_parent = state_parent
        self.state_project_ids: set[str] = set()
        self.conversation_dir = self.root / "conversations"
        self.standing_consent = self.root / "standing-consent.json"
        self.settings = self.root / "settings.json"
        self.global_hooks = self.root / "global-hooks.json"
        self.static_guard_dir = self.root / "execution"
        self.project_a = self.root / "project-a"
        self.project_b = self.root / "project-b"
        self.project_a.mkdir()
        self.project_b.mkdir()
        self.conversation_dir.mkdir()
        self.settings.write_text(
            json.dumps(
                {
                    "allowNonWorkspaceAccess": False,
                    "permissions": agy_adapter.canonical_global_policy(),
                }
            )
        )
        self.project_surface = {
            "allow": [],
            "deny": [],
            "ask": [],
        }
        agy_adapter.SETTINGS = self.settings
        agy_adapter.CONVERSATION_DIR = self.conversation_dir
        agy_adapter.STANDING_CONSENT = self.standing_consent
        agy_adapter.GLOBAL_HOOK_PATHS = (
            self.global_hooks,
            self.root / "cli-hooks.json",
        )
        agy_adapter.STATIC_GUARD_DIR = self.static_guard_dir
        agy_adapter.STATIC_GUARD_LOADER = (
            self.static_guard_dir / "agy-assignment-guard-loader.py"
        )
        agy_adapter.STATIC_GUARD_CORE = (
            self.static_guard_dir / "agy-assignment-guard-core.py"
        )
        agy_adapter.STATIC_GUARD_CHAIN = (
            self.static_guard_dir / "agy-assignment-guard-chain.py"
        )
        self.static_guard_dir.mkdir(mode=0o700)
        for source, target in (
            (agy_adapter.static_guard_sources()["loader"], agy_adapter.STATIC_GUARD_LOADER),
            (agy_adapter.static_guard_sources()["core"], agy_adapter.STATIC_GUARD_CORE),
            (agy_adapter.static_guard_sources()["chain"], agy_adapter.STATIC_GUARD_CHAIN),
        ):
            shutil.copyfile(source, target)
            target.chmod(0o700)
        self.global_hooks.write_text(
            json.dumps(
                {
                    "cap-agent-guard": {"PreToolUse": []},
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: (
                        agy_adapter.global_assignment_guard_hook_config()
                    ),
                }
            )
        )
        self.init_git_repo(self.project_a)
        self.init_git_repo(self.project_b)
        self.repo_a = self.add_nested_worktree(self.project_a, "repo-a")
        self.repo_b = self.add_nested_worktree(self.project_b, "repo-b")

    def tearDown(self) -> None:
        for project_id in self.state_project_ids:
            shutil.rmtree(self.state_parent / project_id, ignore_errors=True)
        self.temporary.cleanup()

    def init_git_repo(self, root: Path) -> None:
        subprocess.run(["git", "init", "-q"], cwd=root, check=True)
        subprocess.run(
            ["git", "config", "user.email", "executor-adapter@test.invalid"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Dispatch to AGY Test"],
            cwd=root,
            check=True,
        )
        (root / ".gitignore").write_text(".venv/\n.agy-worktrees/\n")
        subprocess.run(["git", "add", ".gitignore"], cwd=root, check=True)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=root, check=True)

    def add_nested_worktree(self, scope: Path, name: str) -> Path:
        root = scope / ".agy-worktrees" / name
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(root), "HEAD"],
            cwd=scope,
            check=True,
        )
        return root

    def profile(
        self,
        root: Path,
        project_id: str,
        issue: str,
        *,
        project_root: Path | None = None,
    ) -> dict:
        if project_root is None:
            if root == self.repo_a:
                project_root = self.project_a
            elif root == self.repo_b:
                project_root = self.project_b
            else:
                project_root = root
        bound_project_id = f"{project_id}-{self.root.name}"
        self.state_project_ids.add(bound_project_id)
        backend_resolution = {
            "backend": "agy-cli",
            "persistent_root": str(project_root.resolve()),
            "project_id": bound_project_id,
            "project_config_digest": "b" * 64,
            "workspace_cache_observation_digest": "d" * 64,
            "registry_entry_digest": "c" * 64,
            "agy_version": "agy version 1.2.3",
        }
        return {
            "root": str(root),
            "agy_project_root": str(project_root),
            "repo": "owner/repo",
            "agy_project_id": bound_project_id,
            "backend_resolution": backend_resolution,
            "dispatch_role": "qa",
            "model": "gemini-3.8-flash-high",
            "effort": "high",
            "worktree_layout": "in-project",
            "launch_cwd": "task-worktree",
            "state_dir": str(self.state_parent / bound_project_id / issue),
            "mode": "measure-only",
            "external_payload_consent": {
                "destination": "agy-headless",
                "approved": True,
                "approval_source": "explicit_user_after_risk_disclosure",
                "approval_record": "I approve this AGY test payload transfer.",
                "approved_payload_classes": [
                    "task_contract",
                    "oracle",
                    "repository_read_context",
                ],
            },
            "task_contract": {
                "kind": "measurement",
                "session_policy": "ticketed",
                "issue": issue,
                "design_inputs": [],
            },
            "global_permissions": agy_adapter.canonical_global_policy(),
            "project_permissions": {
                kind: list(self.project_surface[kind])
                for kind in ("allow", "deny", "ask")
            },
            "project_policy_observation": {
                "source": "official_cli_permissions",
                "observed_at": "2026-08-05T00:00:00Z",
                "project_id": bound_project_id,
                "project_root": str(project_root.resolve()),
                "permissions": {
                    kind: list(self.project_surface[kind])
                    for kind in ("allow", "deny", "ask")
                },
            },
            "task_commands": {
                "allow": ["pwd", "rg -n TODO src"],
                "deny": ["git push origin main"],
            },
            "protected_artifacts": [],
            "snapshot_paths": ["src"],
            "allowed_repo_writes": [],
            "assignment_digest": "a" * 64,
            "sandbox": True,
            "path_change_budgets": {},
        }

    def one_shot_profile(
        self,
        root: Path,
        project_id: str,
        run_id: str = "adhoc-1",
    ) -> dict:
        profile = self.profile(root, project_id, "unused")
        profile["state_dir"] = str(
            self.state_parent / profile["agy_project_id"] / run_id
        )
        profile["task_contract"] = {
            "kind": "measurement",
            "session_policy": "one-shot",
            "run_id": run_id,
            "intent": "Inspect one bounded condition and report evidence.",
            "design_inputs": [],
        }
        return profile

    def write_standing_consent(
        self,
        *,
        payload_classes: list[str] | None = None,
        revoked: bool = False,
        approval_record: str = "I approve all bounded AGY test payload transfers.",
    ) -> dict:
        consent = {
            "version": 1,
            "consent_id": "all-bounded-work-items-v1",
            "scope": "all_bounded_work_items",
            "destination": "agy-headless",
            "approved": True,
            "revoked": revoked,
            "approval_source": "standing_explicit_user_authorization",
            "approval_record": approval_record,
            "approved_payload_classes": payload_classes
            or [
                "task_contract",
                "oracle",
                "repository_read_context",
                "design_inputs",
                "repository_write_diff",
                "injected_prompt",
            ],
        }
        self.standing_consent.write_text(json.dumps(consent))
        return consent

    def test_ticketed_policy_remains_the_default_for_legacy_profiles(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "10")
        del profile["task_contract"]["session_policy"]
        self.assertEqual(agy_adapter.task_session_policy(profile), "ticketed")
        agy_adapter.validate_task_key(profile, "10")

    def test_task_local_uv_environment_is_a_rebuild_cache_but_other_drift_fails_closed(self) -> None:
        (self.repo_a / ".venv").mkdir()
        (self.repo_a / ".venv" / "pyvenv.cfg").write_text("home = test\n")

        self.assertEqual(agy_adapter.git_ignored_paths(self.repo_a), [".venv/"])
        agy_adapter.assert_ignored_paths_unchanged(self.repo_a, {"ignored_paths": []})

        (self.repo_a / ".gitignore").write_text(".venv/\n.drift/\n")
        (self.repo_a / ".drift").mkdir()
        with self.assertRaisesRegex(SystemExit, "ignored repository path drift"):
            agy_adapter.assert_ignored_paths_unchanged(
                self.repo_a,
                {"ignored_paths": []},
            )

    def test_ignored_path_baseline_is_required(self) -> None:
        with self.assertRaisesRegex(SystemExit, "ignored-path baseline"):
            agy_adapter.assert_ignored_paths_unchanged(self.repo_a, {})

    def test_profile_accepts_one_shot_without_issue(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "one-shot.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_adapter.load_profile(str(profile_path))
        self.assertEqual(
            loaded["task_contract"]["session_policy"],
            "one-shot",
        )
        self.assertNotIn("issue", loaded["task_contract"])
        agy_adapter.validate_task_key(loaded, "adhoc-1")

    def test_profile_rejects_wrong_execution_contract_settings(self) -> None:
        cases = (
            ("model", "gemini-3.6-flash-high", "qa dispatch_role requires model gemini-3.8-flash-high"),
            ("worktree_layout", "external", "worktree_layout must be in-project"),
            ("launch_cwd", "project-root", "launch_cwd must be task-worktree"),
        )
        for field, value, expected in cases:
            with self.subTest(field=field):
                profile = self.one_shot_profile(self.repo_a, "project-a")
                profile[field] = value
                profile_path = self.root / f"wrong-{field}.json"
                profile_path.write_text(json.dumps(profile))
                with self.assertRaisesRegex(SystemExit, expected):
                    agy_adapter.load_profile(str(profile_path))

    def test_profile_requires_dispatch_role(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        del profile["dispatch_role"]
        profile_path = self.root / "missing-dispatch-role.json"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "dispatch_role"):
            agy_adapter.load_profile(str(profile_path))

    def test_profile_accepts_only_fixed_qa_and_dev_pairs(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["dispatch_role"] = "dev"
        profile["model"] = "gemini-3.8-flash-medium"
        profile["effort"] = "medium"
        profile_path = self.root / "dev-pair.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_adapter.load_profile(str(profile_path))
        self.assertEqual(loaded["dispatch_role"], "dev")

        for field, value, expected in (
            ("model", "gemini-3.8-flash-high", "dev dispatch_role requires model"),
            ("effort", "high", "dev dispatch_role requires effort"),
        ):
            with self.subTest(field=field):
                invalid = dict(profile)
                invalid[field] = value
                profile_path.write_text(json.dumps(invalid))
                with self.assertRaisesRegex(SystemExit, expected):
                    agy_adapter.load_profile(str(profile_path))

    def test_profile_rejects_retired_backend_resolution_digest(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        resolution = profile["backend_resolution"]
        resolution["selected_cache_entry_digest"] = resolution.pop(
            "workspace_cache_observation_digest"
        )
        profile_path = self.root / "retired-backend-resolution-digest.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "backend_resolution has an invalid shape",
        ):
            agy_adapter.load_profile(str(profile_path))

    def test_snapshot_contract_rejects_model_and_root_drift(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        snapshot_contract = agy_adapter.dispatch_contract(profile)

        profile["model"] = "gemini-3.8-flash-medium"
        self.assertFalse(
            agy_adapter.snapshot_contract_matches(profile, snapshot_contract)
        )

        profile["model"] = "gemini-3.8-flash-high"
        profile["root"] = str(self.repo_a.parent / "different-task-root")
        self.assertFalse(
            agy_adapter.snapshot_contract_matches(profile, snapshot_contract)
        )

        profile["root"] = str(self.repo_a)
        profile["repo"] = "other-owner/other-repo"
        self.assertFalse(
            agy_adapter.snapshot_contract_matches(profile, snapshot_contract)
        )

    def test_oracle_digest_is_required_and_frozen(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "oracle-frozen")
        oracle = Path(profile["state_dir"]) / "oracles" / "oracle-frozen.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("expected witness\n")
        snapshot = {"oracle_sha256": agy_adapter.sha256(oracle)}

        self.assertEqual(
            agy_adapter.assert_oracle_unchanged(
                profile,
                "oracle-frozen",
                snapshot,
            ),
            oracle,
        )
        oracle.write_text("changed witness\n")
        with self.assertRaisesRegex(SystemExit, "oracle changed"):
            agy_adapter.assert_oracle_unchanged(
                profile,
                "oracle-frozen",
                snapshot,
            )

    def test_snapshot_id_binds_immutable_history(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "snapshot-id")
        snapshot_dir = Path(profile["state_dir"]) / "snapshots"
        history_dir = snapshot_dir / "history" / "snapshot-id"
        history_dir.mkdir(parents=True)
        payload = {"task_key": "snapshot-id", "value": "frozen"}
        payload["snapshot_id"] = agy_adapter.json_digest(payload)
        encoded = json.dumps(payload)
        current = snapshot_dir / "snapshot-id.json"
        immutable = history_dir / f"{payload['snapshot_id']}.json"
        current.write_text(encoded)
        immutable.write_text(encoded)

        self.assertEqual(
            agy_adapter.load_snapshot(profile, "snapshot-id")["value"],
            "frozen",
        )
        current.write_text(json.dumps({**payload, "value": "rebound"}))
        with self.assertRaisesRegex(SystemExit, "snapshot id/digest mismatch"):
            agy_adapter.load_snapshot(profile, "snapshot-id")

    def test_run_agent_rejects_contract_drift_before_launch(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "prelaunch-drift")
        snapshot = {
            "task_key": "prelaunch-drift",
            "session_policy": "one-shot",
            "dispatch_contract": agy_adapter.dispatch_contract(profile),
            "agy_project_id": profile["agy_project_id"],
            "agy_project_root": profile["agy_project_root"],
            "worktree_scope": agy_adapter.worktree_scope_report(profile),
        }
        profile["model"] = "gemini-3.8-flash-medium"

        with (
            patch.object(agy_adapter, "require_project_ready", return_value={}),
            patch.object(
                agy_adapter,
                "frozen_task_state",
                return_value={"run_id": "prelaunch-drift"},
            ),
            patch.object(agy_adapter, "load_snapshot", return_value=snapshot),
            patch.object(agy_adapter.subprocess, "run") as launch,
            self.assertRaisesRegex(SystemExit, "dispatch contract changed"),
        ):
            agy_adapter.run_agent(profile, "prelaunch-drift", resume=False)
        launch.assert_not_called()

    def test_sandbox_and_stream_json_are_fixed_transport_flags(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "sandbox.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_adapter.load_profile(str(profile_path))

        command = agy_adapter.agy_command(loaded, None)
        self.assertIn("--sandbox", command)
        self.assertEqual(
            command[command.index("--output-format") + 1],
            "stream-json",
        )

        profile["sandbox"] = False
        profile_path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "sandbox.*true|fixed"):
            agy_adapter.load_profile(str(profile_path))

    def test_sandbox_file_access_denial_voids_candidate(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "10")
        state_dir = Path(profile["state_dir"])
        run_dir = state_dir / "runs"
        run_dir.mkdir(parents=True)
        (run_dir / "10.agy.log").write_text(
            "[sandbox-telemetry] emitting SANDBOX_COMMAND_BLOCKED "
            "command_output: rg: src/lib.rs: Operation not permitted"
        )

        with self.assertRaisesRegex(SystemExit, "sandbox denied task-root"):
            agy_adapter.assert_no_sandbox_file_access_denial(profile, "10")

    def test_profile_requires_explicit_informed_external_payload_consent(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "missing-consent.json"
        profile.pop("external_payload_consent")
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_adapter.load_profile(str(profile_path))

        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"]["approval_source"] = "inferred"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_adapter.load_profile(str(profile_path))

        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"]["approved_payload_classes"].remove(
            "repository_read_context"
        )
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_adapter.load_profile(str(profile_path))

    def test_standing_consent_covers_new_project_profiles_without_repeat_prompt(self) -> None:
        stored = self.write_standing_consent()
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"] = {
            "mode": "standing",
            "consent_id": "all-bounded-work-items-v1",
        }
        profile_path = self.root / "standing-consent-profile.json"
        profile_path.write_text(json.dumps(profile))

        loaded = agy_adapter.load_profile(str(profile_path))

        self.assertEqual(loaded["external_payload_consent"]["mode"], "standing")
        self.assertEqual(
            loaded["external_payload_consent"]["consent_id"],
            "all-bounded-work-items-v1",
        )
        self.assertEqual(
            loaded["external_payload_consent"]["registry_digest"],
            agy_adapter.json_digest(stored),
        )
        self.assertEqual(
            loaded["external_payload_consent"]["approved_payload_classes"],
            sorted(stored["approved_payload_classes"]),
        )

    def test_omitted_profile_consent_uses_matching_standing_registry(self) -> None:
        self.write_standing_consent()
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile.pop("external_payload_consent")
        profile_path = self.root / "implicit-standing-consent-profile.json"
        profile_path.write_text(json.dumps(profile))

        loaded = agy_adapter.load_profile(str(profile_path))

        self.assertEqual(loaded["external_payload_consent"]["mode"], "standing")

    def test_standing_consent_rejects_revocation_or_missing_payload_class(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"] = {
            "mode": "standing",
            "consent_id": "all-bounded-work-items-v1",
        }
        profile_path = self.root / "revoked-standing-consent-profile.json"
        profile_path.write_text(json.dumps(profile))

        self.write_standing_consent(revoked=True)
        with self.assertRaisesRegex(SystemExit, "standing consent registry is revoked"):
            agy_adapter.load_profile(str(profile_path))

        self.write_standing_consent(
            payload_classes=["task_contract", "oracle"],
        )
        with self.assertRaisesRegex(SystemExit, "repository_read_context"):
            agy_adapter.load_profile(str(profile_path))

    def test_standing_consent_change_voids_snapshot_contract_identity(self) -> None:
        self.write_standing_consent(approval_record="first approval")
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"] = {
            "mode": "standing",
            "consent_id": "all-bounded-work-items-v1",
        }
        profile_path = self.root / "standing-consent-identity-profile.json"
        profile_path.write_text(json.dumps(profile))
        first = agy_adapter.load_profile(str(profile_path))
        snapshot_contract = agy_adapter.dispatch_contract(first)

        self.write_standing_consent(approval_record="changed approval")
        changed = agy_adapter.load_profile(str(profile_path))

        self.assertFalse(
            agy_adapter.snapshot_contract_matches(changed, snapshot_contract)
        )

    def test_one_shot_rejects_issue_and_unsafe_run_id(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["task_contract"]["issue"] = "10"
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.validate_task_identity(profile)
        self.assertIn("must not set task_contract.issue", str(caught.exception))

        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "../escape",
        )
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.validate_task_identity(profile)
        self.assertIn("task identity must match", str(caught.exception))

    def test_one_shot_has_frozen_local_state_without_tracker_lookup(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        self.assertEqual(
            agy_adapter.frozen_task_state(profile, "adhoc-1"),
            {
                "run_id": "adhoc-1",
                "state": "ONE_SHOT",
                "kind": "measurement",
                "intent": "Inspect one bounded condition and report evidence.",
            },
        )

    def test_one_shot_resume_is_forbidden(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.validate_conversation_action(
                profile,
                "adhoc-1",
                resume=True,
            )
        self.assertIn("cannot resume one-shot", str(caught.exception))

    def test_existing_conversation_requires_resume_or_new_run_id(self) -> None:
        ticketed = self.profile(self.repo_a, "project-a", "11")
        one_shot = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "adhoc-2",
        )
        for profile, task_key in ((ticketed, "11"), (one_shot, "adhoc-2")):
            runs = Path(profile["state_dir"]) / "runs"
            runs.mkdir(parents=True)
            (runs / f"{task_key}.conversation").write_text("conversation-id\n")

        with self.assertRaises(SystemExit) as caught:
            agy_adapter.validate_conversation_action(
                ticketed,
                "11",
                resume=False,
            )
        self.assertIn("use resume for ticket #11", str(caught.exception))

        with self.assertRaises(SystemExit) as caught:
            agy_adapter.validate_conversation_action(
                one_shot,
                "adhoc-2",
                resume=False,
            )
        self.assertIn("create a new one-shot run id", str(caught.exception))

    def test_missing_conversation_attempt_cannot_be_dispatched_again(self) -> None:
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "missing-lineage",
        )
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "missing-lineage.evidence.json").write_text("{}\n")

        with self.assertRaisesRegex(
            SystemExit,
            "already has an initial run attempt.*new one-shot run id",
        ):
            agy_adapter.validate_conversation_action(
                profile,
                "missing-lineage",
                resume=False,
            )

    def test_project_policy_is_ready_without_mutating_project(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        report = agy_adapter.project_policy_report(profile)
        self.assertTrue(report["dispatch_ready"])
        self.assertEqual(report["project_permissions_status"], "ready")
        self.assertEqual(report["global_permissions_status"], "ready")
        self.assertEqual(
            report["project_policy_observability"],
            "official_cli_permissions",
        )
        self.assertEqual(
            report["permission_layer_diagnostics"]["file_access_policy"]["decision"],
            "deny",
        )
        self.assertEqual(
            report["permission_layer_diagnostics"]["controller_host"]["source"],
            "controller_host",
        )

    def test_project_permission_drift_blocks_dispatch(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_permissions"]["allow"].append("command(cargo test)")
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["provisioning_status"],
            "PROJECT_SETUP_REQUIRED: Project policy requires official CLI observation",
        )
        self.assertEqual(
            report["missing_project_rules"]["allow"],
            ["command(cargo test)"],
        )
        with self.assertRaisesRegex(
            SystemExit,
            "PROJECT_SETUP_REQUIRED: Project policy requires official CLI observation",
        ):
            agy_adapter.require_project_ready(profile)

    def test_global_settings_use_safe_sparse_non_workspace_default(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        baseline_digest = agy_adapter.permission_state_digest(profile)
        self.settings.write_text(
            json.dumps(
                {
                    "permissions": agy_adapter.canonical_global_policy(),
                }
            )
        )
        self.assertTrue(agy_adapter.project_policy_report(profile)["dispatch_ready"])
        ready_report = agy_adapter.project_policy_report(profile)
        self.assertTrue(ready_report["assignment_guard_hook"]["installed"])
        self.assertTrue(ready_report["assignment_guard_loader"]["ready"])
        self.assertEqual(
            agy_adapter.permission_state_digest(profile),
            baseline_digest,
        )

        self.settings.write_text(
            json.dumps(
                {
                    "allowNonWorkspaceAccess": True,
                    "permissions": agy_adapter.canonical_global_policy(),
                }
            )
        )
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertIn("allowNonWorkspaceAccess", " ".join(report["blockers"]))

    def test_global_assignment_guard_hook_is_required_and_exact(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "global-guard")
        self.global_hooks.unlink()

        report = agy_adapter.project_policy_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["provisioning_status"],
            "GLOBAL_GUARD_SETUP_REQUIRED: install the exact user-global "
            "execution-assignment-guard loader",
        )
        self.assertFalse(report["assignment_guard_hook"]["installed"])
        self.assertIn("execution-assignment-guard", " ".join(report["blockers"]))

        self.global_hooks.write_text(
            json.dumps(
                {
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: (
                        agy_adapter.global_assignment_guard_hook_config()
                    )
                }
            )
        )
        self.assertTrue(agy_adapter.project_policy_report(profile)["dispatch_ready"])

        source = agy_adapter.static_guard_sources()["core"]
        agy_adapter.STATIC_GUARD_CORE.write_text("changed\n")
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["provisioning_status"],
            "GLOBAL_GUARD_SETUP_REQUIRED: install the fixed user-local "
            "assignment guard loader and core",
        )
        shutil.copyfile(source, agy_adapter.STATIC_GUARD_CORE)
        agy_adapter.STATIC_GUARD_CORE.chmod(0o700)

        self.global_hooks.write_text(
            json.dumps(
                {
                    "cap-agent-guard": {
                        "PreToolUse": [
                            {
                                "matcher": "run_command",
                                "hooks": [
                                    {
                                        "type": "command",
                                        "command": "cap hook agy",
                                        "timeout": 10,
                                    }
                                ],
                            }
                        ]
                    },
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: (
                        agy_adapter.global_assignment_guard_hook_config()
                    ),
                }
            )
        )
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["provisioning_status"],
            "GLOBAL_GUARD_SETUP_REQUIRED: use the fixed "
            "execution-assignment-guard chain for every active "
            "cap-agent-guard run_command matcher",
        )
        self.assertFalse(
            report["assignment_guard_hook"]["command_matcher_covered"]
        )

        self.global_hooks.write_text(
            json.dumps(
                {
                    "cap-agent-guard": {
                        "PreToolUse": [
                            {
                                "matcher": "run_command",
                                "hooks": [
                                    agy_adapter.static_assignment_guard_chain_hook()
                                ],
                            }
                        ]
                    },
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: (
                        agy_adapter.global_assignment_guard_hook_config()
                    ),
                }
            )
        )
        report = agy_adapter.project_policy_report(profile)
        self.assertTrue(report["dispatch_ready"])
        self.assertTrue(
            report["assignment_guard_hook"]["command_matcher_covered"]
        )

        self.global_hooks.write_text(
            json.dumps(
                {
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: {
                        "PreToolUse": []
                    }
                }
            )
        )
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])

    def test_missing_official_project_observation_fails_closed_with_manual_steps(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile.pop("project_policy_observation")
        report = agy_adapter.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(report["project_policy_observability"], "PROJECT_SETUP_REQUIRED")
        self.assertIn(
            "Project policy has not been observed through /permissions Project scope",
            report["blockers"],
        )
        self.assertTrue(report["manual_setup"])
        self.assertIn("agy --new-project", " ".join(report["manual_setup"]))
        self.assertIn("/permissions", " ".join(report["manual_setup"]))

    def test_project_observation_id_must_match_cache_resolved_id(self) -> None:
        profile = self.profile(self.repo_a, "app_lumen", "1")
        profile["project_policy_observation"]["project_id"] = "other-project"
        path = self.root / "mismatched-cli-observation.json"
        path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "project_id.*agy_project_id"):
            agy_adapter.load_profile(str(path))

    def test_project_setup_is_cli_only_and_has_no_desktop_instructions(self) -> None:
        profile = self.profile(self.repo_a, "app_lumen", "1")
        profile.pop("project_policy_observation")
        completed = subprocess.CompletedProcess(
            ["agy", "--help"],
            0,
            stdout="Usage of agy:\n  --project\n  --new-project\nAvailable subcommands:\n  models\n",
            stderr="",
        )
        with patch.object(agy_adapter.subprocess, "run", return_value=completed):
            report = agy_adapter.project_policy_report(profile)
        capabilities = report["formal_project_capabilities"]
        self.assertFalse(capabilities["project_enumeration_cli"])
        self.assertFalse(capabilities["machine_readable_project_policy_cli"])
        self.assertEqual(report["provisioning_status"].split(":", 1)[0], "PROJECT_SETUP_REQUIRED")
        instructions = " ".join(report["manual_setup"])
        self.assertIn("agy --new-project", instructions)
        self.assertIn("agy --project=", instructions)
        self.assertIn("/permissions", instructions)
        for forbidden in ("Desktop", "Project Settings UI", "gear icon", "AppleScript", "osascript"):
            self.assertNotIn(forbidden, instructions)

    def test_global_permissions_drift_blocks_preflight(self) -> None:
        self.settings.write_text(
            json.dumps(
                {
                    "allowNonWorkspaceAccess": False,
                    "permissions": {
                        "allow": ["command(cargo test)"],
                        "deny": [],
                        "ask": [],
                    },
                }
            )
        )
        report = agy_adapter.project_policy_report(
            self.profile(self.repo_a, "project-a", "1")
        )
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(report["global_permissions_status"], "drift")
        self.assertEqual(
            report["provisioning_status"],
            "GLOBAL_SETUP_REQUIRED: Global permissions differ from reviewed baseline",
        )
        self.assertEqual(
            report["global_rule_sources"]["agy_cli_global_settings"]["allow"],
            ["command(cargo test)"],
        )
    def test_canonical_global_policy_preserves_safe_git_reads_and_denies_controller_mutations(self) -> None:
        policy = agy_adapter.canonical_global_policy()
        self.assertEqual(policy, agy_adapter.CANONICAL_GLOBAL_POLICY)
        self.assertNotIn("command(git)", policy["deny"])
        empty = agy_adapter.normalize_permission_surface({})
        for command in (
            "git log --oneline",
            "git status --short",
            "git diff --check",
            "git show HEAD",
            "git rev-parse HEAD",
            "git ls-files",
            "git merge-base HEAD main",
        ):
            self.assertEqual(
                agy_adapter.permission_decision(policy, empty, command)[0],
                "allow",
            )
        for command in (
            "git add file",
            "git commit -m message",
            "git push origin main",
            "git checkout main",
            "git rebase main",
            "git apply change.patch",
            "git tag v1.0.0",
            "gh issue close 42",
            "gh pr merge 42",
        ):
            self.assertEqual(
                agy_adapter.permission_decision(policy, empty, command)[0],
                "deny",
            )

    def test_private_role_pairs_are_fixed(self) -> None:
        self.assertEqual(
            agy_adapter.DISPATCH_ROLE_SETTINGS["qa"],
            {"model": "gemini-3.8-flash-high", "effort": "high"},
        )
        self.assertEqual(
            agy_adapter.DISPATCH_ROLE_SETTINGS["dev"],
            {"model": "gemini-3.8-flash-medium", "effort": "medium"},
        )

    def test_command_permission_matching_and_precedence(self) -> None:
        project = agy_adapter.normalize_permission_surface(
            {
                "allow": ["command(git)", "command(rg)"],
                "deny": ["command(git push)"],
                "ask": [],
            }
        )
        global_rules = agy_adapter.normalize_permission_surface({"allow": ["command(rg)"], "deny": [], "ask": []})
        self.assertEqual(
            agy_adapter.permission_decision(global_rules, project, "rg -n TODO src"),
            ("allow", "command(rg)", "project"),
        )
        self.assertEqual(
            agy_adapter.permission_decision(
                global_rules,
                project,
                "git push origin main",
            ),
            ("deny", "command(git push)", "project"),
        )

    def test_global_project_conflict_and_narrow_task_contract_name_effective_source(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_policy_observation"]["permissions"]["deny"].append("command(git log)")
        profile["project_permissions"]["deny"].append("command(git log)")
        profile["task_commands"]["allow"].append("git log --oneline")
        report = agy_adapter.project_policy_report(profile)
        check = next(item for item in report["task_command_checks"] if item["command"] == "git log --oneline")
        self.assertEqual((check["decision"], check["source"], check["matched_rule"]), ("deny", "project", "command(git log)"))

        profile["task_commands"]["allow"].remove("git log --oneline")
        profile["task_commands"]["deny"].append("git log --oneline")
        report = agy_adapter.project_policy_report(profile)
        check = next(item for item in report["task_command_checks"] if item["command"] == "git log --oneline")
        self.assertEqual((check["decision"], check["source"]), ("deny", "task_contract"))

    def test_expected_soft_denied_canary_is_narrow_and_attempted_once(self) -> None:
        probe = agy_adapter.assignment_guard.SAFE_DENIAL_CANARY

        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "soft-deny-explicit",
        )
        profile["task_contract"]["expected_soft_denied_commands"] = [probe]
        profile["task_commands"]["allow"].append(probe)

        report = agy_adapter.project_policy_report(profile)
        check = next(
            item
            for item in report["task_command_checks"]
            if item["command"] == probe
        )
        self.assertTrue(report["dispatch_ready"], report["blockers"])
        self.assertEqual(check["expected"], "soft-denied")
        self.assertEqual(check["decision"], "deny")

        prompt = agy_adapter.render_prompt(
            profile,
            "soft-deny-explicit",
            "expect the safe negative control",
            agy_adapter.frozen_task_state(profile, "soft-deny-explicit"),
        )
        self.assertIn(probe, prompt)
        self.assertRegex(prompt, r"(?is)(attempt|run).*exactly once")
        self.assertRegex(prompt, r"(?is)(do not|never).*retry")

        normal_ask = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "normal-ask",
        )
        normal_ask["task_commands"]["allow"].append("printf normal-ask")
        normal_report = agy_adapter.project_policy_report(normal_ask)
        self.assertFalse(normal_report["dispatch_ready"])
        self.assertIn("resolves ask", " ".join(normal_report["blockers"]))

        bounded_write = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "write-probe",
        )
        bounded_write["mode"] = "bounded-write"
        design = self.repo_a / "design.md"
        design.write_text("frozen design\n")
        bounded_write["task_contract"]["kind"] = "implementation"
        bounded_write["task_contract"]["design_inputs"] = [
            {"path": str(design), "sha256": agy_adapter.sha256(design)}
        ]
        bounded_write["allowed_repo_writes"] = [".gitignore"]
        bounded_write["task_contract"]["expected_soft_denied_commands"] = [probe]
        bounded_write["task_commands"]["allow"].append(probe)
        bounded_path = self.root / "bounded-soft-deny.json"
        bounded_path.write_text(json.dumps(bounded_write))
        with self.assertRaisesRegex(SystemExit, "mode=measure-only"):
            agy_adapter.load_profile(str(bounded_path))

        unsafe_canary = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "unsafe-probe",
        )
        unsafe = "printf unsafe-negative-control"
        unsafe_canary["task_contract"]["expected_soft_denied_commands"] = [unsafe]
        unsafe_canary["task_commands"]["allow"].append(unsafe)
        unsafe_path = self.root / "unsafe-canary.json"
        unsafe_path.write_text(json.dumps(unsafe_canary))
        with self.assertRaisesRegex(SystemExit, "fixed safe denial canary"):
            agy_adapter.load_profile(str(unsafe_path))

    def test_expected_soft_denial_audit_rejects_execution_or_wrong_count(self) -> None:
        probe = agy_adapter.assignment_guard.SAFE_DENIAL_CANARY
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "soft-deny-audit",
        )
        profile["task_contract"]["expected_soft_denied_commands"] = [probe]

        denied = [{"command": probe, "status": 7, "step": 1}]
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(profile, denied),
            [],
        )

        executed = [{"command": probe, "status": 3, "step": 1}]
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(profile, executed)[0][
                "reason"
            ],
            "executed-instead-of-denied",
        )
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(profile, [*denied, *denied])[
                0
            ]["reason"],
            "attempt-count",
        )

        raw_denial = {
            "tool_calls": [{"command": probe, "step_index": 10}],
            "denied_tool_steps": [10],
        }
        guard_denial = [
            {"command": probe, "step_index": 10, "decision": "deny"}
        ]
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(
                profile,
                executed,
                inspected=raw_denial,
                guard_events=guard_denial,
            ),
            [],
        )
        missing_raw_denial = dict(raw_denial)
        missing_raw_denial["denied_tool_steps"] = []
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(
                profile,
                executed,
                inspected=missing_raw_denial,
                guard_events=guard_denial,
            )[0]["reason"],
            "raw-stream-not-denied",
        )
        nonterminal_raw_denial = {
            "tool_calls": [
                {"command": probe, "step_index": 10},
                {"command": "pwd", "step_index": 12},
            ],
            "denied_tool_steps": [10],
        }
        self.assertEqual(
            agy_adapter.expected_soft_denial_failures(
                profile,
                executed,
                inspected=nonterminal_raw_denial,
                guard_events=guard_denial,
            )[0]["reason"],
            "raw-stream-not-last",
        )

    def test_status_requires_current_project_policy(self) -> None:
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "status-policy",
        )
        settings = json.loads(self.settings.read_text())
        settings.pop("allowNonWorkspaceAccess")
        self.settings.write_text(json.dumps(settings))

        with patch("builtins.print") as output:
            agy_adapter.status(profile)
        self.assertEqual(output.call_count, 0)

    def test_status_reports_executed_negative_control(self) -> None:
        task_key = "status-control-failure"
        probe = agy_adapter.assignment_guard.SAFE_DENIAL_CANARY
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            task_key,
        )
        profile["task_contract"]["expected_soft_denied_commands"] = [probe]
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        evidence = {
            "version": agy_adapter.RUN_EVIDENCE_VERSION,
            "audit_contract_version": agy_adapter.AUDIT_CONTRACT_VERSION,
            "task_key": task_key,
            "delivery_status": "reported",
            "dispatch_role": profile["dispatch_role"],
            "model": profile["model"],
            "effort": profile["effort"],
            "backend_resolution": profile["backend_resolution"],
        }
        (runs / f"{task_key}.evidence.json").write_text(json.dumps(evidence))

        with (
            patch.object(agy_adapter, "require_project_ready"),
            patch.object(agy_adapter, "load_snapshot", return_value={}),
            patch.object(agy_adapter, "assert_run_evidence", return_value=evidence),
            patch.object(
                agy_adapter,
                "audit_task_commands",
                return_value=[{"command": probe, "status": 3, "step": 9}],
            ),
            patch("builtins.print") as output,
        ):
            agy_adapter.status(profile)

        rendered = "\n".join(
            str(call.args[0]) for call in output.call_args_list if call.args
        )
        self.assertIn("ISOLATION CONTROL FAILED", rendered)

    def test_permission_digest_detects_midrun_global_drift(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        snapshot = {
            "permission_state_digest": agy_adapter.permission_state_digest(
                profile
            )
        }
        settings = json.loads(self.settings.read_text())
        settings["permissions"]["allow"].append("command(cargo test)")
        self.settings.write_text(json.dumps(settings))
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.assert_permission_state_unchanged(profile, snapshot)
        self.assertIn("permission state changed", str(caught.exception))

    def test_permission_digest_detects_tool_mode_drift(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "mode-drift")
        snapshot = {
            "permission_state_digest": agy_adapter.permission_state_digest(profile)
        }
        settings = json.loads(self.settings.read_text())
        settings["toolPermission"] = "proceed-in-sandbox"
        self.settings.write_text(json.dumps(settings))
        with self.assertRaisesRegex(SystemExit, "permission state changed"):
            agy_adapter.assert_permission_state_unchanged(profile, snapshot)

    def test_permission_digest_detects_global_assignment_guard_drift(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "global-guard-drift")
        snapshot = {
            "permission_state_digest": agy_adapter.permission_state_digest(profile)
        }
        self.global_hooks.write_text(
            json.dumps(
                {
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: (
                        agy_adapter.global_assignment_guard_hook_config()
                    ),
                    "passive-lifecycle": {"PostToolUse": []},
                }
            )
        )

        with self.assertRaisesRegex(SystemExit, "permission state changed"):
            agy_adapter.assert_permission_state_unchanged(profile, snapshot)

    def test_global_pretool_hooks_are_bound_and_unknown_hooks_fail_closed(self) -> None:
        hook_path = agy_adapter.GLOBAL_HOOK_PATHS[0]
        hook_path.write_text(
            json.dumps(
                {
                    "cap-agent-guard": {
                        "PreToolUse": [{"matcher": "run_command", "hooks": []}]
                    },
                    "passive-lifecycle": {"PostToolUse": []},
                }
            )
        )
        observed = agy_adapter.global_hook_observation()
        self.assertEqual(observed[0]["active_pretool"], ["cap-agent-guard"])

        hook_path.write_text(
            json.dumps(
                {
                    "unknown-guard": {
                        "PreToolUse": [{"matcher": "*", "hooks": [{}]}]
                    }
                }
            )
        )
        with self.assertRaisesRegex(SystemExit, "unsupported user-global"):
            agy_adapter.global_hook_observation()

    def test_project_observation_must_match_persistent_root(self) -> None:
        profile = self.profile(self.repo_b, "project-a", "3")
        profile["project_policy_observation"]["project_root"] = str(self.repo_a)
        path = self.root / "wrong-project-root.json"
        path.write_text(json.dumps(profile))
        loaded = agy_adapter.load_profile(str(path))
        report = agy_adapter.project_policy_report(loaded)
        self.assertFalse(report["dispatch_ready"])
        self.assertIn("Project policy observation root does not match", " ".join(report["blockers"]))

    def test_existing_app_lumen_project_reuses_linked_task_worktree_without_creation(self) -> None:
        scope = self.root / "app_lumen"
        scope.mkdir()
        self.init_git_repo(scope)
        worker_root = self.add_nested_worktree(scope, "issue-77")
        profile = self.profile(
            worker_root,
            "app_lumen",
            "77",
            project_root=scope,
        )
        path = self.root / "shared-profile.json"
        path.write_text(json.dumps(profile))

        loaded = agy_adapter.load_profile(str(path))
        report = agy_adapter.project_policy_report(loaded)

        self.assertTrue(report["dispatch_ready"])
        self.assertEqual(report["worktree_scope"]["mode"], "in-project")
        self.assertEqual(
            agy_adapter.agy_command(loaded, None),
            [
                "agy",
                "--project",
                loaded["agy_project_id"],
                "--sandbox",
                "--output-format",
                "stream-json",
            ],
        )
        self.assertEqual(
            agy_adapter.dispatch_contract(loaded)["agy_project_root"],
            loaded["agy_project_root"],
        )
        command = agy_adapter.agy_command(loaded, None)
        self.assertNotIn("--add-dir", command)
        self.assertNotIn("--new-project", command)

    def test_worktree_readiness_rejects_project_root_as_task_root(self) -> None:
        profile = self.profile(
            self.project_a,
            "project-a",
            "root-is-scope",
            project_root=self.project_a,
        )
        report = agy_adapter.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn(
            "root must be a distinct task worktree inside agy_project_root",
            report["blockers"],
        )

    def test_worktree_readiness_rejects_external_sibling_worktree(self) -> None:
        worker_root = self.root / "external-sibling-worktree"
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(worker_root), "HEAD"],
            cwd=self.project_a,
            check=True,
        )
        profile = self.profile(
            worker_root,
            "project-a",
            "external",
            project_root=self.project_a,
        )
        report = agy_adapter.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn(
            "root must be physically nested inside agy_project_root",
            report["blockers"],
        )

    def test_worktree_readiness_rejects_ordinary_nested_directory(self) -> None:
        ordinary = self.project_a / "ordinary-subdirectory"
        ordinary.mkdir()
        profile = self.profile(
            ordinary,
            "project-a",
            "ordinary",
            project_root=self.project_a,
        )
        report = agy_adapter.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn("root must be an exact Git worktree root", report["blockers"])

    def test_worktree_readiness_requires_git_worktree_registration(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "registered")

        with patch.object(
            agy_adapter,
            "registered_worktree_paths",
            return_value=[self.project_a.resolve()],
        ):
            report = agy_adapter.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn("root is absent from git worktree list", report["blockers"])

    def test_worktree_readiness_rejects_unignored_nested_worktree(self) -> None:
        worker_root = self.project_a / "unignored-worktrees" / "issue-78"
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(worker_root), "HEAD"],
            cwd=self.project_a,
            check=True,
        )
        profile = self.profile(
            worker_root,
            "project-a",
            "unignored",
            project_root=self.project_a,
        )
        report = agy_adapter.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn(
            "the persistent Project root must ignore the nested task worktree path",
            report["blockers"],
        )

    def test_shared_project_rejects_foreign_worktree_and_state_inside_scope(self) -> None:
        scope = self.root / "persistent-project-root"
        scope.mkdir()
        self.init_git_repo(scope)
        worker_root = self.root / "foreign-worktree"
        worker_root.mkdir()
        self.init_git_repo(worker_root)
        profile = self.profile(worker_root, "project-shared", "88")
        profile["agy_project_root"] = str(scope)
        path = self.root / "shared-invalid-profile.json"

        path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "same Git repository"):
            agy_adapter.load_profile(str(path))

        profile["root"] = str(scope)
        profile["state_dir"] = str(scope / "controller-state")
        path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "state_dir must be outside"):
            agy_adapter.load_profile(str(path))

    def test_shared_project_scope_baseline_detects_persistent_root_write(self) -> None:
        scope = self.root / "persistent-project-root"
        scope.mkdir()
        self.init_git_repo(scope)
        worker_root = self.add_nested_worktree(scope, "issue-99")
        profile = self.profile(
            worker_root,
            "project-shared",
            "99",
            project_root=scope,
        )
        baseline = {"project_scope_baseline": agy_adapter.project_scope_baseline(profile)}

        agy_adapter.assert_project_scope_unchanged(profile, baseline)
        (scope / ".gitignore").write_text("changed\n")
        with self.assertRaisesRegex(SystemExit, "persistent AGY Project worktree changed"):
            agy_adapter.assert_project_scope_unchanged(profile, baseline)

    def test_sibling_worktree_baseline_detects_hidden_nested_write(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling")
        profile = self.profile(self.repo_a, "project-a", "sibling-baseline")
        snapshot = {
            "sibling_worktree_baselines": agy_adapter.sibling_worktree_baselines(
                profile
            )
        }

        agy_adapter.assert_sibling_worktrees_unchanged(profile, snapshot)
        (sibling / ".gitignore").write_text("changed by escaped worker\n")

        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_adapter.assert_sibling_worktrees_unchanged(profile, snapshot)

    def test_project_scope_baseline_detects_git_config_write(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "project-git-admin")
        snapshot = {
            "project_scope_baseline": agy_adapter.project_scope_baseline(profile)
        }

        subprocess.run(
            ["git", "config", "agy.probe", "mutated"],
            cwd=self.project_a,
            check=True,
        )
        with self.assertRaisesRegex(SystemExit, "persistent AGY Project"):
            agy_adapter.assert_project_scope_unchanged(profile, snapshot)

    def test_project_scope_baseline_detects_shallow_control_write(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "project-shallow-admin")
        snapshot = {
            "project_scope_baseline": agy_adapter.project_scope_baseline(profile)
        }
        common = agy_adapter.git_common_dir(self.project_a)
        self.assertIsNotNone(common)
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.project_a,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        (common / "shallow").write_text(head + "\n")

        with self.assertRaisesRegex(SystemExit, "persistent AGY Project"):
            agy_adapter.assert_project_scope_unchanged(profile, snapshot)

    def test_project_scope_ignores_only_codex_turn_diff_refs(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "project-turn-diff")
        snapshot = {
            "project_scope_baseline": agy_adapter.project_scope_baseline(profile)
        }
        common = agy_adapter.git_common_dir(self.project_a)
        self.assertIsNotNone(common)
        transient = common / "refs" / "codex" / "turn-diffs" / "captures" / "1"
        transient.mkdir(parents=True)
        (transient / "base").write_text("controller-only\n")

        agy_adapter.assert_project_scope_unchanged(profile, snapshot)

        ordinary = common / "refs" / "executor-canary"
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.project_a,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        ordinary.write_text(head + "\n")
        with self.assertRaisesRegex(SystemExit, "persistent AGY Project"):
            agy_adapter.assert_project_scope_unchanged(profile, snapshot)

    def test_sibling_baseline_detects_git_pointer_write(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling-pointer")
        profile = self.profile(self.repo_a, "project-a", "sibling-pointer")
        snapshot = {
            "sibling_worktree_baselines": agy_adapter.sibling_worktree_baselines(
                profile
            )
        }

        pointer = sibling / ".git"
        pointer.write_text(pointer.read_text() + "\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_adapter.assert_sibling_worktrees_unchanged(profile, snapshot)

    def test_registered_worktree_index_digest_binds_skip_worktree_flag(self) -> None:
        before = agy_adapter.registered_worktree_index_digests(self.repo_a)
        subprocess.run(
            ["git", "update-index", "--skip-worktree", ".gitignore"],
            cwd=self.repo_a,
            check=True,
        )
        after = agy_adapter.registered_worktree_index_digests(self.repo_a)

        self.assertNotEqual(before, after)

    def test_registered_worktree_index_digest_freezes_unavailable_row(self) -> None:
        stale = self.root / "stale-worktree"
        stale.mkdir()
        with patch.object(
            agy_adapter,
            "registered_worktree_paths",
            return_value=[self.repo_a, stale],
        ):
            before = agy_adapter.registered_worktree_index_digests(self.repo_a)

        self.assertRegex(before[str(self.repo_a)], r"^[0-9a-f]{64}$")
        self.assertEqual(before[str(stale)], "<unavailable>")

    def test_shared_object_store_allows_additions_but_requires_frozen_head(self) -> None:
        head = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            cwd=self.repo_a,
            text=True,
            capture_output=True,
            check=True,
        ).stdout.strip()
        snapshot = {
            "task_worktree_baseline": {"head": head},
            "project_scope_baseline": None,
            "sibling_worktree_baselines": {},
        }
        subprocess.run(
            ["git", "hash-object", "-w", "--stdin"],
            cwd=self.repo_a,
            input=b"controller turn snapshot\n",
            capture_output=True,
            check=True,
        )
        agy_adapter.assert_git_common_objects_intact(
            {"root": str(self.repo_a)},
            snapshot,
        )

        common = agy_adapter.git_common_dir(self.repo_a)
        self.assertIsNotNone(common)
        head_path = common / "objects" / head[:2] / head[2:]
        parked = head_path.with_name(head_path.name + ".missing")
        head_path.rename(parked)
        try:
            with self.assertRaisesRegex(SystemExit, "HEAD object graph"):
                agy_adapter.assert_git_common_objects_intact(
                    {"root": str(self.repo_a)},
                    snapshot,
                )
        finally:
            parked.rename(head_path)

    def test_sibling_worktree_baseline_hashes_ignored_bytes_and_caches(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling-ignored")
        gitignore = sibling / ".gitignore"
        gitignore.write_text(gitignore.read_text() + ".secret\n")
        (sibling / ".secret").write_text("private baseline\n")
        (sibling / ".venv").mkdir()
        (sibling / ".venv" / "marker").write_text("cache baseline\n")
        profile = self.profile(self.repo_a, "project-a", "sibling-ignored")
        snapshot = {
            "sibling_worktree_baselines": agy_adapter.sibling_worktree_baselines(
                profile
            )
        }

        (sibling / ".secret").write_text("escaped secret write\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_adapter.assert_sibling_worktrees_unchanged(profile, snapshot)

        (sibling / ".secret").write_text("private baseline\n")
        (sibling / ".venv" / "marker").write_text("escaped cache write\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_adapter.assert_sibling_worktrees_unchanged(profile, snapshot)

    def test_task_ignored_noncache_bytes_are_frozen_but_cache_bytes_are_exempt(self) -> None:
        gitignore = self.repo_a / ".gitignore"
        gitignore.write_text(gitignore.read_text() + ".secret\n")
        secret = self.repo_a / ".secret"
        secret.write_text("private baseline\n")
        cache = self.repo_a / ".venv" / "marker"
        cache.parent.mkdir()
        cache.write_text("cache baseline\n")
        profile = self.profile(self.repo_a, "project-a", "task-ignored")
        snapshot = {
            "task_worktree_baseline": agy_adapter.repository_worktree_baseline(
                self.repo_a
            )
        }

        cache.write_text("rebuild cache drift\n")
        agy_adapter.assert_task_ignored_noncache_unchanged(profile, snapshot)
        secret.write_text("escaped secret write\n")
        with self.assertRaisesRegex(SystemExit, "ignored non-cache"):
            agy_adapter.assert_task_ignored_noncache_unchanged(profile, snapshot)

    def test_injected_prompt_bytes_are_bound_into_the_dispatch_contract(self) -> None:
        prompt = self.root / "round.md"
        prompt.write_text("first frozen instruction\n")
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["inject_prompt_file"] = str(prompt)
        profile["external_payload_consent"]["approved_payload_classes"].append(
            "injected_prompt"
        )
        path = self.root / "injected-profile.json"
        path.write_text(json.dumps(profile))

        loaded = agy_adapter.load_profile(str(path))
        before = agy_adapter.dispatch_contract(loaded)
        self.assertEqual(
            before["inject_prompt_file_sha256"], agy_adapter.sha256(prompt)
        )

        prompt.write_text("changed after snapshot\n")
        reloaded = agy_adapter.load_profile(str(path))
        self.assertNotEqual(before, agy_adapter.dispatch_contract(reloaded))

    def test_prompt_hash_changes_only_for_ticketed_resume_contracts(self) -> None:
        prompt = self.root / "round-verify.md"
        prompt.write_text("frozen instruction\n")
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["inject_prompt_file"] = str(prompt)
        profile["external_payload_consent"]["approved_payload_classes"].append(
            "injected_prompt"
        )
        path = self.root / "round-verify-profile.json"
        path.write_text(json.dumps(profile))

        loaded = agy_adapter.load_profile(str(path))
        snapshot_contract = agy_adapter.dispatch_contract(loaded)
        self.assertTrue(
            agy_adapter.snapshot_contract_matches(loaded, snapshot_contract)
        )

        prompt.write_text("different frozen instruction\n")
        reloaded = agy_adapter.load_profile(str(path))
        self.assertFalse(
            agy_adapter.snapshot_contract_matches(reloaded, snapshot_contract)
        )
        self.assertTrue(
            agy_adapter.snapshot_contract_matches(
                reloaded,
                snapshot_contract,
                allow_prompt_hash_change=True,
            )
        )

        reloaded["task_contract"]["intent"] = "changed authority"
        self.assertFalse(
            agy_adapter.snapshot_contract_matches(reloaded, snapshot_contract)
        )

    def test_extracts_last_line_anchored_exec_report_after_chatter(self) -> None:
        raw = (
            "progress chatter\n"
            "## EXEC REPORT\nDRAFT\n"
            "more work\n"
            "## EXEC REPORT\nPASS\n"
        )
        self.assertEqual(
            agy_adapter.extract_exec_report(raw),
            "## EXEC REPORT\nPASS\n",
        )
        self.assertIsNone(
            agy_adapter.extract_exec_report(
                "prose mentioning ## EXEC REPORT but no heading"
            )
        )

    def test_stream_json_uses_official_event_envelopes_and_terminal_response(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000090"
        stream = self.root / "valid.stream.jsonl"
        stderr = self.root / "valid.stderr.log"
        self.write_stream(
            stream,
            conversation_id,
            response="## EXEC REPORT\nPASS\n",
            steps=[
                {
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": conversation_id,
                        "step_index": 1,
                        "state": "DONE",
                        "step_type": "agent_response",
                        "text_delta": "untrusted chatter",
                    },
                }
            ],
        )
        stderr.write_text("diagnostic only\n")

        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )

        self.assertEqual(inspected["delivery_status"], "reported")
        self.assertEqual(inspected["conversation_id"], conversation_id)
        self.assertEqual(inspected["response"], "## EXEC REPORT\nPASS\n")
        self.assertEqual(inspected["exec_report"], "## EXEC REPORT\nPASS\n")

    def test_stream_json_binds_omitted_command_cwd_to_verified_init_cwd(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000091"
        stream = self.root / "implicit-cwd.stream.jsonl"
        stderr = self.root / "implicit-cwd.stderr.log"
        self.write_stream(
            stream,
            conversation_id,
            steps=[
                {
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": conversation_id,
                        "step_index": 2,
                        "state": "ACTIVE",
                        "step_type": "tool",
                        "tool_name": "run_command",
                        "tool_info": {
                            "name": "run_command",
                            "parameters": {"CommandLine": "pwd"},
                        },
                    },
                },
                {
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": conversation_id,
                        "step_index": 2,
                        "state": "DONE",
                        "step_type": "tool",
                        "tool_name": "run_command",
                        "tool_info": {
                            "name": "run_command",
                            "parameters": {"CommandLine": "pwd"},
                        },
                    },
                },
            ],
        )
        stderr.write_text("")

        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )

        self.assertEqual(inspected["delivery_status"], "reported")
        self.assertEqual(
            inspected["tool_calls"],
            [
                {
                    "step_index": 2,
                    "conversation_id": conversation_id,
                    "tool_name": "run_command",
                    "tool_args": {
                        "CommandLine": "pwd",
                        "Cwd": str(self.repo_a.resolve()),
                    },
                    "tool_args_digest": agy_adapter.assignment_guard.canonical_digest(
                        {
                            "CommandLine": "pwd",
                            "Cwd": str(self.repo_a.resolve()),
                        }
                    ),
                    "command": "pwd",
                    "cwd": str(self.repo_a.resolve()),
                    "target_path": None,
                }
            ],
        )

    def test_stream_json_report_is_never_parsed_from_step_text(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000089"
        stream = self.root / "step-report.stream.jsonl"
        stderr = self.root / "step-report.stderr.log"
        self.write_stream(
            stream,
            conversation_id,
            response="no final report\n",
            steps=[
                {
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": conversation_id,
                        "step_index": 2,
                        "state": "DONE",
                        "step_type": "agent_response",
                        "text_delta": "## EXEC REPORT\nSPOOFED\n",
                    },
                }
            ],
        )
        stderr.write_text("")

        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )

        self.assertEqual(inspected["delivery_status"], "invalid-report")
        self.assertIsNone(inspected["exec_report"])

    def test_stream_json_rejects_invalid_init_and_result_shape(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000088"
        init = {
            "event": "init",
            "conversation_id": conversation_id,
            "init": {"cwd": str(self.repo_a.resolve()), "tools": []},
        }
        result = {
            "event": "result",
            "result": {
                "conversation_id": conversation_id,
                "status": "SUCCESS",
                "response": "## EXEC REPORT\nPASS\n",
            },
        }
        cases = {
            "missing-init": [result],
            "duplicate-init": [init, init, result],
            "result-before-init": [result, init],
            "missing-result": [init],
            "duplicate-result": [init, result, result],
            "event-after-result": [init, result, {"event": "step_update", "step_update": {}}],
        }
        stderr = self.root / "invalid-shape.stderr.log"
        stderr.write_text("")
        for label, events in cases.items():
            with self.subTest(label=label):
                stream = self.root / f"{label}.stream.jsonl"
                stream.write_text("".join(json.dumps(event) + "\n" for event in events))
                inspected = agy_adapter.inspect_headless_stream(
                    stream,
                    stderr,
                    expected_cwd=self.repo_a.resolve(),
                    requested_conversation_id=None,
                )
                self.assertEqual(inspected["delivery_status"], "invalid-stream")

    def test_stream_json_rejects_bad_ndjson_and_cwd(self) -> None:
        stderr = self.root / "bad.stderr.log"
        stderr.write_text("")
        malformed = self.root / "malformed.stream.jsonl"
        malformed.write_text('{"event":"init"}\n{bad json\n')
        inspected = agy_adapter.inspect_headless_stream(
            malformed,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "invalid-stream")

        wrong_cwd = self.root / "wrong-cwd.stream.jsonl"
        self.write_stream(
            wrong_cwd,
            "00000000-0000-0000-0000-000000000087",
            cwd=self.project_a,
        )
        inspected = agy_adapter.inspect_headless_stream(
            wrong_cwd,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "cwd-mismatch")

    def test_partial_stdout_cannot_extend_the_init_deadline(self) -> None:
        stream = self.root / "partial-init.stream.jsonl"
        stderr = self.root / "partial-init.stderr.log"
        code = (
            "import sys,time; "
            "sys.stdout.write('{\"event\":\"init\"'); "
            "sys.stdout.flush(); time.sleep(2)"
        )
        started = agy_adapter.time.monotonic()
        with patch.object(agy_adapter, "INIT_TIMEOUT_SECONDS", 0.15):
            agy_adapter.capture_headless_stream(
                [sys.executable, "-c", code],
                cwd=self.repo_a.resolve(),
                stream_path=stream,
                stderr_path=stderr,
            )
        elapsed = agy_adapter.time.monotonic() - started

        self.assertLess(elapsed, 1.0, "partial stdout blocked past the init deadline")
        source = inspect.getsource(agy_adapter.capture_headless_stream)
        self.assertIn("os.read", source)
        self.assertNotIn(".readline(", source)

    def test_wrong_cwd_stays_cwd_mismatch_after_early_termination(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000083"
        stream = self.root / "early-cwd.stream.jsonl"
        stderr = self.root / "early-cwd.stderr.log"
        marker = self.root / "must-not-run"
        init = json.dumps(
            {
                "event": "init",
                "conversation_id": conversation_id,
                "init": {"cwd": str(self.project_a.resolve()), "tools": []},
            }
        )
        code = (
            "import pathlib,sys,time; "
            f"sys.stdout.write({init!r} + '\\n'); sys.stdout.flush(); "
            "time.sleep(2); "
            f"pathlib.Path({str(marker)!r}).write_text('late')"
        )
        agy_adapter.capture_headless_stream(
            [sys.executable, "-c", code],
            cwd=self.repo_a.resolve(),
            stream_path=stream,
            stderr_path=stderr,
        )

        self.assertFalse(marker.exists())
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "cwd-mismatch")

    def test_permission_mode_mismatch_stops_before_late_work(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000082"
        stream = self.root / "early-mode.stream.jsonl"
        stderr = self.root / "early-mode.stderr.log"
        marker = self.root / "mode-must-not-run"
        init = json.dumps(
            {
                "event": "init",
                "conversation_id": conversation_id,
                "init": {
                    "cwd": str(self.repo_a.resolve()),
                    "tools": [],
                    "permission_mode": "always-proceed",
                },
            }
        )
        code = (
            "import pathlib,sys,time; "
            f"sys.stdout.write({init!r} + '\\n'); sys.stdout.flush(); "
            "time.sleep(2); "
            f"pathlib.Path({str(marker)!r}).write_text('late')"
        )
        agy_adapter.capture_headless_stream(
            [sys.executable, "-c", code],
            cwd=self.repo_a.resolve(),
            stream_path=stream,
            stderr_path=stderr,
            expected_permission_mode="request-review",
        )

        self.assertFalse(marker.exists())
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
            expected_permission_mode="request-review",
        )
        self.assertEqual(
            inspected["delivery_status"],
            "permission-mode-mismatch",
        )
        self.assertEqual(inspected["init_permission_mode"], "always-proceed")

    def test_stream_permission_mode_mismatch_wins_over_success(self) -> None:
        stream = self.root / "mode-mismatch.stream.jsonl"
        stderr = self.root / "mode-mismatch.stderr.log"
        stderr.write_text("")
        self.write_stream(
            stream,
            "00000000-0000-0000-0000-000000000081",
            permission_mode="proceed-in-sandbox",
        )
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
            expected_permission_mode="request-review",
        )
        self.assertEqual(
            inspected["delivery_status"],
            "permission-mode-mismatch",
        )

    def test_stream_json_rejects_conversation_mismatch_and_non_success_result(self) -> None:
        init_id = "00000000-0000-0000-0000-000000000086"
        other_id = "00000000-0000-0000-0000-000000000085"
        stderr = self.root / "mismatch.stderr.log"
        stderr.write_text("")
        stream = self.root / "mismatch.stream.jsonl"
        self.write_stream(stream, init_id, result_conversation_id=other_id)
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "conversation-mismatch")

        self.write_stream(stream, init_id)
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=other_id,
        )
        self.assertEqual(inspected["delivery_status"], "conversation-mismatch")

        for status in ("ERROR", "WAITING", "CANCELLED"):
            with self.subTest(status=status):
                self.write_stream(stream, init_id, status=status)
                inspected = agy_adapter.inspect_headless_stream(
                    stream,
                    stderr,
                    expected_cwd=self.repo_a.resolve(),
                    requested_conversation_id=None,
                )
                self.assertEqual(inspected["delivery_status"], "result-error")

    def test_stream_json_soft_deny_wins_over_success_and_exit_zero(self) -> None:
        conversation_id = "00000000-0000-0000-0000-000000000084"
        stream = self.root / "soft-deny.stream.jsonl"
        stderr = self.root / "soft-deny.stderr.log"
        self.write_stream(stream, conversation_id)
        stderr.write_text("Permission denied by sandbox policy\n")
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "soft-denied")

        self.assertTrue(
            agy_adapter.permission_denial_text(
                "tool call denied by pre-tool hook: frozen assignment denial"
            )
        )

        stderr.write_text("")
        tool_error = {
            "event": "step_update",
            "step_update": {
                "conversation_id": conversation_id,
                "step_index": 3,
                "state": "DONE",
                "step_type": "tool",
                "tool_name": "run_command",
                "tool_info": {
                    "name": "run_command",
                    "parameters": {"CommandLine": "printf denied"},
                    "error": {
                        "type": "permission_denied",
                        "message": "Command was denied by policy",
                    },
                },
            },
        }
        self.write_stream(stream, conversation_id, steps=[tool_error])
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
        )
        self.assertEqual(inspected["delivery_status"], "soft-denied")

    def test_guard_audit_covers_each_streamed_tool_request(self) -> None:
        task_key = "guard-audit"
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            task_key,
        )
        canary = agy_adapter.assignment_guard.SAFE_DENIAL_CANARY
        profile["task_contract"]["expected_soft_denied_commands"] = [canary]
        profile["task_commands"]["allow"].append(canary)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            task_key,
            "",
            runs,
        )
        policy, policy_digest = agy_adapter.assignment_guard.load_policy(
            artifacts["guard_policy"]
        )
        conversation_id = "00000000-0000-0000-0000-000000000080"
        steps = []
        for step, command in ((1, "pwd"), (2, canary)):
            args = {"CommandLine": command, "Cwd": str(self.repo_a.resolve())}
            payload = {
                "conversationId": conversation_id,
                "stepIdx": step,
                "workspacePaths": [str(self.project_a.resolve())],
                "modelName": profile["model"],
                "toolCall": {"name": "run_command", "args": args},
            }
            _decision, event = agy_adapter.assignment_guard.decide(
                policy,
                policy_digest,
                payload,
            )
            agy_adapter.assignment_guard.append_audit(
                artifacts["guard_audit"],
                event,
            )
            steps.extend(
                {
                    "event": "step_update",
                    "step_update": {
                        "conversation_id": conversation_id,
                        "step_index": step,
                        "state": state,
                        "step_type": "tool",
                        "tool_name": "run_command",
                        "tool_info": {
                            "name": "run_command",
                            "parameters": args,
                            **(
                                {
                                    "error": {
                                        "type": "permission_denied",
                                        "message": "Tool use denied by hook",
                                    }
                                }
                                if command == canary
                                else {}
                            ),
                        },
                    },
                }
                for state in ("ACTIVE", "DONE")
            )
        stream = runs / f"{task_key}.stream.jsonl"
        stderr = runs / f"{task_key}.stderr.log"
        stderr.write_text("")
        self.write_stream(stream, conversation_id, steps=steps)
        inspected = agy_adapter.inspect_headless_stream(
            stream,
            stderr,
            expected_cwd=self.repo_a.resolve(),
            requested_conversation_id=None,
            expected_permission_mode="request-review",
        )
        self.assertEqual(inspected["delivery_status"], "soft-denied")
        self.assertEqual(len(inspected["tool_calls"]), 2)
        guard_events = agy_adapter.assert_assignment_guard_audit(
            profile,
            task_key,
            inspected,
            artifacts,
        )
        reported_view = dict(inspected)
        reported_view["delivery_status"] = "reported"
        self.assertEqual(
            agy_adapter.guarded_delivery_status(reported_view, guard_events),
            "soft-denied",
        )

        records = [
            json.loads(line)
            for line in artifacts["guard_audit"].read_text().splitlines()
        ]
        self.assertEqual(inspected["denied_tool_steps"], [2])
        hook_default_record = dict(records[0])
        hook_default_record["tool_args_digest"] = "b" * 64
        artifacts["guard_audit"].write_text(
            "".join(
                json.dumps(record) + "\n"
                for record in [hook_default_record, records[1]]
            )
        )
        self.assertEqual(
            len(
                agy_adapter.assert_assignment_guard_audit(
                    profile,
                    task_key,
                    inspected,
                    artifacts,
                )
            ),
            2,
        )
        hook_default_record["tool_args_digest"] = "not-a-sha256"
        artifacts["guard_audit"].write_text(
            "".join(
                json.dumps(record) + "\n"
                for record in [hook_default_record, records[1]]
            )
        )
        with self.assertRaisesRegex(SystemExit, "audit identity mismatch"):
            agy_adapter.assert_assignment_guard_audit(
                profile,
                task_key,
                inspected,
                artifacts,
            )
        artifacts["guard_audit"].write_text(
            "".join(json.dumps(record) + "\n" for record in [*records, dict(records[0])])
        )
        self.assertEqual(
            len(
                agy_adapter.assert_assignment_guard_audit(
                    profile,
                    task_key,
                    inspected,
                    artifacts,
                )
            ),
            3,
        )
        extra_step = dict(records[0])
        extra_step["step_index"] = 99
        artifacts["guard_audit"].write_text(
            "".join(json.dumps(record) + "\n" for record in [*records, extra_step])
        )
        with self.assertRaisesRegex(SystemExit, "audit step identity"):
            agy_adapter.assert_assignment_guard_audit(
                profile,
                task_key,
                inspected,
                artifacts,
            )
        artifacts["guard_audit"].write_text(json.dumps(records[0]) + "\n")
        self.assertEqual(
            len(
                agy_adapter.assert_assignment_guard_audit(
                    profile,
                    task_key,
                    inspected,
                    artifacts,
                )
            ),
            1,
        )
        artifacts["guard_audit"].write_text("")
        with self.assertRaisesRegex(SystemExit, "missing streamed tool step 1"):
            agy_adapter.assert_assignment_guard_audit(
                profile,
                task_key,
                inspected,
                artifacts,
            )
        artifacts["guard_audit"].write_text(
            "".join(json.dumps(record) + "\n" for record in records)
        )
        records[0]["decision"] = "deny"
        artifacts["guard_audit"].write_text(
            "".join(json.dumps(record) + "\n" for record in records)
        )
        with self.assertRaisesRegex(SystemExit, "decision differs"):
            agy_adapter.assert_assignment_guard_audit(
                profile,
                task_key,
                inspected,
                artifacts,
            )

    def test_conversation_log_rejects_conflicting_injected_id(self) -> None:
        first = "00000000-0000-0000-0000-000000000001"
        injected = "00000000-0000-0000-0000-000000000002"
        log = self.root / "conversation-spoof.log"
        log.write_text(
            "ERROR: logging before google.Init: I0820 12:00:00.000000 "
            f"1 server.go:1074] Created conversation {first}\n"
            "ERROR: logging before google.Init: I0820 12:00:00.000001 "
            f"1 printmode.go:340] Print mode: conversation={first}, sending message\n"
            f"conversation={injected}\n"
        )

        with self.assertRaisesRegex(SystemExit, "conflicting conversation ids"):
            agy_adapter.conversation_id_from_log(log)

    def test_run_evidence_binds_prompt_log_report_contract_and_conversation(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "evidence")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        conversation_id = "00000000-0000-0000-0000-000000000099"
        (runs / "evidence.conversation").write_text(conversation_id + "\n")
        prompt = runs / "evidence.prompt.md"
        contract = runs / "evidence.contract.json"
        agy_log = runs / "evidence.agy.log"
        stream = runs / "evidence.stream.jsonl"
        stderr = runs / "evidence.stderr.log"
        response = runs / "evidence.response.md"
        normalized = runs / "evidence.report.md"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_adapter.dispatch_contract(profile)))
        agy_log.write_text(f"conversation={conversation_id}\n")
        self.write_stream(stream, conversation_id, response="## EXEC REPORT\nPASS\n")
        stderr.write_text("")
        response.write_text("## EXEC REPORT\nPASS\n")
        normalized.write_text("## EXEC REPORT\nPASS\n")
        guard_artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            "evidence",
            "",
            runs,
        )

        agy_adapter.write_run_evidence(
            profile=profile,
            task_key="evidence",
            suffix="",
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            stream_path=stream,
            stderr_path=stderr,
            response_path=response,
            guard_artifacts=guard_artifacts,
            normalized_report_path=normalized,
            snapshot_id="snapshot-evidence",
            exit_code=0,
            delivery_status="reported",
            init_permission_mode="request-review",
        )
        snapshot = {"snapshot_id": "snapshot-evidence"}
        evidence = agy_adapter.assert_run_evidence(profile, "evidence", snapshot)
        self.assertEqual(evidence["conversation_id"], conversation_id)

        guard_artifacts["guard_audit"].write_text("{}\n")
        with self.assertRaisesRegex(SystemExit, "digest mismatch: guard_audit"):
            agy_adapter.assert_run_evidence(profile, "evidence", snapshot)
        with (
            patch.object(agy_adapter, "load_snapshot", return_value=snapshot),
            patch("builtins.print") as output,
        ):
            agy_adapter.status(profile)
        self.assertIn("INVALID EVIDENCE", output.call_args.args[0])
        guard_artifacts["guard_audit"].write_text("")

        with self.assertRaisesRegex(SystemExit, "different snapshot"):
            agy_adapter.assert_run_evidence(
                profile,
                "evidence",
                {"snapshot_id": "newer-snapshot"},
            )

        normalized.write_text("## EXEC REPORT\nTAMPERED\n")
        with self.assertRaisesRegex(SystemExit, "digest mismatch"):
            agy_adapter.assert_run_evidence(profile, "evidence", snapshot)

    def test_run_evidence_accepts_result_error_and_rejects_reclassification(
        self,
    ) -> None:
        task_key = "failed-evidence"
        conversation_id = "00000000-0000-0000-0000-000000000091"
        snapshot = {"snapshot_id": "snapshot-failed-evidence"}
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        self.write_conversation(
            conversation_id,
            [(1, "pwd", str(self.repo_a.resolve()))],
        )
        evidence_path, _ = self.write_failed_run_evidence(
            profile,
            task_key,
            conversation_id,
            snapshot["snapshot_id"],
        )

        evidence = agy_adapter.assert_run_evidence(profile, task_key, snapshot)
        self.assertEqual(evidence["exit_code"], 1)
        self.assertEqual(evidence["delivery_status"], "result-error")
        self.assertNotIn("normalized_report", evidence["files"])

        reclassified = json.loads(evidence_path.read_text())
        reclassified["delivery_status"] = "invalid-stream"
        evidence_path.write_text(json.dumps(reclassified, indent=2) + "\n")
        with self.assertRaisesRegex(SystemExit, "classification mismatch"):
            agy_adapter.assert_run_evidence(profile, task_key, snapshot)

    def test_attempt_selection_uses_canonical_ordinal_not_mtime(self) -> None:
        task_key = "attempt-order"
        conversation_id = "00000000-0000-0000-0000-000000000077"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        self.write_conversation(
            conversation_id,
            [(1, "pwd", str(self.repo_a.resolve()))],
        )
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-initial",
        )
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-resume",
            suffix=".resume",
        )
        initial_contract = runs / f"{task_key}.contract.json"
        initial_evidence = runs / f"{task_key}.evidence.json"
        resume_contract = runs / f"{task_key}.resume.contract.json"
        resume_evidence = runs / f"{task_key}.resume.evidence.json"
        os.utime(initial_contract, (4_000_000_000, 4_000_000_000))
        os.utime(initial_evidence, (4_000_000_000, 4_000_000_000))
        os.utime(resume_contract, (1, 1))
        os.utime(resume_evidence, (1, 1))

        self.assertEqual(
            agy_adapter.latest_round_contract(profile, task_key)[0],
            resume_contract.resolve(),
        )
        self.assertEqual(
            agy_adapter.latest_run_evidence(profile, task_key)[0],
            resume_evidence.resolve(),
        )
        self.assertEqual(
            agy_adapter.assert_complete_attempt_lineage(profile, task_key),
            [0, 1],
        )

    def test_attempt_lineage_rejects_gap_and_missing_artifact(self) -> None:
        task_key = "attempt-gap"
        conversation_id = "00000000-0000-0000-0000-000000000078"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-initial",
        )
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-gap",
            suffix=".resume.2",
        )

        with self.assertRaisesRegex(SystemExit, "not contiguous"):
            agy_adapter.assert_complete_attempt_lineage(profile, task_key)

        gap_suffix = ".resume.2"
        for ending in (
            ".prompt.md",
            ".contract.json",
            ".agy.log",
            ".stream.jsonl",
            ".stderr.log",
            ".response.md",
            ".report.md",
            ".evidence.json",
        ):
            (runs / f"{task_key}{gap_suffix}{ending}").unlink()
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-resume",
            suffix=".resume",
        )
        (runs / f"{task_key}.resume.prompt.md").unlink()
        with self.assertRaisesRegex(SystemExit, "one-to-one canonical artifacts"):
            agy_adapter.assert_complete_attempt_lineage(profile, task_key)

    def test_resume_evidence_uses_stream_lineage_but_still_hashes_agy_log(self) -> None:
        task_key = "resume-log-mismatch"
        conversation_id = "00000000-0000-0000-0000-000000000081"
        other_id = "00000000-0000-0000-0000-000000000082"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-initial",
        )
        evidence_path, _ = self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            "snapshot-resume",
            suffix=".resume",
        )
        agy_log = runs / f"{task_key}.resume.agy.log"
        agy_log.write_text(f"conversation={other_id}\n")
        evidence = json.loads(evidence_path.read_text())
        evidence["files"]["agy_log"]["sha256"] = agy_adapter.sha256(agy_log)
        evidence_path.write_text(json.dumps(evidence, indent=2) + "\n")

        verified = agy_adapter.assert_run_evidence(
            profile,
            task_key,
            {"snapshot_id": "snapshot-resume"},
        )
        self.assertEqual(verified["conversation_id"], conversation_id)

        agy_log.write_text(f"conversation={other_id}\ntampered diagnostics\n")
        with self.assertRaisesRegex(SystemExit, "digest mismatch: agy_log"):
            agy_adapter.assert_run_evidence(
                profile,
                task_key,
                {"snapshot_id": "snapshot-resume"},
            )

    def test_verify_result_error_attempt_writes_predecessor_marker(self) -> None:
        task_key = "verify-failed-delivery"
        conversation_id = "00000000-0000-0000-0000-000000000092"
        snapshot_id = "snapshot-verify-failed-delivery"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        state = Path(profile["state_dir"])
        oracle = state / "oracles" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("frozen oracle\n")
        runs = state / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        self.write_conversation(
            conversation_id,
            [(1, "pwd", str(self.repo_a.resolve()))],
        )
        self.write_failed_run_evidence(
            profile,
            task_key,
            conversation_id,
            snapshot_id,
        )
        snapshot = self.failed_attempt_snapshot(
            profile,
            task_key,
            snapshot_id,
            None,
        )

        with (
            patch.object(agy_adapter, "require_project_ready"),
            patch.object(agy_adapter, "load_snapshot", return_value=snapshot),
            patch.object(agy_adapter, "assert_snapshot_identity"),
            patch.object(agy_adapter, "assert_injected_prompt_unchanged"),
            patch.object(agy_adapter, "assert_executed_round_contract"),
            patch.object(agy_adapter, "assert_worktree_scope_unchanged"),
            patch.object(agy_adapter, "assert_oracle_unchanged"),
            patch.object(agy_adapter, "assert_permission_state_unchanged"),
            patch.object(agy_adapter, "assert_project_scope_unchanged"),
            patch.object(agy_adapter, "assert_sibling_worktrees_unchanged"),
            patch.object(agy_adapter, "assert_ignored_paths_unchanged"),
            patch.object(agy_adapter, "assert_task_ignored_noncache_unchanged"),
            patch.object(agy_adapter, "assert_task_git_admin_unchanged"),
            patch.object(agy_adapter, "assert_git_common_objects_intact"),
            patch.object(
                agy_adapter,
                "assert_registered_worktree_indexes_unchanged",
            ),
            patch("builtins.print") as output,
        ):
            agy_adapter.verify(profile, task_key)

        rendered = "\n".join(
            str(call.args[0]) for call in output.call_args_list if call.args
        )
        self.assertIn("DELIVERY_FAILED_ISOLATION_VERIFIED", rendered)
        self.assertIn("delivery_status=result-error", rendered)
        marker_path = agy_adapter.verified_marker_path(profile, task_key)
        self.assertTrue(marker_path.is_file())
        marker = json.loads(marker_path.read_text())
        self.assertEqual(marker["delivery_status"], "result-error")
        self.assertEqual(marker["conversation_step_max"], 1)

    def test_verify_missing_conversation_attempt_remains_void(self) -> None:
        task_key = "verify-missing-conversation"
        snapshot_id = "snapshot-verify-missing-conversation"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        state = Path(profile["state_dir"])
        oracle = state / "oracles" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("frozen oracle\n")
        self.write_failed_run_evidence(
            profile,
            task_key,
            None,
            snapshot_id,
            delivery_status="invalid-stream",
        )
        snapshot = self.failed_attempt_snapshot(
            profile,
            task_key,
            snapshot_id,
            None,
        )

        with (
            patch.object(agy_adapter, "require_project_ready"),
            patch.object(agy_adapter, "load_snapshot", return_value=snapshot),
            patch.object(agy_adapter, "assert_snapshot_identity"),
            patch.object(agy_adapter, "assert_injected_prompt_unchanged"),
            patch.object(agy_adapter, "assert_executed_round_contract"),
            patch.object(agy_adapter, "assert_worktree_scope_unchanged"),
            patch.object(agy_adapter, "assert_oracle_unchanged"),
            patch.object(agy_adapter, "assert_permission_state_unchanged"),
            patch.object(agy_adapter, "assert_project_scope_unchanged"),
            patch.object(agy_adapter, "assert_sibling_worktrees_unchanged"),
            patch.object(agy_adapter, "assert_ignored_paths_unchanged"),
            patch.object(agy_adapter, "assert_task_ignored_noncache_unchanged"),
            patch.object(agy_adapter, "assert_task_git_admin_unchanged"),
            patch.object(agy_adapter, "assert_git_common_objects_intact"),
            patch.object(
                agy_adapter,
                "assert_registered_worktree_indexes_unchanged",
            ),
            self.assertRaisesRegex(
                SystemExit,
                "delivery evidence is invalid.*conversation lineage",
            ),
        ):
            agy_adapter.verify(profile, task_key)

        self.assertFalse(
            agy_adapter.verified_marker_path(profile, task_key).exists()
        )

    def test_extracts_denied_run_command_from_protobuf_payload(self) -> None:
        payload = (
            b"\x08\x15garbage"
            b'{"CommandLine":"rg -c NATIVE_FUNC_ADDRS\\\\.with projects/mamba",'
            b'"Cwd":"/repo","WaitMsBeforeAsync":5000}'
            b"\x00trailer"
        )
        self.assertEqual(
            agy_adapter.extract_run_command_lines(payload),
            [r"rg -c NATIVE_FUNC_ADDRS\.with projects/mamba"],
        )

    def test_extracts_all_command_requests_with_whitespace_and_reordered_keys(self) -> None:
        payload = (
            b'prefix {"CommandLine":"pwd","Cwd":"/task"} middle '
            b'{ "Cwd": "/escaped", "CommandLine": "git status --short" } suffix'
        )

        self.assertEqual(
            agy_adapter.extract_run_command_requests(payload),
            [
                {"command": "pwd", "cwd": "/task"},
                {"command": "git status --short", "cwd": "/escaped"},
            ],
        )

    def write_conversation(
        self,
        conversation_id: str,
        commands: list[tuple],
    ) -> None:
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, "
                "step_type integer not null, "
                "status integer not null, "
                "step_payload blob"
                ")"
            )
            for entry in commands:
                idx, command = entry[:2]
                cwd = entry[2] if len(entry) == 3 else None
                request = {"CommandLine": command}
                if cwd is not None:
                    request["Cwd"] = cwd
                payload = (
                    b"prefix"
                    + json.dumps(request, separators=(",", ":")).encode()
                    + b"suffix"
                )
                connection.execute(
                    "insert into steps values (?, 15, 3, ?)",
                    (idx, payload),
                )
            connection.commit()
        finally:
            connection.close()

    def write_stream(
        self,
        path: Path,
        conversation_id: str,
        *,
        cwd: Path | None = None,
        response: str = "## EXEC REPORT\nPASS\n",
        status: str = "SUCCESS",
        result_conversation_id: str | None = None,
        steps: list[dict] | None = None,
        permission_mode: str = "request-review",
    ) -> None:
        events = [
            {
                "event": "init",
                "conversation_id": conversation_id,
                "init": {
                    "cwd": str((cwd or self.repo_a).resolve()),
                    "tools": ["run_command"],
                    "permission_mode": permission_mode,
                },
            },
            *(steps or []),
            {
                "event": "result",
                "result": {
                    "conversation_id": result_conversation_id or conversation_id,
                    "status": status,
                    "response": response,
                    "duration_seconds": 1.0,
                    "num_turns": 1,
                    "usage": {},
                },
            },
        ]
        path.write_text("".join(json.dumps(event) + "\n" for event in events))

    def test_conversation_reads_use_immutable_read_only_uri(self) -> None:
        conversation_id = "12345678-1234-1234-1234-123456789abc"
        self.write_conversation(conversation_id, [(0, "pwd")])
        real_connect = sqlite3.connect
        with patch.object(
            agy_adapter.sqlite3,
            "connect",
            wraps=real_connect,
        ) as connect:
            self.assertEqual(
                agy_adapter.conversation_step_max(conversation_id),
                0,
            )
        self.assertIn(
            "?mode=ro&immutable=1",
            connect.call_args.args[0],
        )

    def write_raw_conversation(
        self,
        conversation_id: str,
        steps: list[tuple[int, int, int, bytes]],
    ) -> None:
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, "
                "step_type integer not null, "
                "status integer not null, "
                "step_payload blob"
                ")"
            )
            connection.executemany(
                "insert into steps values (?, ?, ?, ?)",
                steps,
            )
            connection.commit()
        finally:
            connection.close()

    def append_raw_conversation_step(
        self,
        conversation_id: str,
        step: tuple[int, int, int, bytes],
    ) -> None:
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute("insert into steps values (?, ?, ?, ?)", step)
            connection.commit()
        finally:
            connection.close()

    def replace_raw_conversation_step_payload(
        self,
        conversation_id: str,
        idx: int,
        payload: bytes,
    ) -> None:
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "update steps set step_payload = ? where idx = ?",
                (payload, idx),
            )
            connection.commit()
        finally:
            connection.close()

    def write_successful_run_evidence(
        self,
        profile: dict,
        task_key: str,
        conversation_id: str,
        snapshot_id: str,
        *,
        suffix: str = "",
    ) -> tuple[Path, dict]:
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        prompt = runs / f"{task_key}{suffix}.prompt.md"
        contract = runs / f"{task_key}{suffix}.contract.json"
        agy_log = runs / f"{task_key}{suffix}.agy.log"
        stream = runs / f"{task_key}{suffix}.stream.jsonl"
        stderr = runs / f"{task_key}{suffix}.stderr.log"
        response = runs / f"{task_key}{suffix}.response.md"
        normalized = runs / f"{task_key}{suffix}.report.md"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_adapter.dispatch_contract(profile)))
        agy_log.write_text(f"conversation={conversation_id}\n")
        self.write_stream(stream, conversation_id, response="## EXEC REPORT\nPASS\n")
        stderr.write_text("")
        response.write_text("## EXEC REPORT\nPASS\n")
        normalized.write_text("## EXEC REPORT\nPASS\n")
        guard_artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            task_key,
            suffix,
            runs,
        )
        evidence_path = agy_adapter.write_run_evidence(
            profile=profile,
            task_key=task_key,
            suffix=suffix,
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            stream_path=stream,
            stderr_path=stderr,
            response_path=response,
            guard_artifacts=guard_artifacts,
            normalized_report_path=normalized,
            snapshot_id=snapshot_id,
            exit_code=0,
            delivery_status="reported",
            init_permission_mode="request-review",
        )
        return evidence_path, json.loads(evidence_path.read_text())

    def write_failed_run_evidence(
        self,
        profile: dict,
        task_key: str,
        conversation_id: str | None,
        snapshot_id: str,
        *,
        exit_code: int = 1,
        delivery_status: str = "result-error",
    ) -> tuple[Path, dict]:
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        prompt = runs / f"{task_key}.prompt.md"
        contract = runs / f"{task_key}.contract.json"
        agy_log = runs / f"{task_key}.agy.log"
        stream = runs / f"{task_key}.stream.jsonl"
        stderr = runs / f"{task_key}.stderr.log"
        response = runs / f"{task_key}.response.md"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_adapter.dispatch_contract(profile)))
        agy_log.write_text(
            f"conversation={conversation_id}\n"
            if conversation_id is not None
            else "agy transport log without a conversation id\n"
        )
        if conversation_id is None:
            stream.write_text('{"event":"result","result":{"status":"ERROR"}}\n')
        else:
            self.write_stream(
                stream,
                conversation_id,
                response="transport failed before a terminal report\n",
                status="ERROR",
            )
        stderr.write_text("transport failed\n")
        response.write_text("transport failed before a terminal report\n")
        guard_artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            task_key,
            "",
            runs,
        )
        evidence_path = agy_adapter.write_run_evidence(
            profile=profile,
            task_key=task_key,
            suffix="",
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            stream_path=stream,
            stderr_path=stderr,
            response_path=response,
            guard_artifacts=guard_artifacts,
            normalized_report_path=None,
            snapshot_id=snapshot_id,
            exit_code=exit_code,
            delivery_status=delivery_status,
            init_permission_mode=(
                "request-review" if conversation_id is not None else None
            ),
        )
        return evidence_path, json.loads(evidence_path.read_text())

    def failed_attempt_snapshot(
        self,
        profile: dict,
        task_key: str,
        snapshot_id: str,
        conversation_id: str | None,
    ) -> dict:
        root = Path(profile["root"])
        status = subprocess.run(
            [
                "git",
                "-C",
                str(root),
                "status",
                "--porcelain=v1",
                "--untracked-files=all",
            ],
            text=True,
            capture_output=True,
            check=True,
        ).stdout
        return {
            "snapshot_id": snapshot_id,
            "git_status": status,
            "manifest": agy_adapter.manifest(root, profile["snapshot_paths"]),
            "protected_artifacts": {},
            "protected_contents_base64": {},
            "writable_contents": {},
            "conversation_id": conversation_id,
            "conversation_step_floor": -1,
        }

    def test_requested_run_commands_rejects_hidden_malformed_command_marker(
        self,
    ) -> None:
        conversation_id = "conversation-malformed-command-marker"
        valid = json.dumps(
            {
                "CommandLine": "pwd",
                "Cwd": str(self.repo_a.resolve()),
            },
            separators=(",", ":"),
        ).encode()
        payload = b"prefix" + valid + b' hidden {"CommandLine":' + b"suffix"
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, payload)],
        )

        with self.assertRaisesRegex(
            SystemExit,
            "CommandLine|shell command|command request",
        ):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_rejects_command_marker_in_non_shell_step(
        self,
    ) -> None:
        conversation_id = "conversation-command-in-non-shell-step"
        payload = json.dumps(
            {
                "CommandLine": "pwd",
                "Cwd": str(self.repo_a.resolve()),
            },
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [(1, 7, 3, payload)],
        )

        with self.assertRaisesRegex(
            SystemExit,
            "CommandLine|shell command|step type",
        ):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_rejects_escaped_key_in_non_shell_step(
        self,
    ) -> None:
        conversation_id = "conversation-escaped-command-in-non-shell-step"
        payload = (
            b'{"\\u0043ommandLine":"git push origin main",'
            + json.dumps({"Cwd": str(self.repo_a.resolve())})[1:].encode()
        )
        self.write_raw_conversation(
            conversation_id,
            [(1, 7, 3, payload)],
        )

        with self.assertRaisesRegex(SystemExit, "shell command|step type"):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_rejects_balanced_escaped_and_bad_keys(
        self,
    ) -> None:
        conversation_id = "conversation-balanced-command-markers"
        valid_escaped = (
            b'{"\\u0043ommandLine":"pwd","Cwd":'
            + json.dumps(str(self.repo_a.resolve())).encode()
            + b"}"
        )
        payload = valid_escaped + b' {"CommandLine":"git push origin main"'
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, payload)],
        )

        with self.assertRaisesRegex(SystemExit, "completely parse every"):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_rejects_negative_step_index(self) -> None:
        conversation_id = "conversation-negative-step"
        payload = json.dumps(
            {"CommandLine": "git push origin main", "Cwd": str(self.repo_a)},
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [(-1, 15, 3, payload)],
        )

        with self.assertRaisesRegex(SystemExit, "negative step index"):
            agy_adapter.requested_run_commands(conversation_id)

    def test_conversation_digest_binds_non_payload_columns(self) -> None:
        conversation_id = "conversation-metadata-digest"
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob, metadata blob)"
            )
            connection.execute(
                "insert into steps values (?, ?, ?, ?, ?)",
                (1, 15, 3, b'{"CommandLine":"pwd","Cwd":"/task"}', b"before"),
            )
            connection.commit()
        finally:
            connection.close()

        before = agy_adapter.conversation_steps_digest(
            conversation_id,
            through_step=1,
        )
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "update steps set metadata = ? where idx = 1",
                (b"after",),
            )
            connection.commit()
        finally:
            connection.close()
        after = agy_adapter.conversation_steps_digest(
            conversation_id,
            through_step=1,
        )
        self.assertNotEqual(before, after)

    def test_requested_run_commands_rejects_unaudited_subtrajectory(self) -> None:
        conversation_id = "conversation-subtrajectory"
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob, "
                "has_subtrajectory integer, metadata blob, error_details blob, "
                "permissions blob, task_details blob, render_info blob, "
                "step_format blob)"
            )
            connection.execute(
                "insert into steps values (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                (1, 15, 3, b"{}", 1, None, None, None, None, None, None),
            )
            connection.commit()
        finally:
            connection.close()

        with self.assertRaisesRegex(SystemExit, "unaudited subtrajectory"):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_audits_metadata_only_command(self) -> None:
        conversation_id = "conversation-metadata-command"
        database = self.conversation_dir / f"{conversation_id}.db"
        command = {
            "CommandLine": "pwd",
            "Cwd": str(self.repo_a.resolve()),
        }
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob, metadata blob)"
            )
            connection.execute(
                "insert into steps values (?, ?, ?, ?, ?)",
                (1, 15, 3, b"{}", json.dumps(command).encode()),
            )
            connection.commit()
        finally:
            connection.close()

        self.assertEqual(
            agy_adapter.requested_run_commands(conversation_id),
            [
                {
                    "step": 1,
                    "status": 3,
                    "command": "pwd",
                    "cwd": str(self.repo_a.resolve()),
                }
            ],
        )

    def test_requested_run_commands_rejects_surface_mismatch(self) -> None:
        conversation_id = "conversation-command-surface-mismatch"
        database = self.conversation_dir / f"{conversation_id}.db"
        payload = {
            "CommandLine": "pwd",
            "Cwd": str(self.repo_a.resolve()),
        }
        metadata = {
            "CommandLine": "git push origin main",
            "Cwd": str(self.repo_a.resolve()),
        }
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob, metadata blob)"
            )
            connection.execute(
                "insert into steps values (?, ?, ?, ?, ?)",
                (
                    1,
                    15,
                    3,
                    json.dumps(payload).encode(),
                    json.dumps(metadata).encode(),
                ),
            )
            connection.commit()
        finally:
            connection.close()

        with self.assertRaisesRegex(SystemExit, "records disagree"):
            agy_adapter.requested_run_commands(conversation_id)

    def test_requested_run_commands_allows_non_shell_type15_row(self) -> None:
        conversation_id = "conversation-non-shell-type15"
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, b'{"tool":"read_file","path":"src/lib.rs"}')],
        )

        self.assertEqual(
            agy_adapter.requested_run_commands(conversation_id),
            [],
        )

    def test_requested_run_commands_allows_lifecycle_result_replay(self) -> None:
        conversation_id = "conversation-lifecycle-replay"
        request = json.dumps(
            {"CommandLine": "pwd", "Cwd": str(self.repo_a.resolve())},
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [
                (1, 15, 3, request),
                (2, 21, 3, request),
            ],
        )

        self.assertEqual(
            agy_adapter.requested_run_commands(conversation_id),
            [
                {
                    "step": 1,
                    "status": 3,
                    "command": "pwd",
                    "cwd": str(self.repo_a.resolve()),
                }
            ],
        )

    def test_requested_run_commands_allows_replica_multiplicity_difference(
        self,
    ) -> None:
        conversation_id = "conversation-replica-multiplicity"
        database = self.conversation_dir / f"{conversation_id}.db"
        request = {
            "CommandLine": "pwd",
            "Cwd": str(self.repo_a.resolve()),
        }
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob, metadata blob)"
            )
            connection.executemany(
                "insert into steps values (?, ?, ?, ?, ?)",
                [
                    (1, 15, 3, json.dumps(request).encode(), b"{}"),
                    (
                        2,
                        132,
                        3,
                        json.dumps([request, request]).encode(),
                        json.dumps(request).encode(),
                    ),
                ],
            )
            connection.commit()
        finally:
            connection.close()

        self.assertEqual(
            [
                item["command"]
                for item in agy_adapter.requested_run_commands(conversation_id)
            ],
            ["pwd"],
        )

    def test_requested_run_commands_allows_post_floor_prior_request_replay(
        self,
    ) -> None:
        conversation_id = "conversation-prior-request-replay"
        request = json.dumps(
            {"CommandLine": "pwd", "Cwd": str(self.repo_a.resolve())},
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [
                (1, 15, 3, request),
                (2, 21, 3, request),
            ],
        )

        self.assertEqual(
            agy_adapter.requested_run_commands(
                conversation_id,
                after_step=1,
            ),
            [],
        )

    def test_resnapshot_requires_exact_verified_conversation_predecessor(
        self,
    ) -> None:
        task_key = "verified-predecessor"
        conversation_id = "00000000-0000-0000-0000-000000000093"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        state = Path(profile["state_dir"])
        oracle = state / "oracles" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("frozen oracle\n")
        readiness = {"permission_state_digest": "permission-digest"}

        with (
            patch.object(
                agy_adapter,
                "require_project_ready",
                return_value=readiness,
            ),
            patch.object(agy_adapter, "assert_verified_predecessor") as gate,
        ):
            first_snapshot_path = agy_adapter.snapshot(profile, task_key)
        gate.assert_not_called()
        first_snapshot = agy_adapter.load_snapshot(profile, task_key)

        runs = state / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        command_payload = json.dumps(
            {
                "CommandLine": "pwd",
                "Cwd": str(self.repo_a.resolve()),
            },
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, command_payload)],
        )

        with (
            patch.object(
                agy_adapter,
                "require_project_ready",
                return_value=readiness,
            ),
            self.assertRaisesRegex(
                SystemExit,
                "verify the prior|verified predecessor",
            ),
        ):
            agy_adapter.snapshot(profile, task_key)

        evidence_path, evidence = self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            first_snapshot["snapshot_id"],
        )
        marker_path = agy_adapter.write_verified_marker(
            profile,
            task_key,
            first_snapshot,
            evidence_path,
            evidence,
        )
        self.assertEqual(
            marker_path,
            agy_adapter.verified_marker_path(profile, task_key),
        )
        agy_adapter.assert_verified_predecessor(
            profile,
            task_key,
            conversation_id,
        )

        forbidden_payload = json.dumps(
            {
                "CommandLine": "git push origin main",
                "Cwd": str(self.repo_a.resolve()),
            },
            separators=(",", ":"),
        ).encode()
        self.replace_raw_conversation_step_payload(
            conversation_id,
            1,
            forbidden_payload,
        )
        with self.assertRaisesRegex(SystemExit, "rows changed"):
            agy_adapter.assert_verified_predecessor(
                profile,
                task_key,
                conversation_id,
            )
        self.replace_raw_conversation_step_payload(
            conversation_id,
            1,
            command_payload,
        )
        agy_adapter.assert_verified_predecessor(
            profile,
            task_key,
            conversation_id,
        )

        with patch.object(
            agy_adapter,
            "require_project_ready",
            return_value=readiness,
        ):
            second_snapshot_path = agy_adapter.snapshot(profile, task_key)
        self.assertTrue(second_snapshot_path.is_file())

        self.append_raw_conversation_step(
            conversation_id,
            (2, 15, 3, command_payload),
        )
        with (
            patch.object(
                agy_adapter,
                "require_project_ready",
                return_value=readiness,
            ),
            self.assertRaisesRegex(
                SystemExit,
                "verified predecessor|new AGY steps|newer steps",
            ),
        ):
            agy_adapter.snapshot(profile, task_key)

    def test_verified_marker_bound_field_tampering_is_rejected(self) -> None:
        task_key = "verified-marker-tamper"
        conversation_id = "00000000-0000-0000-0000-000000000094"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        state = Path(profile["state_dir"])
        oracle = state / "oracles" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("frozen oracle\n")

        with patch.object(
            agy_adapter,
            "require_project_ready",
            return_value={"permission_state_digest": "permission-digest"},
        ):
            agy_adapter.snapshot(profile, task_key)
        snapshot = agy_adapter.load_snapshot(profile, task_key)

        runs = state / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        command_payload = json.dumps(
            {"CommandLine": "pwd", "Cwd": str(self.repo_a.resolve())},
            separators=(",", ":"),
        ).encode()
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, command_payload)],
        )
        evidence_path, evidence = self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            snapshot["snapshot_id"],
        )
        marker_path = agy_adapter.write_verified_marker(
            profile,
            task_key,
            snapshot,
            evidence_path,
            evidence,
        )
        agy_adapter.assert_verified_predecessor(
            profile,
            task_key,
            conversation_id,
        )

        marker = json.loads(marker_path.read_text())
        marker["backend_resolution"]["agy_version"] = "tampered version"
        marker_path.write_text(json.dumps(marker, indent=2) + "\n")

        with self.assertRaisesRegex(
            SystemExit,
            "verified-predecessor lineage changed",
        ):
            agy_adapter.assert_verified_predecessor(
                profile,
                task_key,
                conversation_id,
            )

    def test_task_operation_lock_rejects_concurrent_resume_or_snapshot(self) -> None:
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "operation-lock",
        )

        with agy_adapter.task_operation_lock(
            profile,
            "operation-lock",
            "resume",
        ):
            with self.assertRaisesRegex(SystemExit, "another snapshot.*launch.*verify"):
                with agy_adapter.task_operation_lock(
                    profile,
                    "operation-lock",
                    "snapshot",
                ):
                    self.fail("the second task operation lock must not be acquired")

    def test_project_lock_allows_measurements_but_blocks_bounded_write(self) -> None:
        measurement = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "measure-lock",
        )
        second_measurement = dict(measurement)
        second_measurement["state_dir"] = str(
            self.state_parent / measurement["agy_project_id"] / "measure-lock-2"
        )
        bounded_write = dict(measurement)
        bounded_write["mode"] = "bounded-write"
        bounded_write["allowed_repo_writes"] = ["README.md"]
        bounded_write["state_dir"] = str(
            self.state_parent / measurement["agy_project_id"] / "write-lock"
        )

        with agy_adapter.project_concurrency_lock(
            measurement, "measure-lock", "snapshot"
        ):
            with agy_adapter.project_concurrency_lock(
                second_measurement, "measure-lock-2", "snapshot"
            ):
                pass
            with self.assertRaisesRegex(SystemExit, "bounded-write task"):
                with agy_adapter.project_concurrency_lock(
                    bounded_write, "write-lock", "verify"
                ):
                    self.fail("a bounded write must not start during a measurement")

    def test_verify_keeps_project_lock_until_task_verification_finishes(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "verify-lock")
        events: list[str] = []

        @contextmanager
        def project_lock(*_args):
            events.append("project-enter")
            try:
                yield
            finally:
                events.append("project-exit")

        @contextmanager
        def task_lock(*_args):
            events.append("task-enter")
            try:
                yield
            finally:
                events.append("task-exit")

        def verify_body(*_args):
            events.append("verify")

        with (
            patch.object(agy_adapter, "project_concurrency_lock", side_effect=project_lock),
            patch.object(agy_adapter, "task_operation_lock", side_effect=task_lock),
            patch.object(agy_adapter, "verify_under_lock", side_effect=verify_body),
        ):
            agy_adapter.verify(profile, "verify-lock")
        self.assertEqual(
            events,
            ["project-enter", "task-enter", "verify", "task-exit", "project-exit"],
        )

    def test_agy_launch_cwd_is_exact_task_worktree(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "cwd")

        self.assertEqual(agy_adapter.agy_launch_cwd(profile), self.repo_a.resolve())

        profile["launch_cwd"] = "project-root"
        with self.assertRaisesRegex(SystemExit, "launch_cwd must be task-worktree"):
            agy_adapter.agy_launch_cwd(profile)

    def test_assignment_guard_artifact_binds_static_global_loader(self) -> None:
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "global-hook-contract",
        )
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            "global-hook-contract",
            "",
            runs,
        )
        self.assertEqual(
            json.loads(artifacts["guard_hooks"].read_text()),
            agy_adapter.assignment_hooks_config(),
        )
        self.assertTrue(artifacts["guard_hooks"].is_file())
        self.assertFalse((self.repo_a / ".agents").exists())

    def test_fixed_global_loader_uses_only_an_active_worktree_registration(self) -> None:
        task_key = "static-loader"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            task_key,
            "",
            runs,
        )
        payload = {
            "conversationId": "00000000-0000-0000-0000-000000000088",
            "stepIdx": 2,
            "workspacePaths": [str(self.project_a.resolve())],
            "modelName": profile["model"],
            "toolCall": {
                "name": "run_command",
                "args": {"CommandLine": "pwd", "Cwd": str(self.repo_a.resolve())},
            },
        }
        idle = subprocess.run(
            [str(agy_adapter.STATIC_GUARD_LOADER)],
            input=json.dumps(payload),
            text=True,
            capture_output=True,
            check=True,
        )
        self.assertEqual(json.loads(idle.stdout)["decision"], "allow")
        with agy_adapter.installed_active_guard_registration(
            profile,
            task_key,
            "",
            artifacts,
        ):
            active = subprocess.run(
                [str(agy_adapter.STATIC_GUARD_LOADER)],
                input=json.dumps(payload),
                text=True,
                capture_output=True,
                check=True,
            )
            self.assertEqual(json.loads(active.stdout)["decision"], "allow")
        records = artifacts["guard_audit"].read_text().splitlines()
        self.assertEqual(len(records), 1)
        self.assertEqual(json.loads(records[0])["command"], "pwd")

    def test_persistent_workspace_hook_payload_keeps_command_cwd_at_task_worktree(
        self,
    ) -> None:
        task_key = "persistent-workspace-cwd"
        profile = self.one_shot_profile(self.repo_a, "project-a", task_key)
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        artifacts = agy_adapter.materialize_assignment_guard(
            profile,
            task_key,
            "",
            runs,
        )
        policy, policy_digest = agy_adapter.assignment_guard.load_policy(
            artifacts["guard_policy"]
        )
        self.assertEqual(
            policy["workspace_roots"],
            [str(self.repo_a.resolve()), str(self.project_a.resolve())],
        )
        decision, event = agy_adapter.assignment_guard.decide(
            policy,
            policy_digest,
            {
                "conversationId": "00000000-0000-0000-0000-000000000089",
                "stepIdx": 2,
                "workspacePaths": [str(self.project_a.resolve())],
                "modelName": profile["model"],
                "toolCall": {
                    "name": "run_command",
                    "args": {
                        "CommandLine": "pwd",
                        "Cwd": str(self.project_a.resolve()),
                    },
                },
            },
        )
        self.assertEqual(decision["decision"], "deny")
        self.assertEqual(
            event["reason"],
            "run_command cwd does not equal the assigned worktree",
        )

    def test_fixed_pretool_chain_runs_loader_only_after_cap_allows(self) -> None:
        chain_path = SCRIPT.with_name("_agy_global_pretool_chain.py")
        spec = importlib.util.spec_from_file_location("agy_chain_test", chain_path)
        assert spec and spec.loader
        chain = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(chain)
        calls: list[tuple[str, ...]] = []

        def fake_invoke(command: tuple[str, ...], payload: bytes):
            calls.append(command)
            self.assertEqual(payload, b'{"toolCall":{"name":"run_command"}}')
            if command == chain.CAP_HOOK:
                return 0, b'{"decision":"allow","reason":"cap allow"}', b""
            return 0, b'{"decision":"allow","reason":"assignment allow"}', b""

        raw_out = io.BytesIO()
        raw_err = io.BytesIO()
        fake_stdin = io.TextIOWrapper(io.BytesIO(b'{"toolCall":{"name":"run_command"}}'))
        fake_stdout = io.TextIOWrapper(raw_out)
        fake_stderr = io.TextIOWrapper(raw_err)
        with (
            patch.object(chain, "invoke", side_effect=fake_invoke),
            patch.object(sys, "stdin", fake_stdin),
            patch.object(sys, "stdout", fake_stdout),
            patch.object(sys, "stderr", fake_stderr),
        ):
            chain.main()
            sys.stdout.flush()

        self.assertEqual(len(calls), 2)
        self.assertEqual(json.loads(raw_out.getvalue())["decision"], "allow")

    def test_fixed_pretool_chain_keeps_cap_denial_before_loader(self) -> None:
        chain_path = SCRIPT.with_name("_agy_global_pretool_chain.py")
        spec = importlib.util.spec_from_file_location("agy_chain_deny_test", chain_path)
        assert spec and spec.loader
        chain = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(chain)
        calls: list[tuple[str, ...]] = []

        def fake_invoke(command: tuple[str, ...], _payload: bytes):
            calls.append(command)
            return 0, b'{"decision":"deny","reason":"cap deny"}', b""

        raw_out = io.BytesIO()
        fake_stdin = io.TextIOWrapper(io.BytesIO(b"{}"))
        fake_stdout = io.TextIOWrapper(raw_out)
        with (
            patch.object(chain, "invoke", side_effect=fake_invoke),
            patch.object(sys, "stdin", fake_stdin),
            patch.object(sys, "stdout", fake_stdout),
        ):
            chain.main()
            sys.stdout.flush()

        self.assertEqual(calls, [chain.CAP_HOOK])
        self.assertEqual(json.loads(raw_out.getvalue())["decision"], "deny")

    def test_malformed_active_global_assignment_loader_fails_closed(self) -> None:
        self.global_hooks.write_text(
            json.dumps(
                {
                    agy_adapter.GLOBAL_ASSIGNMENT_GUARD_NAME: {
                        "PreToolUse": [
                            {
                                "matcher": "*",
                                "hooks": [
                                    {
                                        "type": "command",
                                        "command": "echo unsafe",
                                        "timeout": 10,
                                    }
                                ],
                            }
                        ]
                    }
                }
            )
        )
        with self.assertRaisesRegex(SystemExit, "frozen opt-in loader contract"):
            agy_adapter.global_hook_observation()

    def test_run_agent_launches_subprocess_from_exact_task_worktree(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "launch-cwd")
        state = Path(profile["state_dir"])
        oracle_dir = state / "oracles"
        oracle_dir.mkdir(parents=True)
        (oracle_dir / "launch-cwd.md").write_text("expected canary\n")

        def fake_capture(command: list[str], **kwargs: object) -> int:
            self.assertEqual(kwargs["cwd"], self.repo_a.resolve())
            self.assertIn("gemini-3.8-flash-high", command)
            self.assertNotIn("--add-dir", command)
            self.assertEqual(kwargs["expected_permission_mode"], "request-review")
            self.assertNotIn("env", kwargs)
            self.assertFalse((self.repo_a / ".agents").exists())
            log_path = Path(command[command.index("--log-file") + 1])
            log_path.write_text("conversation=00000000-0000-0000-0000-000000000001\n")
            self.write_stream(
                kwargs["stream_path"],
                "00000000-0000-0000-0000-000000000001",
                response="## EXEC REPORT\nPASS\n",
            )
            kwargs["stderr_path"].write_text("")
            return 0

        with ExitStack() as stack:
            stack.enter_context(
                patch.object(agy_adapter, "require_project_ready", return_value={})
            )
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "frozen_task_state",
                    return_value={"number": 1, "state": "OPEN"},
                )
            )
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "load_snapshot",
                    return_value={"snapshot_id": "snapshot-launch-cwd"},
                )
            )
            for name in (
                "assert_snapshot_identity",
                "assert_dispatch_contract_unchanged",
                "assert_initial_task_worktree_unchanged",
                "assert_permission_state_unchanged",
                "assert_project_scope_unchanged",
                "assert_sibling_worktrees_unchanged",
                "assert_ignored_paths_unchanged",
                "assert_task_ignored_noncache_unchanged",
                "assert_task_git_admin_unchanged",
                "assert_git_common_objects_intact",
                "assert_registered_worktree_indexes_unchanged",
                "assert_task_state_unchanged",
                "assert_verified_predecessor",
            ):
                stack.enter_context(patch.object(agy_adapter, name))
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "assert_oracle_unchanged",
                    return_value=oracle_dir / "launch-cwd.md",
                )
            )
            stack.enter_context(
                patch.object(agy_adapter, "audit_task_commands", return_value=[])
            )
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "validate_conversation_action",
                    return_value="00000000-0000-0000-0000-000000000001",
                )
            )
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "assert_complete_attempt_lineage",
                    return_value=[0],
                )
            )
            stack.enter_context(
                patch.object(agy_adapter, "render_prompt", return_value="prompt")
            )
            stack.enter_context(
                patch.object(
                    agy_adapter,
                    "capture_headless_stream",
                    side_effect=fake_capture,
                )
            )
            agy_adapter.run_agent(profile, "launch-cwd", resume=True)
        self.assertFalse((self.repo_a / ".agents").exists())

    def test_task_command_audit_accepts_exact_task_worktree_cwd(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "cwd-pass")
        state = Path(profile["state_dir"])
        runs = state / "runs"
        runs.mkdir(parents=True)
        (runs / "cwd-pass.conversation").write_text("conversation-cwd-pass\n")
        self.write_conversation(
            "conversation-cwd-pass",
            [(1, "pwd", str(self.repo_a.resolve()))],
        )

        audited = agy_adapter.audit_task_commands(
            profile,
            "cwd-pass",
            {"conversation_id": None, "conversation_step_floor": -1},
        )

        self.assertEqual(audited[0]["cwd"], str(self.repo_a.resolve()))

    def test_task_command_audit_rejects_non_task_worktree_cwd(self) -> None:
        cases = (
            ("missing", None),
            ("project-root", str(self.project_a)),
            ("sibling", str(self.repo_a.parent / "sibling-worktree")),
            ("outside", str(self.root / "outside-project")),
        )
        for label, cwd in cases:
            with self.subTest(cwd=label):
                task_key = f"cwd-{label}"
                conversation_id = f"conversation-{task_key}"
                profile = self.profile(self.repo_a, "project-a", task_key)
                state = Path(profile["state_dir"])
                runs = state / "runs"
                runs.mkdir(parents=True)
                (runs / f"{task_key}.conversation").write_text(
                    conversation_id + "\n"
                )
                command = (1, "pwd") if cwd is None else (1, "pwd", cwd)
                self.write_conversation(conversation_id, [command])

                with self.assertRaisesRegex(SystemExit, "task-worktree cwd"):
                    agy_adapter.audit_task_commands(
                        profile,
                        task_key,
                        {
                            "conversation_id": None,
                            "conversation_step_floor": -1,
                        },
                    )

    def test_task_command_audit_checks_only_post_snapshot_steps(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "7")
        state = Path(profile["state_dir"])
        runs = state / "runs"
        runs.mkdir(parents=True)
        (runs / "7.conversation").write_text("conversation-7\n")
        self.write_conversation(
            "conversation-7",
            [
                (1, "old command outside current contract"),
                (2, "pwd", str(self.repo_a.resolve())),
                (3, "rg -n TODO src", str(self.repo_a.resolve())),
            ],
        )
        audited = agy_adapter.audit_task_commands(
            profile,
            "7",
            {
                "conversation_id": "conversation-7",
                "conversation_step_floor": 1,
                "conversation_predecessor_digest": (
                    agy_adapter.conversation_steps_digest(
                        "conversation-7",
                        through_step=1,
                    )
                ),
            },
        )
        self.assertEqual(
            [item["command"] for item in audited],
            ["pwd", "rg -n TODO src"],
        )

    def test_task_command_audit_rejects_broader_project_command(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "8")
        state = Path(profile["state_dir"])
        runs = state / "runs"
        runs.mkdir(parents=True)
        (runs / "8.conversation").write_text("conversation-8\n")
        self.write_conversation(
            "conversation-8",
            [(1, "rg -n SECRET unrelated")],
        )
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.audit_task_commands(
                profile,
                "8",
                {
                    "conversation_id": None,
                    "conversation_step_floor": -1,
                },
            )
        self.assertIn("task-local exact allowlist", str(caught.exception))

    def test_prompt_separates_reusable_policy_from_ticket_commands(
        self,
    ) -> None:
        profile = self.profile(self.repo_a, "project-a", "4")
        profile["task_contract"]["instructions"] = (
            "Correct the controller-identified whitespace defect only."
        )
        prompt = agy_adapter.render_prompt(
            profile,
            "4",
            "oracle",
            {"number": 4, "state": "OPEN"},
        )
        self.assertIn("Persistent AGY permission-state digest", prompt)
        self.assertIn("authorized for this task", prompt)
        self.assertIn("broader reusable tool access", prompt)
        self.assertIn(
            "Every Bash tool call must copy one authorized command line "
            "byte-for-byte.",
            prompt,
        )
        self.assertIn("last report marker", prompt)
        self.assertIn("Task-local controller instruction", prompt)
        self.assertIn(
            "Correct the controller-identified whitespace defect only.",
            prompt,
        )

    def test_one_shot_prompt_has_no_ticket_or_resume_claim(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        prompt = agy_adapter.render_prompt(
            profile,
            "adhoc-1",
            "oracle",
            agy_adapter.frozen_task_state(profile, "adhoc-1"),
        )
        self.assertIn("One-shot run id: adhoc-1", prompt)
        self.assertIn("This session will not be resumed", prompt)
        self.assertNotIn("Ticket: #", prompt)
        self.assertNotIn("live ticket snapshot", prompt)

    def test_snapshot_identity_accepts_legacy_ticket_snapshot(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "12")
        del profile["task_contract"]["session_policy"]
        agy_adapter.assert_snapshot_identity(
            profile,
            "12",
            {"issue": "12"},
        )

    def test_profile_rejects_controller_state_inside_repository(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "5")
        profile["state_dir"] = str(self.repo_a / ".agy-state")
        profile_path = self.root / "profile.json"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.load_profile(str(profile_path))
        self.assertIn("state_dir must be outside", str(caught.exception))

    def test_profile_rejects_controller_state_outside_tmp(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "9")
        profile["state_dir"] = str(
            Path.home() / ".execution-state-forbidden"
        )
        profile_path = self.root / "profile-outside-tmp.json"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit) as caught:
            agy_adapter.load_profile(str(profile_path))
        self.assertIn(
            "state_dir must be under /tmp/execution/agy",
            str(caught.exception),
        )

    def test_profile_rejects_controller_state_under_tmp_but_outside_dispatch_root(
        self,
    ) -> None:
        profile = self.profile(self.repo_a, "project-a", "tmp-other")
        profile["state_dir"] = "/tmp/other/execution-controller-state"
        profile_path = self.root / "profile-tmp-other.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "state_dir must be under /tmp/execution/agy",
        ):
            agy_adapter.load_profile(str(profile_path))

    def test_profile_rejects_state_without_project_and_task_namespaces(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "shallow-state")
        profile["state_dir"] = "/tmp/execution/agy/only-one"
        profile_path = self.root / "profile-shallow-state.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "agy_project_id.*task-key|two components|namespace",
        ):
            agy_adapter.load_profile(str(profile_path))

    def test_profile_rejects_state_bound_to_another_project(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "wrong-project-state")
        profile["state_dir"] = str(
            agy_adapter.TEMP_ROOT / "another-project" / "wrong-project-state"
        )
        profile_path = self.root / "profile-wrong-state-project.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(SystemExit, "must equal.*agy_project_id"):
            agy_adapter.load_profile(str(profile_path))

    def test_profile_rejects_state_bound_to_another_task(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "right-task")
        profile["state_dir"] = str(
            agy_adapter.TEMP_ROOT
            / profile["agy_project_id"]
            / "another-task"
        )
        profile_path = self.root / "profile-wrong-state-task.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(SystemExit, "must equal.*task-key"):
            agy_adapter.load_profile(str(profile_path))

    def test_verify_can_load_after_protected_artifact_was_modified(self) -> None:
        protected = self.repo_a / "contract.md"
        protected.write_text("before\n")
        profile = self.profile(self.repo_a, "project-a", "6")
        profile["protected_artifacts"] = [
            {
                "path": str(protected),
                "sha256": agy_adapter.sha256(protected),
            }
        ]
        profile_path = self.root / "profile.json"
        profile_path.write_text(json.dumps(profile))
        protected.write_text("after\n")

        with self.assertRaises(SystemExit):
            agy_adapter.load_profile(str(profile_path))
        loaded = agy_adapter.load_profile(
            str(profile_path),
            validate_design=False,
        )
        self.assertEqual(loaded["protected_artifacts"][0]["path"], str(protected))

    def test_profile_rejects_cli_settings_as_a_raw_protected_artifact(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "settings-protected")
        profile["protected_artifacts"] = [
            {
                "path": str(self.settings),
                "sha256": agy_adapter.sha256(self.settings),
            }
        ]
        profile_path = self.root / "profile-settings-protected.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "protected_artifacts must not include the CLI settings file",
        ):
            agy_adapter.load_profile(str(profile_path))


if __name__ == "__main__":
    unittest.main()
