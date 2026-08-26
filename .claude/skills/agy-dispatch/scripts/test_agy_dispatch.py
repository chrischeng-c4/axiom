#!/usr/bin/env python3
from __future__ import annotations

import importlib.util
import json
import os
import shutil
import sqlite3
import subprocess
import tempfile
import unittest
from contextlib import ExitStack
from pathlib import Path
from unittest.mock import patch


SCRIPT = Path(__file__).with_name("agy_dispatch.py")
SPEC = importlib.util.spec_from_file_location("agy_dispatch", SCRIPT)
assert SPEC and SPEC.loader
agy_dispatch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(agy_dispatch)


class DispatchControllerTest(unittest.TestCase):
    def setUp(self) -> None:
        state_parent = Path("/tmp/agy-dispatch")
        state_parent.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=state_parent)
        self.root = Path(self.temporary.name)
        self.state_parent = state_parent
        self.state_project_ids: set[str] = set()
        self.conversation_dir = self.root / "conversations"
        self.standing_consent = self.root / "standing-consent.json"
        self.settings = self.root / "settings.json"
        self.project_a = self.root / "project-a"
        self.project_b = self.root / "project-b"
        self.project_a.mkdir()
        self.project_b.mkdir()
        self.conversation_dir.mkdir()
        self.settings.write_text(
            json.dumps({"permissions": agy_dispatch.canonical_global_policy()})
        )
        self.project_surface = {
            "allow": [],
            "deny": [],
            "ask": [],
        }
        agy_dispatch.SETTINGS = self.settings
        agy_dispatch.CONVERSATION_DIR = self.conversation_dir
        agy_dispatch.STANDING_CONSENT = self.standing_consent
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
            ["git", "config", "user.email", "agy-dispatch@test.invalid"],
            cwd=root,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "AGY Dispatch Test"],
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
        return {
            "root": str(root),
            "agy_project_root": str(project_root),
            "repo": "owner/repo",
            "agy_project_id": bound_project_id,
            "model": "gemini-3.7-flash-high",
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
            "global_permissions": agy_dispatch.canonical_global_policy(),
            "project_permissions": {
                kind: list(self.project_surface[kind])
                for kind in ("allow", "deny", "ask")
            },
            "project_settings": {
                "outside_of_folder_file_access": "always_deny"
            },
            "project_policy_observation": {
                "source": "official_project_ui_or_permissions",
                "observed_at": "2026-08-05T00:00:00Z",
                "project_id": bound_project_id,
                "matching_project_ids": [bound_project_id],
                "project_root": str(project_root.resolve()),
                "permissions": {
                    kind: list(self.project_surface[kind])
                    for kind in ("allow", "deny", "ask")
                },
                "outside_of_folder_file_access": "always_deny",
            },
            "task_commands": {
                "allow": ["pwd", "rg -n TODO src"],
                "deny": ["git push origin main"],
            },
            "protected_artifacts": [],
            "snapshot_paths": ["src"],
            "allowed_repo_writes": [],
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
        self.assertEqual(agy_dispatch.task_session_policy(profile), "ticketed")
        agy_dispatch.validate_task_key(profile, "10")

    def test_task_local_uv_environment_is_a_rebuild_cache_but_other_drift_fails_closed(self) -> None:
        (self.repo_a / ".venv").mkdir()
        (self.repo_a / ".venv" / "pyvenv.cfg").write_text("home = test\n")

        self.assertEqual(agy_dispatch.git_ignored_paths(self.repo_a), [".venv/"])
        agy_dispatch.assert_ignored_paths_unchanged(self.repo_a, {"ignored_paths": []})

        (self.repo_a / ".gitignore").write_text(".venv/\n.drift/\n")
        (self.repo_a / ".drift").mkdir()
        with self.assertRaisesRegex(SystemExit, "ignored repository path drift"):
            agy_dispatch.assert_ignored_paths_unchanged(
                self.repo_a,
                {"ignored_paths": []},
            )

    def test_ignored_path_baseline_is_required(self) -> None:
        with self.assertRaisesRegex(SystemExit, "ignored-path baseline"):
            agy_dispatch.assert_ignored_paths_unchanged(self.repo_a, {})

    def test_profile_accepts_one_shot_without_issue(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "one-shot.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(str(profile_path))
        self.assertEqual(
            loaded["task_contract"]["session_policy"],
            "one-shot",
        )
        self.assertNotIn("issue", loaded["task_contract"])
        agy_dispatch.validate_task_key(loaded, "adhoc-1")

    def test_profile_rejects_wrong_execution_contract_settings(self) -> None:
        cases = (
            ("model", "gemini-3.6-flash-high", "model must be gemini-3.7-flash-high"),
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
                    agy_dispatch.load_profile(str(profile_path))

    def test_snapshot_contract_rejects_model_and_root_drift(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        snapshot_contract = agy_dispatch.dispatch_contract(profile)

        profile["model"] = "gemini-3.7-flash-medium"
        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(profile, snapshot_contract)
        )

        profile["model"] = "gemini-3.7-flash-high"
        profile["root"] = str(self.repo_a.parent / "different-task-root")
        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(profile, snapshot_contract)
        )

        profile["root"] = str(self.repo_a)
        profile["repo"] = "other-owner/other-repo"
        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(profile, snapshot_contract)
        )

    def test_oracle_digest_is_required_and_frozen(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "oracle-frozen")
        oracle = Path(profile["state_dir"]) / "oracles" / "oracle-frozen.md"
        oracle.parent.mkdir(parents=True)
        oracle.write_text("expected witness\n")
        snapshot = {"oracle_sha256": agy_dispatch.sha256(oracle)}

        self.assertEqual(
            agy_dispatch.assert_oracle_unchanged(
                profile,
                "oracle-frozen",
                snapshot,
            ),
            oracle,
        )
        oracle.write_text("changed witness\n")
        with self.assertRaisesRegex(SystemExit, "oracle changed"):
            agy_dispatch.assert_oracle_unchanged(
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
        payload["snapshot_id"] = agy_dispatch.json_digest(payload)
        encoded = json.dumps(payload)
        current = snapshot_dir / "snapshot-id.json"
        immutable = history_dir / f"{payload['snapshot_id']}.json"
        current.write_text(encoded)
        immutable.write_text(encoded)

        self.assertEqual(
            agy_dispatch.load_snapshot(profile, "snapshot-id")["value"],
            "frozen",
        )
        current.write_text(json.dumps({**payload, "value": "rebound"}))
        with self.assertRaisesRegex(SystemExit, "snapshot id/digest mismatch"):
            agy_dispatch.load_snapshot(profile, "snapshot-id")

    def test_run_agent_rejects_contract_drift_before_launch(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "prelaunch-drift")
        snapshot = {
            "task_key": "prelaunch-drift",
            "session_policy": "one-shot",
            "dispatch_contract": agy_dispatch.dispatch_contract(profile),
            "agy_project_id": profile["agy_project_id"],
            "agy_project_root": profile["agy_project_root"],
            "worktree_scope": agy_dispatch.worktree_scope_report(profile),
        }
        profile["model"] = "gemini-3.7-flash-medium"

        with (
            patch.object(agy_dispatch, "require_project_ready", return_value={}),
            patch.object(
                agy_dispatch,
                "frozen_task_state",
                return_value={"run_id": "prelaunch-drift"},
            ),
            patch.object(agy_dispatch, "load_snapshot", return_value=snapshot),
            patch.object(agy_dispatch.subprocess, "run") as launch,
            self.assertRaisesRegex(SystemExit, "dispatch contract changed"),
        ):
            agy_dispatch.run_agent(profile, "prelaunch-drift", resume=False)
        launch.assert_not_called()

    def test_sandbox_is_opt_in_for_legacy_profiles_and_contract_bound(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "sandbox.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(str(profile_path))

        self.assertFalse(loaded["sandbox"])
        self.assertNotIn("--sandbox", agy_dispatch.agy_command(loaded, None))
        before = agy_dispatch.dispatch_contract(loaded)

        loaded["sandbox"] = True
        self.assertIn("--sandbox", agy_dispatch.agy_command(loaded, None))
        self.assertNotEqual(before, agy_dispatch.dispatch_contract(loaded))

        profile["sandbox"] = "yes"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(profile_path))
        self.assertIn("sandbox must be boolean", str(caught.exception))

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
            agy_dispatch.assert_no_sandbox_file_access_denial(profile, "10")

    def test_profile_requires_explicit_informed_external_payload_consent(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile_path = self.root / "missing-consent.json"
        profile.pop("external_payload_consent")
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_dispatch.load_profile(str(profile_path))

        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"]["approval_source"] = "inferred"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_dispatch.load_profile(str(profile_path))

        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"]["approved_payload_classes"].remove(
            "repository_read_context"
        )
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit):
            agy_dispatch.load_profile(str(profile_path))

    def test_standing_consent_covers_new_project_profiles_without_repeat_prompt(self) -> None:
        stored = self.write_standing_consent()
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"] = {
            "mode": "standing",
            "consent_id": "all-bounded-work-items-v1",
        }
        profile_path = self.root / "standing-consent-profile.json"
        profile_path.write_text(json.dumps(profile))

        loaded = agy_dispatch.load_profile(str(profile_path))

        self.assertEqual(loaded["external_payload_consent"]["mode"], "standing")
        self.assertEqual(
            loaded["external_payload_consent"]["consent_id"],
            "all-bounded-work-items-v1",
        )
        self.assertEqual(
            loaded["external_payload_consent"]["registry_digest"],
            agy_dispatch.json_digest(stored),
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

        loaded = agy_dispatch.load_profile(str(profile_path))

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
            agy_dispatch.load_profile(str(profile_path))

        self.write_standing_consent(
            payload_classes=["task_contract", "oracle"],
        )
        with self.assertRaisesRegex(SystemExit, "repository_read_context"):
            agy_dispatch.load_profile(str(profile_path))

    def test_standing_consent_change_voids_snapshot_contract_identity(self) -> None:
        self.write_standing_consent(approval_record="first approval")
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["external_payload_consent"] = {
            "mode": "standing",
            "consent_id": "all-bounded-work-items-v1",
        }
        profile_path = self.root / "standing-consent-identity-profile.json"
        profile_path.write_text(json.dumps(profile))
        first = agy_dispatch.load_profile(str(profile_path))
        snapshot_contract = agy_dispatch.dispatch_contract(first)

        self.write_standing_consent(approval_record="changed approval")
        changed = agy_dispatch.load_profile(str(profile_path))

        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(changed, snapshot_contract)
        )

    def test_one_shot_rejects_issue_and_unsafe_run_id(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["task_contract"]["issue"] = "10"
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.validate_task_identity(profile)
        self.assertIn("must not set task_contract.issue", str(caught.exception))

        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "../escape",
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.validate_task_identity(profile)
        self.assertIn("task identity must match", str(caught.exception))

    def test_one_shot_has_frozen_local_state_without_tracker_lookup(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a")
        self.assertEqual(
            agy_dispatch.frozen_task_state(profile, "adhoc-1"),
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
            agy_dispatch.validate_conversation_action(
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
            agy_dispatch.validate_conversation_action(
                ticketed,
                "11",
                resume=False,
            )
        self.assertIn("use resume for ticket #11", str(caught.exception))

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.validate_conversation_action(
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
            agy_dispatch.validate_conversation_action(
                profile,
                "missing-lineage",
                resume=False,
            )

    def test_project_policy_is_ready_without_mutating_project(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        report = agy_dispatch.project_policy_report(profile)
        self.assertTrue(report["dispatch_ready"])
        self.assertEqual(report["project_permissions_status"], "ready")
        self.assertEqual(report["global_permissions_status"], "ready")
        self.assertEqual(
            report["project_policy_observability"],
            "manual_official_ui_observation",
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
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["provisioning_status"],
            "PROJECT_SETUP_REQUIRED: Project policy requires formal UI observation or provisioning",
        )
        self.assertEqual(
            report["missing_project_rules"]["allow"],
            ["command(cargo test)"],
        )
        with self.assertRaisesRegex(
            SystemExit,
            "PROJECT_SETUP_REQUIRED: Project policy requires formal UI observation or provisioning",
        ):
            agy_dispatch.require_project_ready(profile)

    def test_project_file_access_policy_must_deny_outside_workspace(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_policy_observation"]["outside_of_folder_file_access"] = "always_allow"
        with self.assertRaisesRegex(SystemExit, "must be always_deny"):
            path = self.root / "invalid-project-observation.json"
            path.write_text(json.dumps(profile))
            agy_dispatch.load_profile(str(path))

        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_settings"]["outside_of_folder_file_access"] = "always_allow"
        with self.assertRaisesRegex(SystemExit, "must be always_deny"):
            path = self.root / "invalid-project-settings.json"
            path.write_text(json.dumps(profile))
            agy_dispatch.load_profile(str(path))

    def test_missing_official_project_observation_fails_closed_with_manual_steps(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile.pop("project_policy_observation")
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(report["project_policy_observability"], "PROJECT_SETUP_REQUIRED")
        self.assertIn(
            "Project policy has not been observed through the official Project Settings UI or /permissions Project scope",
            report["blockers"],
        )
        self.assertTrue(report["manual_setup"])

    def test_multiple_manual_project_matches_fail_closed_without_selecting_or_deleting(self) -> None:
        profile = self.profile(self.repo_a, "app_lumen", "1")
        profile["project_policy_observation"]["matching_project_ids"] = [
            "app_lumen",
            "stale-app_lumen",
        ]
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(report["project_discovery_status"], "PROJECT_SETUP_REQUIRED")
        self.assertIn("multiple matching persistent-root Projects", " ".join(report["blockers"]))
        self.assertIn("do not delete", " ".join(report["manual_setup"]).lower())

    def test_formal_cli_gap_returns_project_setup_without_registry_or_ui_automation(self) -> None:
        profile = self.profile(self.repo_a, "app_lumen", "1")
        profile.pop("project_policy_observation")
        completed = subprocess.CompletedProcess(
            ["agy", "--help"],
            0,
            stdout="Usage of agy:\n  --project\n  --new-project\nAvailable subcommands:\n  models\n",
            stderr="",
        )
        with patch.object(agy_dispatch.subprocess, "run", return_value=completed):
            report = agy_dispatch.project_policy_report(profile)
        capabilities = report["formal_project_capabilities"]
        self.assertFalse(capabilities["project_enumeration_cli"])
        self.assertFalse(capabilities["machine_readable_project_policy_cli"])
        self.assertEqual(report["provisioning_status"].split(":", 1)[0], "PROJECT_SETUP_REQUIRED")
        source = SCRIPT.read_text()
        for forbidden in ("PROJECT_DIR", "cache/projects", "AppleScript", "osascript", "Computer Use"):
            self.assertNotIn(forbidden, source)

    def test_global_permissions_drift_blocks_preflight(self) -> None:
        self.settings.write_text(
            json.dumps(
                {
                    "permissions": {
                        "allow": ["command(cargo test)"],
                        "deny": [],
                        "ask": [],
                    }
                }
            )
        )
        report = agy_dispatch.project_policy_report(
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
        policy = agy_dispatch.canonical_global_policy()
        self.assertEqual(policy, agy_dispatch.CANONICAL_GLOBAL_POLICY)
        self.assertNotIn("command(git)", policy["deny"])
        empty = agy_dispatch.normalize_permission_surface({})
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
                agy_dispatch.permission_decision(policy, empty, command)[0],
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
                agy_dispatch.permission_decision(policy, empty, command)[0],
                "deny",
            )

    def test_profile_templates_materialize_global_baseline_and_empty_project_exceptions(self) -> None:
        references = SCRIPT.parent.parent / "references"
        for name in ("profile-template.json", "one-shot-profile-template.json"):
            template = json.loads((references / name).read_text())
            self.assertEqual(template["model"], "gemini-3.7-flash-high")
            self.assertEqual(template["worktree_layout"], "in-project")
            self.assertEqual(template["launch_cwd"], "task-worktree")
            self.assertEqual(
                {
                    kind: template["global_permissions"][kind]
                    for kind in ("allow", "deny", "ask")
                },
                agy_dispatch.canonical_global_policy(),
            )
            self.assertEqual(template["project_permissions"], {"allow": [], "deny": [], "ask": []})
            self.assertEqual(
                template["external_payload_consent"],
                {
                    "mode": "standing",
                    "consent_id": "all-bounded-work-items-v1",
                },
            )

    def test_command_permission_matching_and_precedence(self) -> None:
        project = agy_dispatch.normalize_permission_surface(
            {
                "allow": ["command(git)", "command(rg)"],
                "deny": ["command(git push)"],
                "ask": [],
            }
        )
        global_rules = agy_dispatch.normalize_permission_surface({"allow": ["command(rg)"], "deny": [], "ask": []})
        self.assertEqual(
            agy_dispatch.permission_decision(global_rules, project, "rg -n TODO src"),
            ("allow", "command(rg)", "project"),
        )
        self.assertEqual(
            agy_dispatch.permission_decision(
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
        report = agy_dispatch.project_policy_report(profile)
        check = next(item for item in report["task_command_checks"] if item["command"] == "git log --oneline")
        self.assertEqual((check["decision"], check["source"], check["matched_rule"]), ("deny", "project", "command(git log)"))

        profile["task_commands"]["allow"].remove("git log --oneline")
        profile["task_commands"]["deny"].append("git log --oneline")
        report = agy_dispatch.project_policy_report(profile)
        check = next(item for item in report["task_command_checks"] if item["command"] == "git log --oneline")
        self.assertEqual((check["decision"], check["source"]), ("deny", "task_contract"))

    def test_permission_digest_detects_midrun_global_drift(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        snapshot = {
            "permission_state_digest": agy_dispatch.permission_state_digest(
                profile
            )
        }
        settings = json.loads(self.settings.read_text())
        settings["permissions"]["allow"].append("command(cargo test)")
        self.settings.write_text(json.dumps(settings))
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.assert_permission_state_unchanged(profile, snapshot)
        self.assertIn("permission state changed", str(caught.exception))

    def test_project_observation_must_match_persistent_root(self) -> None:
        profile = self.profile(self.repo_b, "project-a", "3")
        profile["project_policy_observation"]["project_root"] = str(self.repo_a)
        path = self.root / "wrong-project-root.json"
        path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(str(path))
        report = agy_dispatch.project_policy_report(loaded)
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

        loaded = agy_dispatch.load_profile(str(path))
        report = agy_dispatch.project_policy_report(loaded)

        self.assertTrue(report["dispatch_ready"])
        self.assertEqual(report["worktree_scope"]["mode"], "in-project")
        self.assertEqual(
            agy_dispatch.agy_command(loaded, None),
            ["agy", "--project", loaded["agy_project_id"]],
        )
        self.assertEqual(
            agy_dispatch.dispatch_contract(loaded)["agy_project_root"],
            loaded["agy_project_root"],
        )
        command = agy_dispatch.agy_command(loaded, None)
        self.assertNotIn("--add-dir", command)
        self.assertNotIn("--new-project", command)

    def test_worktree_readiness_rejects_project_root_as_task_root(self) -> None:
        profile = self.profile(
            self.project_a,
            "project-a",
            "root-is-scope",
            project_root=self.project_a,
        )
        report = agy_dispatch.worktree_scope_report(profile)

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
        report = agy_dispatch.worktree_scope_report(profile)

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
        report = agy_dispatch.worktree_scope_report(profile)

        self.assertFalse(report["dispatch_ready"])
        self.assertIn("root must be an exact Git worktree root", report["blockers"])

    def test_worktree_readiness_requires_git_worktree_registration(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "registered")

        with patch.object(
            agy_dispatch,
            "registered_worktree_paths",
            return_value=[self.project_a.resolve()],
        ):
            report = agy_dispatch.worktree_scope_report(profile)

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
        report = agy_dispatch.worktree_scope_report(profile)

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
            agy_dispatch.load_profile(str(path))

        profile["root"] = str(scope)
        profile["state_dir"] = str(scope / "controller-state")
        path.write_text(json.dumps(profile))
        with self.assertRaisesRegex(SystemExit, "state_dir must be outside"):
            agy_dispatch.load_profile(str(path))

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
        baseline = {"project_scope_baseline": agy_dispatch.project_scope_baseline(profile)}

        agy_dispatch.assert_project_scope_unchanged(profile, baseline)
        (scope / ".gitignore").write_text("changed\n")
        with self.assertRaisesRegex(SystemExit, "persistent AGY Project worktree changed"):
            agy_dispatch.assert_project_scope_unchanged(profile, baseline)

    def test_sibling_worktree_baseline_detects_hidden_nested_write(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling")
        profile = self.profile(self.repo_a, "project-a", "sibling-baseline")
        snapshot = {
            "sibling_worktree_baselines": agy_dispatch.sibling_worktree_baselines(
                profile
            )
        }

        agy_dispatch.assert_sibling_worktrees_unchanged(profile, snapshot)
        (sibling / ".gitignore").write_text("changed by escaped worker\n")

        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_dispatch.assert_sibling_worktrees_unchanged(profile, snapshot)

    def test_project_scope_baseline_detects_git_config_write(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "project-git-admin")
        snapshot = {
            "project_scope_baseline": agy_dispatch.project_scope_baseline(profile)
        }

        subprocess.run(
            ["git", "config", "agy.probe", "mutated"],
            cwd=self.project_a,
            check=True,
        )
        with self.assertRaisesRegex(SystemExit, "persistent AGY Project"):
            agy_dispatch.assert_project_scope_unchanged(profile, snapshot)

    def test_project_scope_baseline_detects_shallow_control_write(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "project-shallow-admin")
        snapshot = {
            "project_scope_baseline": agy_dispatch.project_scope_baseline(profile)
        }
        common = agy_dispatch.git_common_dir(self.project_a)
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
            agy_dispatch.assert_project_scope_unchanged(profile, snapshot)

    def test_sibling_baseline_detects_git_pointer_write(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling-pointer")
        profile = self.profile(self.repo_a, "project-a", "sibling-pointer")
        snapshot = {
            "sibling_worktree_baselines": agy_dispatch.sibling_worktree_baselines(
                profile
            )
        }

        pointer = sibling / ".git"
        pointer.write_text(pointer.read_text() + "\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_dispatch.assert_sibling_worktrees_unchanged(profile, snapshot)

    def test_registered_worktree_index_digest_binds_skip_worktree_flag(self) -> None:
        before = agy_dispatch.registered_worktree_index_digests(self.repo_a)
        subprocess.run(
            ["git", "update-index", "--skip-worktree", ".gitignore"],
            cwd=self.repo_a,
            check=True,
        )
        after = agy_dispatch.registered_worktree_index_digests(self.repo_a)

        self.assertNotEqual(before, after)

    def test_sibling_worktree_baseline_hashes_ignored_bytes_and_caches(self) -> None:
        sibling = self.add_nested_worktree(self.project_a, "sibling-ignored")
        gitignore = sibling / ".gitignore"
        gitignore.write_text(gitignore.read_text() + ".secret\n")
        (sibling / ".secret").write_text("private baseline\n")
        (sibling / ".venv").mkdir()
        (sibling / ".venv" / "marker").write_text("cache baseline\n")
        profile = self.profile(self.repo_a, "project-a", "sibling-ignored")
        snapshot = {
            "sibling_worktree_baselines": agy_dispatch.sibling_worktree_baselines(
                profile
            )
        }

        (sibling / ".secret").write_text("escaped secret write\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_dispatch.assert_sibling_worktrees_unchanged(profile, snapshot)

        (sibling / ".secret").write_text("private baseline\n")
        (sibling / ".venv" / "marker").write_text("escaped cache write\n")
        with self.assertRaisesRegex(SystemExit, "sibling worktree"):
            agy_dispatch.assert_sibling_worktrees_unchanged(profile, snapshot)

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
            "task_worktree_baseline": agy_dispatch.repository_worktree_baseline(
                self.repo_a
            )
        }

        cache.write_text("rebuild cache drift\n")
        agy_dispatch.assert_task_ignored_noncache_unchanged(profile, snapshot)
        secret.write_text("escaped secret write\n")
        with self.assertRaisesRegex(SystemExit, "ignored non-cache"):
            agy_dispatch.assert_task_ignored_noncache_unchanged(profile, snapshot)

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

        loaded = agy_dispatch.load_profile(str(path))
        before = agy_dispatch.dispatch_contract(loaded)
        self.assertEqual(
            before["inject_prompt_file_sha256"], agy_dispatch.sha256(prompt)
        )

        prompt.write_text("changed after snapshot\n")
        reloaded = agy_dispatch.load_profile(str(path))
        self.assertNotEqual(before, agy_dispatch.dispatch_contract(reloaded))

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

        loaded = agy_dispatch.load_profile(str(path))
        snapshot_contract = agy_dispatch.dispatch_contract(loaded)
        self.assertTrue(
            agy_dispatch.snapshot_contract_matches(loaded, snapshot_contract)
        )

        prompt.write_text("different frozen instruction\n")
        reloaded = agy_dispatch.load_profile(str(path))
        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(reloaded, snapshot_contract)
        )
        self.assertTrue(
            agy_dispatch.snapshot_contract_matches(
                reloaded,
                snapshot_contract,
                allow_prompt_hash_change=True,
            )
        )

        reloaded["task_contract"]["intent"] = "changed authority"
        self.assertFalse(
            agy_dispatch.snapshot_contract_matches(reloaded, snapshot_contract)
        )

    def test_extracts_last_line_anchored_exec_report_after_chatter(self) -> None:
        raw = (
            "progress chatter\n"
            "## EXEC REPORT\nDRAFT\n"
            "more work\n"
            "## EXEC REPORT\nPASS\n"
        )
        self.assertEqual(
            agy_dispatch.extract_exec_report(raw),
            "## EXEC REPORT\nPASS\n",
        )
        self.assertIsNone(
            agy_dispatch.extract_exec_report(
                "prose mentioning ## EXEC REPORT but no heading"
            )
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
            agy_dispatch.conversation_id_from_log(log)

    def test_run_evidence_binds_prompt_log_report_contract_and_conversation(self) -> None:
        profile = self.one_shot_profile(self.repo_a, "project-a", "evidence")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        conversation_id = "00000000-0000-0000-0000-000000000099"
        (runs / "evidence.conversation").write_text(conversation_id + "\n")
        prompt = runs / "evidence.prompt.md"
        contract = runs / "evidence.contract.json"
        agy_log = runs / "evidence.agy.log"
        raw = runs / "evidence.log"
        normalized = runs / "evidence.report.md"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_dispatch.dispatch_contract(profile)))
        agy_log.write_text(f"conversation={conversation_id}\n")
        raw.write_text("chatter\n## EXEC REPORT\nPASS\n")
        normalized.write_text("## EXEC REPORT\nPASS\n")

        agy_dispatch.write_run_evidence(
            profile=profile,
            task_key="evidence",
            suffix="",
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            raw_report_path=raw,
            normalized_report_path=normalized,
            snapshot_id="snapshot-evidence",
            exit_code=0,
            delivery_status="reported",
        )
        snapshot = {"snapshot_id": "snapshot-evidence"}
        evidence = agy_dispatch.assert_run_evidence(profile, "evidence", snapshot)
        self.assertEqual(evidence["conversation_id"], conversation_id)

        with self.assertRaisesRegex(SystemExit, "different snapshot"):
            agy_dispatch.assert_run_evidence(
                profile,
                "evidence",
                {"snapshot_id": "newer-snapshot"},
            )

        normalized.write_text("## EXEC REPORT\nTAMPERED\n")
        with self.assertRaisesRegex(SystemExit, "digest mismatch"):
            agy_dispatch.assert_run_evidence(profile, "evidence", snapshot)

    def test_run_evidence_accepts_nonzero_attempt_and_rejects_reclassification(
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

        evidence = agy_dispatch.assert_run_evidence(profile, task_key, snapshot)
        self.assertEqual(evidence["exit_code"], 1)
        self.assertEqual(evidence["delivery_status"], "nonzero")
        self.assertNotIn("normalized_report", evidence["files"])

        reclassified = json.loads(evidence_path.read_text())
        reclassified["delivery_status"] = "empty"
        evidence_path.write_text(json.dumps(reclassified, indent=2) + "\n")
        with self.assertRaisesRegex(SystemExit, "classification mismatch"):
            agy_dispatch.assert_run_evidence(profile, task_key, snapshot)

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
            agy_dispatch.latest_round_contract(profile, task_key)[0],
            resume_contract.resolve(),
        )
        self.assertEqual(
            agy_dispatch.latest_run_evidence(profile, task_key)[0],
            resume_evidence.resolve(),
        )
        self.assertEqual(
            agy_dispatch.assert_complete_attempt_lineage(profile, task_key),
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
            agy_dispatch.assert_complete_attempt_lineage(profile, task_key)

        gap_suffix = ".resume.2"
        for ending in (
            ".prompt.md",
            ".contract.json",
            ".agy.log",
            ".log",
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
            agy_dispatch.assert_complete_attempt_lineage(profile, task_key)

    def test_resume_evidence_rejects_different_logged_conversation(self) -> None:
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
        evidence["files"]["agy_log"]["sha256"] = agy_dispatch.sha256(agy_log)
        evidence_path.write_text(json.dumps(evidence, indent=2) + "\n")

        with self.assertRaisesRegex(
            SystemExit,
            "does not bind the recorded conversation id",
        ):
            agy_dispatch.assert_run_evidence(
                profile,
                task_key,
                {"snapshot_id": "snapshot-resume"},
            )

    def test_verify_nonzero_attempt_writes_predecessor_marker(self) -> None:
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
            patch.object(agy_dispatch, "require_project_ready"),
            patch.object(agy_dispatch, "load_snapshot", return_value=snapshot),
            patch.object(agy_dispatch, "assert_snapshot_identity"),
            patch.object(agy_dispatch, "assert_injected_prompt_unchanged"),
            patch.object(agy_dispatch, "assert_executed_round_contract"),
            patch.object(agy_dispatch, "assert_worktree_scope_unchanged"),
            patch.object(agy_dispatch, "assert_oracle_unchanged"),
            patch.object(agy_dispatch, "assert_permission_state_unchanged"),
            patch.object(agy_dispatch, "assert_project_scope_unchanged"),
            patch.object(agy_dispatch, "assert_sibling_worktrees_unchanged"),
            patch.object(agy_dispatch, "assert_ignored_paths_unchanged"),
            patch.object(agy_dispatch, "assert_task_ignored_noncache_unchanged"),
            patch.object(agy_dispatch, "assert_task_git_admin_unchanged"),
            patch.object(agy_dispatch, "assert_git_common_objects_unchanged"),
            patch.object(
                agy_dispatch,
                "assert_registered_worktree_indexes_unchanged",
            ),
            patch("builtins.print") as output,
        ):
            agy_dispatch.verify(profile, task_key)

        rendered = "\n".join(
            str(call.args[0]) for call in output.call_args_list if call.args
        )
        self.assertIn("DELIVERY_FAILED_ISOLATION_VERIFIED", rendered)
        self.assertIn("delivery_status=nonzero", rendered)
        marker_path = agy_dispatch.verified_marker_path(profile, task_key)
        self.assertTrue(marker_path.is_file())
        marker = json.loads(marker_path.read_text())
        self.assertEqual(marker["delivery_status"], "nonzero")
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
            delivery_status="missing-conversation",
        )
        snapshot = self.failed_attempt_snapshot(
            profile,
            task_key,
            snapshot_id,
            None,
        )

        with (
            patch.object(agy_dispatch, "require_project_ready"),
            patch.object(agy_dispatch, "load_snapshot", return_value=snapshot),
            patch.object(agy_dispatch, "assert_snapshot_identity"),
            patch.object(agy_dispatch, "assert_injected_prompt_unchanged"),
            patch.object(agy_dispatch, "assert_executed_round_contract"),
            patch.object(agy_dispatch, "assert_worktree_scope_unchanged"),
            patch.object(agy_dispatch, "assert_oracle_unchanged"),
            patch.object(agy_dispatch, "assert_permission_state_unchanged"),
            patch.object(agy_dispatch, "assert_project_scope_unchanged"),
            patch.object(agy_dispatch, "assert_sibling_worktrees_unchanged"),
            patch.object(agy_dispatch, "assert_ignored_paths_unchanged"),
            patch.object(agy_dispatch, "assert_task_ignored_noncache_unchanged"),
            patch.object(agy_dispatch, "assert_task_git_admin_unchanged"),
            patch.object(agy_dispatch, "assert_git_common_objects_unchanged"),
            patch.object(
                agy_dispatch,
                "assert_registered_worktree_indexes_unchanged",
            ),
            self.assertRaisesRegex(
                SystemExit,
                "delivery evidence is invalid.*conversation lineage",
            ),
        ):
            agy_dispatch.verify(profile, task_key)

        self.assertFalse(
            agy_dispatch.verified_marker_path(profile, task_key).exists()
        )

    def test_extracts_denied_run_command_from_protobuf_payload(self) -> None:
        payload = (
            b"\x08\x15garbage"
            b'{"CommandLine":"rg -c NATIVE_FUNC_ADDRS\\\\.with apps/mamba",'
            b'"Cwd":"/repo","WaitMsBeforeAsync":5000}'
            b"\x00trailer"
        )
        self.assertEqual(
            agy_dispatch.extract_run_command_lines(payload),
            [r"rg -c NATIVE_FUNC_ADDRS\.with apps/mamba"],
        )

    def test_extracts_all_command_requests_with_whitespace_and_reordered_keys(self) -> None:
        payload = (
            b'prefix {"CommandLine":"pwd","Cwd":"/task"} middle '
            b'{ "Cwd": "/escaped", "CommandLine": "git status --short" } suffix'
        )

        self.assertEqual(
            agy_dispatch.extract_run_command_requests(payload),
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
        raw = runs / f"{task_key}{suffix}.log"
        normalized = runs / f"{task_key}{suffix}.report.md"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_dispatch.dispatch_contract(profile)))
        agy_log.write_text(f"conversation={conversation_id}\n")
        raw.write_text("chatter\n## EXEC REPORT\nPASS\n")
        normalized.write_text("## EXEC REPORT\nPASS\n")
        evidence_path = agy_dispatch.write_run_evidence(
            profile=profile,
            task_key=task_key,
            suffix=suffix,
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            raw_report_path=raw,
            normalized_report_path=normalized,
            snapshot_id=snapshot_id,
            exit_code=0,
            delivery_status="reported",
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
        delivery_status: str = "nonzero",
    ) -> tuple[Path, dict]:
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        prompt = runs / f"{task_key}.prompt.md"
        contract = runs / f"{task_key}.contract.json"
        agy_log = runs / f"{task_key}.agy.log"
        raw = runs / f"{task_key}.log"
        prompt.write_text("prompt\n")
        contract.write_text(json.dumps(agy_dispatch.dispatch_contract(profile)))
        agy_log.write_text(
            f"conversation={conversation_id}\n"
            if conversation_id is not None
            else "agy transport log without a conversation id\n"
        )
        raw.write_text("transport failed before a terminal report\n")
        evidence_path = agy_dispatch.write_run_evidence(
            profile=profile,
            task_key=task_key,
            suffix="",
            conversation_id=conversation_id,
            prompt_path=prompt,
            round_contract_path=contract,
            agy_log_path=agy_log,
            raw_report_path=raw,
            normalized_report_path=None,
            snapshot_id=snapshot_id,
            exit_code=exit_code,
            delivery_status=delivery_status,
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
            "manifest": agy_dispatch.manifest(root, profile["snapshot_paths"]),
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
            agy_dispatch.requested_run_commands(conversation_id)

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
            agy_dispatch.requested_run_commands(conversation_id)

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
            agy_dispatch.requested_run_commands(conversation_id)

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
            agy_dispatch.requested_run_commands(conversation_id)

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
            agy_dispatch.requested_run_commands(conversation_id)

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

        before = agy_dispatch.conversation_steps_digest(
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
        after = agy_dispatch.conversation_steps_digest(
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
            agy_dispatch.requested_run_commands(conversation_id)

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
            agy_dispatch.requested_run_commands(conversation_id),
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
            agy_dispatch.requested_run_commands(conversation_id)

    def test_requested_run_commands_allows_non_shell_type15_row(self) -> None:
        conversation_id = "conversation-non-shell-type15"
        self.write_raw_conversation(
            conversation_id,
            [(1, 15, 3, b'{"tool":"read_file","path":"src/lib.rs"}')],
        )

        self.assertEqual(
            agy_dispatch.requested_run_commands(conversation_id),
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
            agy_dispatch.requested_run_commands(conversation_id),
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
                for item in agy_dispatch.requested_run_commands(conversation_id)
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
            agy_dispatch.requested_run_commands(
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
                agy_dispatch,
                "require_project_ready",
                return_value=readiness,
            ),
            patch.object(agy_dispatch, "assert_verified_predecessor") as gate,
        ):
            first_snapshot_path = agy_dispatch.snapshot(profile, task_key)
        gate.assert_not_called()
        first_snapshot = agy_dispatch.load_snapshot(profile, task_key)

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
                agy_dispatch,
                "require_project_ready",
                return_value=readiness,
            ),
            self.assertRaisesRegex(
                SystemExit,
                "verify the prior|verified predecessor",
            ),
        ):
            agy_dispatch.snapshot(profile, task_key)

        evidence_path, evidence = self.write_successful_run_evidence(
            profile,
            task_key,
            conversation_id,
            first_snapshot["snapshot_id"],
        )
        marker_path = agy_dispatch.write_verified_marker(
            profile,
            task_key,
            first_snapshot,
            evidence_path,
            evidence,
        )
        self.assertEqual(
            marker_path,
            agy_dispatch.verified_marker_path(profile, task_key),
        )
        agy_dispatch.assert_verified_predecessor(
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
            agy_dispatch.assert_verified_predecessor(
                profile,
                task_key,
                conversation_id,
            )
        self.replace_raw_conversation_step_payload(
            conversation_id,
            1,
            command_payload,
        )
        agy_dispatch.assert_verified_predecessor(
            profile,
            task_key,
            conversation_id,
        )

        with patch.object(
            agy_dispatch,
            "require_project_ready",
            return_value=readiness,
        ):
            second_snapshot_path = agy_dispatch.snapshot(profile, task_key)
        self.assertTrue(second_snapshot_path.is_file())

        self.append_raw_conversation_step(
            conversation_id,
            (2, 15, 3, command_payload),
        )
        with (
            patch.object(
                agy_dispatch,
                "require_project_ready",
                return_value=readiness,
            ),
            self.assertRaisesRegex(
                SystemExit,
                "verified predecessor|new AGY steps|newer steps",
            ),
        ):
            agy_dispatch.snapshot(profile, task_key)

    def test_task_operation_lock_rejects_concurrent_resume_or_snapshot(self) -> None:
        profile = self.one_shot_profile(
            self.repo_a,
            "project-a",
            "operation-lock",
        )

        with agy_dispatch.task_operation_lock(
            profile,
            "operation-lock",
            "resume",
        ):
            with self.assertRaisesRegex(SystemExit, "another snapshot.*launch.*verify"):
                with agy_dispatch.task_operation_lock(
                    profile,
                    "operation-lock",
                    "snapshot",
                ):
                    self.fail("the second task operation lock must not be acquired")

    def test_project_concurrency_lock_rejects_second_bounded_write_in_same_project(
        self,
    ) -> None:
        first = self.one_shot_profile(self.repo_a, "project-a", "writer-1")
        first["mode"] = "bounded-write"
        second = self.one_shot_profile(self.repo_a, "project-a", "writer-2")
        second["mode"] = "bounded-write"

        with agy_dispatch.project_concurrency_lock(first, "writer-1", "dispatch"):
            with self.assertRaisesRegex(SystemExit, "refusing dispatch.*Project"):
                with agy_dispatch.project_concurrency_lock(
                    second, "writer-2", "dispatch"
                ):
                    self.fail("a second bounded-write task must not start")

    def test_project_concurrency_lock_rejects_measure_only_while_bounded_write_active(
        self,
    ) -> None:
        writer = self.one_shot_profile(self.repo_a, "project-a", "writer")
        writer["mode"] = "bounded-write"
        reader = self.one_shot_profile(self.repo_a, "project-a", "reader")
        reader["mode"] = "measure-only"

        with agy_dispatch.project_concurrency_lock(writer, "writer", "dispatch"):
            with self.assertRaisesRegex(SystemExit, "refusing dispatch.*Project"):
                with agy_dispatch.project_concurrency_lock(
                    reader, "reader", "dispatch"
                ):
                    self.fail(
                        "a measure-only task must not start against an active "
                        "bounded-write task"
                    )

    def test_project_concurrency_lock_rejects_bounded_write_while_measure_only_active(
        self,
    ) -> None:
        reader = self.one_shot_profile(self.repo_a, "project-a", "reader")
        reader["mode"] = "measure-only"
        writer = self.one_shot_profile(self.repo_a, "project-a", "writer")
        writer["mode"] = "bounded-write"

        with agy_dispatch.project_concurrency_lock(reader, "reader", "dispatch"):
            with self.assertRaisesRegex(SystemExit, "refusing dispatch.*Project"):
                with agy_dispatch.project_concurrency_lock(
                    writer, "writer", "dispatch"
                ):
                    self.fail(
                        "a bounded-write task must not start against an active "
                        "measure-only task"
                    )

    def test_project_concurrency_lock_allows_concurrent_measure_only_in_same_project(
        self,
    ) -> None:
        first = self.one_shot_profile(self.repo_a, "project-a", "reader-1")
        first["mode"] = "measure-only"
        second = self.one_shot_profile(self.repo_a, "project-a", "reader-2")
        second["mode"] = "measure-only"

        entered: list[str] = []
        with agy_dispatch.project_concurrency_lock(first, "reader-1", "dispatch"):
            entered.append("reader-1")
            with agy_dispatch.project_concurrency_lock(second, "reader-2", "dispatch"):
                entered.append("reader-2")

        self.assertEqual(entered, ["reader-1", "reader-2"])

    def test_project_concurrency_lock_does_not_block_across_different_projects(
        self,
    ) -> None:
        writer_a = self.one_shot_profile(self.repo_a, "project-a", "writer")
        writer_a["mode"] = "bounded-write"
        writer_b = self.one_shot_profile(self.repo_b, "project-b", "writer")
        writer_b["mode"] = "bounded-write"

        entered: list[str] = []
        with agy_dispatch.project_concurrency_lock(writer_a, "writer", "dispatch"):
            entered.append("project-a")
            with agy_dispatch.project_concurrency_lock(writer_b, "writer", "dispatch"):
                entered.append("project-b")

        self.assertEqual(entered, ["project-a", "project-b"])

    def test_agy_launch_cwd_is_exact_task_worktree(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "cwd")

        self.assertEqual(agy_dispatch.agy_launch_cwd(profile), self.repo_a.resolve())

        profile["launch_cwd"] = "project-root"
        with self.assertRaisesRegex(SystemExit, "launch_cwd must be task-worktree"):
            agy_dispatch.agy_launch_cwd(profile)

    def test_run_agent_launches_subprocess_from_exact_task_worktree(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "launch-cwd")
        state = Path(profile["state_dir"])
        oracle_dir = state / "oracles"
        oracle_dir.mkdir(parents=True)
        (oracle_dir / "launch-cwd.md").write_text("expected canary\n")

        def fake_run(command: list[str], **kwargs: object) -> subprocess.CompletedProcess:
            self.assertEqual(kwargs["cwd"], self.repo_a.resolve())
            self.assertIn("gemini-3.7-flash-high", command)
            self.assertNotIn("--add-dir", command)
            log_path = Path(command[command.index("--log-file") + 1])
            log_path.write_text("conversation=00000000-0000-0000-0000-000000000001\n")
            kwargs["stdout"].write("## EXEC REPORT\nPASS\n")
            return subprocess.CompletedProcess(command, 0)

        with ExitStack() as stack:
            stack.enter_context(
                patch.object(agy_dispatch, "require_project_ready", return_value={})
            )
            stack.enter_context(
                patch.object(
                    agy_dispatch,
                    "frozen_task_state",
                    return_value={"number": 1, "state": "OPEN"},
                )
            )
            stack.enter_context(
                patch.object(
                    agy_dispatch,
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
                "assert_git_common_objects_unchanged",
                "assert_registered_worktree_indexes_unchanged",
                "assert_task_state_unchanged",
                "assert_verified_predecessor",
            ):
                stack.enter_context(patch.object(agy_dispatch, name))
            stack.enter_context(
                patch.object(
                    agy_dispatch,
                    "assert_oracle_unchanged",
                    return_value=oracle_dir / "launch-cwd.md",
                )
            )
            stack.enter_context(
                patch.object(agy_dispatch, "audit_task_commands", return_value=[])
            )
            stack.enter_context(
                patch.object(
                    agy_dispatch,
                    "validate_conversation_action",
                    return_value="00000000-0000-0000-0000-000000000001",
                )
            )
            stack.enter_context(
                patch.object(
                    agy_dispatch,
                    "assert_complete_attempt_lineage",
                    return_value=[0],
                )
            )
            stack.enter_context(
                patch.object(agy_dispatch, "render_prompt", return_value="prompt")
            )
            stack.enter_context(
                patch.object(agy_dispatch.subprocess, "run", side_effect=fake_run)
            )
            agy_dispatch.run_agent(profile, "launch-cwd", resume=True)

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

        audited = agy_dispatch.audit_task_commands(
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
                    agy_dispatch.audit_task_commands(
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
        audited = agy_dispatch.audit_task_commands(
            profile,
            "7",
            {
                "conversation_id": "conversation-7",
                "conversation_step_floor": 1,
                "conversation_predecessor_digest": (
                    agy_dispatch.conversation_steps_digest(
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
            agy_dispatch.audit_task_commands(
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
        prompt = agy_dispatch.render_prompt(
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
        prompt = agy_dispatch.render_prompt(
            profile,
            "adhoc-1",
            "oracle",
            agy_dispatch.frozen_task_state(profile, "adhoc-1"),
        )
        self.assertIn("One-shot run id: adhoc-1", prompt)
        self.assertIn("This session will not be resumed", prompt)
        self.assertNotIn("Ticket: #", prompt)
        self.assertNotIn("live ticket snapshot", prompt)

    def test_snapshot_identity_accepts_legacy_ticket_snapshot(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "12")
        del profile["task_contract"]["session_policy"]
        agy_dispatch.assert_snapshot_identity(
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
            agy_dispatch.load_profile(str(profile_path))
        self.assertIn("state_dir must be outside", str(caught.exception))

    def test_profile_rejects_controller_state_outside_tmp(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "9")
        profile["state_dir"] = str(
            Path.home() / ".agy-dispatch-state-forbidden"
        )
        profile_path = self.root / "profile-outside-tmp.json"
        profile_path.write_text(json.dumps(profile))
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(profile_path))
        self.assertIn(
            "state_dir must be under /tmp/agy-dispatch",
            str(caught.exception),
        )

    def test_profile_rejects_controller_state_under_tmp_but_outside_dispatch_root(
        self,
    ) -> None:
        profile = self.profile(self.repo_a, "project-a", "tmp-other")
        profile["state_dir"] = "/tmp/other/agy-dispatch-controller-state"
        profile_path = self.root / "profile-tmp-other.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "state_dir must be under /tmp/agy-dispatch",
        ):
            agy_dispatch.load_profile(str(profile_path))

    def test_profile_rejects_state_without_project_and_task_namespaces(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "shallow-state")
        profile["state_dir"] = "/tmp/agy-dispatch/only-one"
        profile_path = self.root / "profile-shallow-state.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(
            SystemExit,
            "agy_project_id.*task-key|two components|namespace",
        ):
            agy_dispatch.load_profile(str(profile_path))

    def test_profile_rejects_state_bound_to_another_project(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "wrong-project-state")
        profile["state_dir"] = str(
            agy_dispatch.TEMP_ROOT / "another-project" / "wrong-project-state"
        )
        profile_path = self.root / "profile-wrong-state-project.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(SystemExit, "must equal.*agy_project_id"):
            agy_dispatch.load_profile(str(profile_path))

    def test_profile_rejects_state_bound_to_another_task(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "right-task")
        profile["state_dir"] = str(
            agy_dispatch.TEMP_ROOT
            / profile["agy_project_id"]
            / "another-task"
        )
        profile_path = self.root / "profile-wrong-state-task.json"
        profile_path.write_text(json.dumps(profile))

        with self.assertRaisesRegex(SystemExit, "must equal.*task-key"):
            agy_dispatch.load_profile(str(profile_path))

    def test_verify_can_load_after_protected_artifact_was_modified(self) -> None:
        protected = self.repo_a / "contract.md"
        protected.write_text("before\n")
        profile = self.profile(self.repo_a, "project-a", "6")
        profile["protected_artifacts"] = [
            {
                "path": str(protected),
                "sha256": agy_dispatch.sha256(protected),
            }
        ]
        profile_path = self.root / "profile.json"
        profile_path.write_text(json.dumps(profile))
        protected.write_text("after\n")

        with self.assertRaises(SystemExit):
            agy_dispatch.load_profile(str(profile_path))
        loaded = agy_dispatch.load_profile(
            str(profile_path),
            validate_design=False,
        )
        self.assertEqual(loaded["protected_artifacts"][0]["path"], str(protected))


if __name__ == "__main__":
    unittest.main()
