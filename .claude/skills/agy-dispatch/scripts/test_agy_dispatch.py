#!/usr/bin/env python3
from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import shutil
import sqlite3
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("agy_dispatch.py")
SPEC = importlib.util.spec_from_file_location("agy_dispatch", SCRIPT)
assert SPEC and SPEC.loader
agy_dispatch = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(agy_dispatch)


class DispatchControllerTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/tmp")
        self.root = Path(self.temporary.name)
        self.project_dir = self.root / "projects"
        self.conversation_dir = self.root / "conversations"
        self.settings = self.root / "settings.json"
        self.global_config = self.root / "config.json"
        self.repo_a = self.root / "repo-a"
        self.repo_b = self.root / "repo-b"
        self.repo_a.mkdir()
        self.repo_b.mkdir()
        self.project_dir.mkdir()
        self.conversation_dir.mkdir()
        self.settings.write_text(
            json.dumps({"permissions": {"allow": [], "deny": [], "ask": []}})
        )
        self.global_config.write_text(
            json.dumps(
                {
                    "userSettings": {
                        "globalPermissionGrants": {
                            "allow": [],
                            "deny": [],
                            "ask": [],
                        }
                    }
                }
            )
        )
        self.project_surface = {
            "allow": ["command(pwd)", "command(rg)"],
            "deny": ["command(git push)"],
            "ask": [],
        }
        self.write_project("project-a", self.repo_a, self.project_surface)
        self.write_project("project-b", self.repo_b, self.project_surface)
        agy_dispatch.SETTINGS = self.settings
        agy_dispatch.GLOBAL = self.global_config
        agy_dispatch.PROJECT_DIR = self.project_dir
        agy_dispatch.CONVERSATION_DIR = self.conversation_dir

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_project(
        self,
        project_id: str,
        root: Path,
        surface: dict[str, list[str]],
    ) -> None:
        (self.project_dir / f"{project_id}.json").write_text(
            json.dumps(
                {
                    "id": project_id,
                    "name": project_id,
                    "projectResources": {
                        "resources": [
                            {
                                "gitFolder": {
                                    "folderUri": root.resolve().as_uri()
                                }
                            }
                        ]
                    },
                    "permissionGrants": {
                        "permissionGrants": surface,
                    },
                    "settings": {},
                }
            )
        )

    def profile(self, root: Path, project_id: str, issue: str) -> dict:
        return {
            "root": str(root),
            "repo": "owner/repo",
            "agy_project_id": project_id,
            "state_dir": str(self.root / f"state-{issue}"),
            "mode": "measure-only",
            "task_contract": {
                "kind": "measurement",
                "session_policy": "ticketed",
                "issue": issue,
                "design_inputs": [],
            },
            "project_permissions": {
                **self.project_surface,
                "require_empty_global": True,
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
        profile["state_dir"] = str(self.root / f"state-{run_id}")
        profile["task_contract"] = {
            "kind": "measurement",
            "session_policy": "one-shot",
            "run_id": run_id,
            "intent": "Inspect one bounded condition and report evidence.",
            "design_inputs": [],
        }
        return profile

    def test_ticketed_policy_remains_the_default_for_legacy_profiles(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "10")
        del profile["task_contract"]["session_policy"]
        self.assertEqual(agy_dispatch.task_session_policy(profile), "ticketed")
        agy_dispatch.validate_task_key(profile, "10")

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

    def test_project_policy_is_ready_without_mutating_project(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        project_path = self.project_dir / "project-a.json"
        before = project_path.read_bytes()
        report = agy_dispatch.project_policy_report(profile)
        self.assertTrue(report["dispatch_ready"])
        self.assertEqual(report["project_permissions_status"], "ready")
        self.assertEqual(report["global_permissions_status"], "empty")
        self.assertEqual(project_path.read_bytes(), before)

    def test_project_permission_drift_blocks_dispatch(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_permissions"]["allow"].append("command(cargo test)")
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["missing_project_rules"]["allow"],
            ["command(cargo test)"],
        )

    def test_global_permissions_block_project_isolation(self) -> None:
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
        self.assertEqual(
            report["global_permissions_status"], "inherited-broadening"
        )
        self.assertEqual(
            report["global_broadening_rules"], ["command(cargo test)"]
        )

    def test_inherited_deny_only_globals_do_not_block(self) -> None:
        """Only an inherited `allow` can widen the worker past the declared
        surface. Blocking on deny/ask too made a harmless inherited rule look
        like a policy failure, which pushed the controller toward disabling the
        check instead of reading it."""
        self.settings.write_text(
            json.dumps(
                {
                    "permissions": {
                        "allow": [],
                        "deny": ["command(git push)"],
                        "ask": [],
                    }
                }
            )
        )
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_permissions"]["require_empty_global"] = False
        report = agy_dispatch.project_policy_report(profile)
        self.assertEqual(report["global_broadening_rules"], [])
        self.assertEqual(
            report["global_permissions_status"], "inherited-narrowing-only"
        )
        self.assertTrue(report["dispatch_ready"], report["blockers"])

    def test_inherited_allow_blocks_even_with_the_strict_flag_off(self) -> None:
        """The broadening check is the real invariant, so turning the opt-in
        strictness off must not reopen the hole it used to cover."""
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
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_permissions"]["require_empty_global"] = False
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertEqual(
            report["global_broadening_rules"], ["command(cargo test)"]
        )

    def test_command_permission_matching_and_precedence(self) -> None:
        project = agy_dispatch.normalize_permission_surface(
            {
                "allow": ["command(git)", "command(rg)"],
                "deny": ["command(git push)"],
                "ask": [],
            }
        )
        empty = agy_dispatch.normalize_permission_surface({})
        self.assertEqual(
            agy_dispatch.permission_decision(project, empty, "rg -n TODO src"),
            ("allow", "command(rg)"),
        )
        self.assertEqual(
            agy_dispatch.permission_decision(
                project,
                empty,
                "git push origin main",
            ),
            ("deny", "command(git push)"),
        )

    def test_agy_deadline_is_not_reported_as_a_denial(self) -> None:
        log_dir = Path(tempfile.mkdtemp())
        agy_log = log_dir / "run.agy.log"

        agy_log.write_text("startup chatter\n")
        # The local report is what AGY writes when its own deadline fires.
        self.assertTrue(
            agy_dispatch.timed_out("Error: timeout waiting for response\n", agy_log)
        )
        # A deadline that killed the process before it wrote a report is only
        # visible in the AGY log.
        agy_log.write_text(
            "chatter\nE0806 printmode.go:499] Print mode: timed out after 13412 polls\n"
        )
        self.assertTrue(agy_dispatch.timed_out("", agy_log))
        # A denial must keep routing the controller to `denied`.
        agy_log.write_text("chatter\npermission denied for command: cargo publish\n")
        self.assertFalse(
            agy_dispatch.timed_out("Error: command not permitted\n", agy_log)
        )
        # A missing log is not evidence of a deadline.
        self.assertFalse(agy_dispatch.timed_out("", log_dir / "absent.agy.log"))

    def test_reads_wal_conversation_store_after_agy_removed_sidecars(self) -> None:
        database = Path(tempfile.mkdtemp()) / "conversation.db"
        writer = sqlite3.connect(database)
        writer.execute("pragma journal_mode=wal")
        writer.execute("create table steps (idx integer, step_payload text)")
        writer.execute("insert into steps values (0, '{}')")
        writer.commit()
        writer.close()
        # AGY checkpoints and removes the sidecars on exit while leaving the
        # header in WAL format -- bytes 18 and 19 are the write/read file
        # versions, and 2 means WAL.
        for sidecar in ("-wal", "-shm"):
            side = database.with_name(database.name + sidecar)
            if side.exists():
                side.unlink()
        self.assertEqual(database.read_bytes()[18:20], b"\x02\x02")

        plain = sqlite3.connect(f"file:{database}?mode=ro", uri=True)
        with self.assertRaises(sqlite3.OperationalError):
            plain.execute("select count(*) from steps").fetchone()
        plain.close()

        connection = agy_dispatch.connect_conversation(database)
        try:
            self.assertEqual(
                connection.execute("select count(*) from steps").fetchone()[0], 1
            )
        finally:
            connection.close()

    def test_live_conversation_is_read_through_its_wal_not_around_it(self) -> None:
        database = Path(tempfile.mkdtemp()) / "conversation.db"
        writer = sqlite3.connect(database)
        # Schema in the main file, rows in the WAL: the difference the two
        # reads disagree about is then the steps, not the table's existence.
        writer.execute("create table steps (idx integer, step_payload text)")
        writer.commit()
        writer.execute("pragma journal_mode=wal")
        writer.execute("pragma wal_autocheckpoint=0")
        writer.execute("insert into steps values (0, '{}')")
        writer.execute("insert into steps values (1, '{}')")
        writer.commit()
        try:
            # A running AGY leaves committed steps in an uncheckpointed WAL.
            self.assertTrue(database.with_name(database.name + "-wal").exists())
            skipped = sqlite3.connect(
                f"file:{database}?mode=ro&immutable=1", uri=True
            )
            try:
                # `immutable=1` promises the file is not being written and so
                # reads around the WAL, returning a conversation missing the
                # worker's most recent commands.
                self.assertEqual(
                    skipped.execute("select count(*) from steps").fetchone()[0], 0
                )
            finally:
                skipped.close()

            connection = agy_dispatch.connect_conversation(database)
            try:
                self.assertEqual(
                    connection.execute("select count(*) from steps").fetchone()[0], 2
                )
            finally:
                connection.close()
        finally:
            writer.close()

    def test_unsandboxed_rule_without_command_twin_is_inert(self) -> None:
        surface = agy_dispatch.normalize_permission_surface(
            {
                "allow": [
                    "command(cargo build -p x --lib)",
                    "unsandboxed(cargo build -p x --lib)",
                    "unsandboxed(cargo test -p x --lib gate)",
                ],
                "deny": [],
                "ask": [],
            }
        )
        self.assertEqual(
            agy_dispatch.inert_unsandboxed_rules(surface),
            ["unsandboxed(cargo test -p x --lib gate)"],
        )
        # A sandbox escape is consulted only after the command already resolved
        # to allow, so the unpaired rule can never fire on its own.
        empty = agy_dispatch.normalize_permission_surface({})
        self.assertEqual(
            agy_dispatch.permission_decision(
                surface, empty, "cargo test -p x --lib gate"
            ),
            ("ask", None),
        )

    def test_fully_paired_escape_surface_reports_nothing_inert(self) -> None:
        surface = agy_dispatch.normalize_permission_surface(
            {
                "allow": [
                    "command(rustfmt --edition 2021)",
                    "command(cargo build -p x --lib)",
                    "unsandboxed(cargo build -p x --lib)",
                ],
                "deny": [],
                "ask": [],
            }
        )
        self.assertEqual(agy_dispatch.inert_unsandboxed_rules(surface), [])

    def test_task_command_check_reports_sandbox_escape_per_command(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["project_permissions"] = {
            "allow": [
                "command(rustfmt --edition 2021)",
                "command(cargo build -p x --lib)",
                "unsandboxed(cargo build -p x --lib)",
            ],
            "deny": [],
            "ask": [],
        }
        profile["task_commands"] = {
            "allow": [
                "rustfmt --edition 2021 src/a.rs",
                "cargo build -p x --lib",
            ],
            "deny": [],
        }
        project_path = self.project_dir / "project-a.json"
        project = json.loads(project_path.read_text())
        project["permissionGrants"]["permissionGrants"] = (
            agy_dispatch.normalize_permission_surface(
                profile["project_permissions"]
            )
        )
        project_path.write_text(json.dumps(project))
        report = agy_dispatch.project_policy_report(profile)
        escapes = {
            check["command"]: check["unsandboxed"]
            for check in report["task_command_checks"]
        }
        self.assertEqual(
            escapes,
            {
                "rustfmt --edition 2021 src/a.rs": False,
                "cargo build -p x --lib": True,
            },
        )
        # Reported, not blocked: whether a command needs the escape is a
        # judgement the profile cannot state.
        self.assertEqual(report["blockers"], [])

    def test_permission_digest_detects_midrun_project_drift(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "1")
        snapshot = {
            "permission_state_digest": agy_dispatch.permission_state_digest(
                profile
            )
        }
        project_path = self.project_dir / "project-a.json"
        project = json.loads(project_path.read_text())
        grants = project["permissionGrants"]["permissionGrants"]
        grants["allow"].append("command(cargo test)")
        project_path.write_text(json.dumps(project))
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.assert_permission_state_unchanged(profile, snapshot)
        self.assertIn("permission state changed", str(caught.exception))

    def test_explicit_project_must_match_profile_root(self) -> None:
        profile = self.profile(self.repo_b, "project-a", "3")
        with self.assertRaises(SystemExit):
            agy_dispatch.agy_project_id(profile)

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

    def test_extracts_denied_run_command_from_protobuf_payload(self) -> None:
        payload = (
            b"\x08\x15garbage"
            b'{"CommandLine":"rg -c NATIVE_FUNC_ADDRS\\\\.with projects/mamba",'
            b'"Cwd":"/repo","WaitMsBeforeAsync":5000}'
            b"\x00trailer"
        )
        self.assertEqual(
            agy_dispatch.extract_run_command_lines(payload),
            [r"rg -c NATIVE_FUNC_ADDRS\.with projects/mamba"],
        )

    def write_conversation(
        self,
        conversation_id: str,
        commands: list[tuple[int, str]],
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
            for idx, command in commands:
                payload = (
                    b"prefix"
                    + json.dumps(
                        {"CommandLine": command},
                        separators=(",", ":"),
                    ).encode()
                    + b"suffix"
                )
                connection.execute(
                    "insert into steps values (?, 15, 3, ?)",
                    (idx, payload),
                )
            connection.commit()
        finally:
            connection.close()

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
                (2, "pwd"),
                (3, "rg -n TODO src"),
            ],
        )
        audited = agy_dispatch.audit_task_commands(
            profile,
            "7",
            {
                "conversation_id": "conversation-7",
                "conversation_step_floor": 1,
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

    def test_prompt_separates_static_project_policy_from_ticket_commands(
        self,
    ) -> None:
        profile = self.profile(self.repo_a, "project-a", "4")
        prompt = agy_dispatch.render_prompt(
            profile,
            "4",
            "oracle",
            {"number": 4, "state": "OPEN"},
        )
        self.assertIn("Persistent project-policy digest", prompt)
        self.assertIn("authorized for this task", prompt)
        self.assertIn("broader reusable tool access", prompt)
        self.assertIn(
            "Every Bash tool call must copy one authorized command line "
            "byte-for-byte.",
            prompt,
        )
        self.assertIn("last report marker", prompt)

    def test_prompt_demands_a_verdict_only_when_the_worker_can_observe(
        self,
    ) -> None:
        profile = self.profile(self.repo_a, "project-a", "4")
        prompt = agy_dispatch.render_prompt(
            profile,
            "4",
            "oracle",
            {"number": 4, "state": "OPEN"},
        )
        self.assertIn("PASS or FAIL per criterion", prompt)

    def test_no_shell_prompt_forbids_a_verdict_it_cannot_support(self) -> None:
        profile = self.profile(self.repo_a, "project-a", "4")
        profile["task_commands"] = {"allow": [], "deny": []}
        profile["project_permissions"] = {
            "allow": [],
            "deny": [],
            "ask": [],
            "require_empty_global": True,
        }
        prompt = agy_dispatch.render_prompt(
            profile,
            "4",
            "oracle",
            {"number": 4, "state": "OPEN"},
        )
        self.assertNotIn("PASS or FAIL per criterion", prompt)
        self.assertIn("This round grants you NO shell", prompt)
        self.assertIn("Do not write `PASS`", prompt)
        self.assertIn("Report only what you wrote", prompt)
        self.assertIn("prescribes its own `## EXEC REPORT`", prompt)

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
        self.assertIn("state_dir must be under /tmp", str(caught.exception))

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


class DerivedWorktreeTest(unittest.TestCase):
    """The worker runs in its own checkout; the diff is the acceptance surface."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/tmp")
        self.root = Path(self.temporary.name)
        self.project_dir = self.root / "projects"
        self.project_dir.mkdir()
        agy_dispatch.PROJECT_DIR = self.project_dir
        self.controller = self.root / "controller"
        self.controller.mkdir()
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.email", "controller@example.test")
        self.git("config", "user.name", "Controller")
        (self.controller / "README.md").write_text("base\n")
        (self.controller / "keep.md").write_text("frozen\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "base")
        self.write_project("project-a", self.controller)
        self.conversation_dir = self.root / "conversations"
        self.conversation_dir.mkdir()
        agy_dispatch.CONVERSATION_DIR = self.conversation_dir

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def git(self, *args: str) -> str:
        return agy_dispatch.git_output(self.controller, *args)

    def write_project(self, project_id: str, root: Path) -> None:
        (self.project_dir / f"{project_id}.json").write_text(
            json.dumps(
                {
                    "id": project_id,
                    "name": project_id,
                    "projectResources": {
                        "resources": [
                            {"gitFolder": {"folderUri": root.resolve().as_uri()}}
                        ]
                    },
                    "permissionGrants": {
                        "permissionGrants": {
                            "allow": ["command(pwd)"],
                            "deny": ["command(git push)"],
                            "ask": [],
                        }
                    },
                    "settings": {},
                }
            )
        )

    def profile_path(self, **overrides: object) -> Path:
        profile = {
            "controller_root": str(self.controller),
            "root": str(self.controller),
            "repo": "owner/repo",
            "agy_project_id": "project-a",
            "state_dir": str(self.root / "state"),
            "mode": "bounded-write",
            "task_contract": {
                "kind": "implementation",
                "session_policy": "one-shot",
                "run_id": "round-1",
                "intent": "bounded change",
                "design_inputs": [],
                # Tree-dependent on purpose: the gate must be able to tell the
                # reverted tree from the candidate, or `prove` measures nothing.
                "gate_command": "grep -q accepted README.md",
            },
            "project_permissions": {
                "allow": ["command(pwd)"],
                "deny": ["command(git push)"],
                "ask": [],
                "require_empty_global": True,
            },
            "task_commands": {"allow": [], "deny": []},
            "protected_artifacts": [],
            "snapshot_paths": [],
            "allowed_repo_writes": ["README.md"],
            "path_change_budgets": {},
            **overrides,
        }
        path = self.root / "profile.json"
        path.write_text(json.dumps(profile, indent=2))
        return path

    def contract_with_design_input(self) -> dict:
        return {
            "kind": "implementation",
            "session_policy": "one-shot",
            "run_id": "round-1",
            "intent": "bounded change",
            "gate_command": "grep -q accepted README.md",
            "design_inputs": [
                {
                    "path": "keep.md",
                    "sha256": agy_dispatch.sha256(self.controller / "keep.md"),
                }
            ],
        }

    def record_proofs(
        self, profile: dict, worker: Path, task_key: str = "round-1"
    ) -> None:
        """Measure the gate the way a controller must: once with the product
        change reverted, once with it restored."""
        candidate = (worker / "README.md").read_text()
        (worker / "README.md").write_text("base\n")
        agy_dispatch.prove(profile, task_key, "mutant")
        (worker / "README.md").write_text(candidate)
        agy_dispatch.prove(profile, task_key, "candidate")

    def derive(self, path: Path, task_key: str = "round-1") -> dict:
        agy_dispatch.worktree(str(path), task_key)
        return json.loads(path.read_text())

    def project_binding(self, project_id: str = "project-a") -> Path:
        document = json.loads((self.project_dir / f"{project_id}.json").read_text())
        return agy_dispatch.project_root(document)

    def test_worker_gets_its_own_checkout_and_branch(self) -> None:
        profile = self.derive(self.profile_path())
        spec = profile["worktree"]
        self.assertEqual(spec["branch"], "agy/round-1")
        self.assertEqual(profile["root"], spec["path"])
        self.assertTrue(Path(spec["path"]).is_dir())
        self.assertNotEqual(profile["root"], str(self.controller))
        self.assertEqual(
            agy_dispatch.git_output(
                Path(spec["path"]), "rev-parse", "--abbrev-ref", "HEAD"
            ).strip(),
            "agy/round-1",
        )

    def project_grants(self, project_id: str = "project-a") -> dict:
        document = json.loads((self.project_dir / f"{project_id}.json").read_text())
        return agy_dispatch.project_permission_surface(document)

    def widen_grants(self, rule: str, project_id: str = "project-a") -> None:
        path = self.project_dir / f"{project_id}.json"
        document = json.loads(path.read_text())
        document["permissionGrants"]["permissionGrants"]["allow"].append(rule)
        path.write_text(json.dumps(document, indent=2))

    def test_grant_applies_the_declared_surface_and_discard_undoes_it(
        self,
    ) -> None:
        """The profile is the only statement of the round's surface; applying
        it by hand to AGY's JSON was the step that kept going wrong."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        loaded = json.loads(path.read_text())
        loaded["project_permissions"]["allow"].append("command(cargo test)")
        path.write_text(json.dumps(loaded))
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn("command(cargo test)", self.project_grants()["allow"])
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.discard(str(path), "round-1")
        self.assertNotIn("command(cargo test)", self.project_grants()["allow"])

    def test_a_missing_injection_still_blocks_dispatch(self) -> None:
        """`scaffold` runs before the injection exists, so loading must tolerate
        its absence — but only for authoring, never on the way to a worker."""
        path = self.profile_path(
            task_contract=self.contract_with_design_input(),
            inject_prompt_file=str(self.root / "state" / "never-written.md"),
        )
        agy_dispatch.load_profile(str(path), validate_design=False)
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(path))
        self.assertIn("inject_prompt_file is not a file", str(caught.exception))

    def test_grant_refuses_without_a_restorable_baseline(self) -> None:
        """Widening a Project that `discard` cannot narrow again is how a
        round-local grant becomes permanent by accident."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        profile = self.derive(path)
        agy_dispatch.grants_baseline_path(
            Path(profile["state_dir"]), "project-a"
        ).unlink()
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn("no grants baseline", str(caught.exception))

    def test_doctor_runs_before_the_injection_is_scaffolded(self) -> None:
        """`doctor` preflights the round; `scaffold` is what creates the
        injection, so requiring it here would make the loop unrunnable."""
        path = self.profile_path(
            task_contract=self.contract_with_design_input(),
            inject_prompt_file=str(self.root / "not-written-yet.md"),
        )
        profile = agy_dispatch.load_profile(str(path), require_injection=False)
        self.assertEqual(
            profile["inject_prompt_file"], str(self.root / "not-written-yet.md")
        )
        with self.assertRaises(SystemExit):
            agy_dispatch.load_profile(str(path))

    def test_round_local_grants_are_withdrawn_on_discard(self) -> None:
        path = self.profile_path()
        self.derive(path)
        before = self.project_grants()
        self.widen_grants("command(cargo test)")
        self.assertIn("command(cargo test)", self.project_grants()["allow"])

        agy_dispatch.discard(str(path), "round-1")
        self.assertEqual(self.project_grants(), before)

    def test_discard_without_a_baseline_leaves_grants_alone(self) -> None:
        """An older round's profile has no recorded baseline. Restoring from a
        guess would withdraw grants the controller never granted."""
        path = self.profile_path()
        self.derive(path)
        state = Path(json.loads(path.read_text())["state_dir"])
        agy_dispatch.grants_baseline_path(state, "project-a").unlink()
        self.widen_grants("command(cargo test)")

        agy_dispatch.discard(str(path), "round-1")
        self.assertIn("command(cargo test)", self.project_grants()["allow"])

    def test_a_second_derive_does_not_overwrite_an_open_baseline(self) -> None:
        """Re-deriving after granting a round-local rule must not promote that
        rule into the baseline, or the shared Project ratchets wider forever."""
        path = self.profile_path()
        self.derive(path)
        before = self.project_grants()
        self.widen_grants("command(cargo test)")
        self.derive(path)

        agy_dispatch.discard(str(path), "round-1")
        self.assertEqual(self.project_grants(), before)

    def test_one_project_moves_to_the_round_and_returns_home(self) -> None:
        path = self.profile_path()
        profile = self.derive(path)
        self.assertEqual(
            self.project_binding(),
            Path(profile["worktree"]["path"]).resolve(),
        )
        self.assertEqual(
            Path(profile["worktree"]["project_home_root"]).resolve(),
            self.controller.resolve(),
        )
        agy_dispatch.discard(str(path), "round-1")
        self.assertEqual(self.project_binding(), self.controller.resolve())

    def test_deriving_never_widens_the_reviewed_permission_surface(self) -> None:
        before = json.loads(
            (self.project_dir / "project-a.json").read_text()
        )["permissionGrants"]
        self.derive(self.profile_path())
        after = json.loads(
            (self.project_dir / "project-a.json").read_text()
        )["permissionGrants"]
        self.assertEqual(before, after)

    def test_worker_branch_must_be_namespaced(self) -> None:
        path = self.profile_path(worktree={"branch": "main"})
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.worktree(str(path), "round-1")
        self.assertIn("must start with 'agy/'", str(caught.exception))

    def test_worker_checkout_may_not_nest_inside_the_controller(self) -> None:
        path = self.profile_path(
            worktree={"path": str(self.controller / "nested")}
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.worktree(str(path), "round-1")
        self.assertIn("outside controller_root", str(caught.exception))

    def test_scope_overrun_is_a_finding_not_a_lost_round(self) -> None:
        profile = self.derive(
            self.profile_path(path_change_budgets={"README.md": 1})
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\nline a\nline b\nline c\n")
        (worker / "stray.md").write_text("undeclared\n")

        touched = agy_dispatch.worker_touched_paths(profile)
        self.assertEqual(touched, ["README.md", "stray.md"])
        findings = agy_dispatch.scope_findings(profile, touched)
        self.assertTrue(
            any("outside allowed_repo_writes" in item for item in findings)
        )
        self.assertTrue(any("exceeds the 1-line budget" in item for item in findings))
        # The candidate survives: the controller still has a diff to read.
        self.assertIn("line a", agy_dispatch.git_output(
            worker, "diff", profile["worktree"]["base_sha"]
        ))

    def dead_round(
        self,
        *,
        commands: tuple[tuple[int, str], ...] = (),
        report: str = "request failed (code 502)\n",
        task_key: str = "round-1",
        conversation_id: str = "conversation-dead",
    ) -> dict:
        """A round whose dispatch recorded a conversation, plus whatever the
        worker left behind before the run ended."""
        profile = self.derive(self.profile_path())
        state = Path(profile["state_dir"])
        (state / "snapshots").mkdir(parents=True, exist_ok=True)
        (state / "snapshots" / f"{task_key}.json").write_text(
            json.dumps(
                {
                    "task_key": task_key,
                    "session_policy": "one-shot",
                    "conversation_id": None,
                    "conversation_step_floor": -1,
                }
            )
        )
        runs = state / "runs"
        runs.mkdir(parents=True, exist_ok=True)
        (runs / f"{task_key}.conversation").write_text(conversation_id + "\n")
        (runs / f"{task_key}.agy.log").write_text(
            f"Created conversation {conversation_id}\n"
        )
        (runs / f"{task_key}.log").write_text(report)
        (runs / f"{task_key}.prompt.md").write_text("prompt\n")
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)
        try:
            connection.execute(
                "create table steps ("
                "idx integer primary key, step_type integer not null, "
                "status integer not null, step_payload blob)"
            )
            for idx, command in commands:
                connection.execute(
                    "insert into steps values (?, 15, 3, ?)",
                    (
                        idx,
                        json.dumps(
                            {"CommandLine": command}, separators=(",", ":")
                        ).encode(),
                    ),
                )
            connection.commit()
        finally:
            connection.close()
        return profile

    def test_a_dispatch_that_produced_nothing_releases_its_run_id(self) -> None:
        profile = self.dead_round()
        self.assertEqual(
            agy_dispatch.conversation_id_for_task(profile, "round-1"),
            "conversation-dead",
        )

        agy_dispatch.abandon(profile, "round-1")

        # Both sources of the id are gone: the recorded file and the run log
        # the lookup falls back to.
        self.assertIsNone(
            agy_dispatch.conversation_id_for_task(profile, "round-1")
        )
        # The point of the release: the same run id dispatches again, so the
        # linted round documents are not re-authored under a fresh id.
        self.assertIsNone(
            agy_dispatch.validate_conversation_action(
                profile, "round-1", resume=False
            )
        )
        parked = Path(profile["state_dir"]) / "runs" / "abandoned" / "round-1.1"
        self.assertEqual(
            sorted(item.name for item in parked.iterdir()),
            [
                "abandoned.json",
                "round-1.agy.log",
                "round-1.conversation",
                "round-1.log",
                "round-1.prompt.md",
            ],
        )

    def test_abandon_refuses_when_the_worker_left_a_candidate(self) -> None:
        profile = self.dead_round()
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\nbounded\n")

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.abandon(profile, "round-1")

        self.assertIn("changed 1 path(s) (README.md)", str(error.exception))
        # The dead-run claim was rejected, so the id stays spent.
        self.assertEqual(
            agy_dispatch.conversation_id_for_task(profile, "round-1"),
            "conversation-dead",
        )

    def test_abandon_refuses_when_the_worker_ran_commands(self) -> None:
        profile = self.dead_round(commands=((0, "pwd"),))

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.abandon(profile, "round-1")

        self.assertIn("ran 1 command(s)", str(error.exception))

    def test_abandon_refuses_when_the_worker_filed_a_report(self) -> None:
        profile = self.dead_round(report="## EXEC REPORT\nverdict: PASS\n")

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.abandon(profile, "round-1")

        self.assertIn("filed an EXEC REPORT", str(error.exception))

    def test_abandon_needs_a_conversation_to_release(self) -> None:
        profile = self.derive(self.profile_path())

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.abandon(profile, "round-1")

        self.assertIn("no conversation is recorded", str(error.exception))

    def test_a_second_dead_attempt_parks_beside_the_first(self) -> None:
        profile = self.dead_round()
        agy_dispatch.abandon(profile, "round-1")
        # A re-dispatch opens its own conversation, and that one dies too.
        self.dead_round(conversation_id="conversation-dead-again")

        agy_dispatch.abandon(profile, "round-1")

        abandoned = Path(profile["state_dir"]) / "runs" / "abandoned"
        self.assertEqual(
            sorted(item.name for item in abandoned.iterdir()),
            ["round-1.1", "round-1.2"],
        )

    def test_a_clean_round_reports_no_findings(self) -> None:
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\nbounded\n")
        touched = agy_dispatch.worker_touched_paths(profile)
        self.assertEqual(
            agy_dispatch.scope_findings(profile, touched),
            [],
        )

    def test_a_worker_commit_is_reported_as_a_moved_head(self) -> None:
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\nbounded\n")
        agy_dispatch.git_output(worker, "add", "-A")
        agy_dispatch.git_output(worker, "commit", "-qm", "worker overreach")
        findings = agy_dispatch.scope_findings(profile, [])
        self.assertTrue(any("HEAD moved" in item for item in findings))

    def test_accept_commits_on_the_worker_branch_only(self) -> None:
        path = self.profile_path()
        profile = self.derive(path)
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        agy_dispatch.accept(profile, "round-1")

        self.assertEqual(
            agy_dispatch.git_output(worker, "rev-parse", "--abbrev-ref", "HEAD").strip(),
            "agy/round-1",
        )
        self.assertEqual(self.git("rev-parse", "--abbrev-ref", "HEAD").strip(), "main")
        self.assertEqual((self.controller / "README.md").read_text(), "base\n")
        self.assertEqual(self.git("status", "--porcelain").strip(), "")

    def test_discard_keeps_the_branch_when_asked(self) -> None:
        path = self.profile_path()
        profile = self.derive(path)
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\nkept\naccepted\n")
        self.record_proofs(profile, worker)
        agy_dispatch.accept(profile, "round-1")
        agy_dispatch.discard(str(path), "round-1", keep_branch=True)
        self.assertIn("agy/round-1", self.git("branch", "--list", "agy/round-1"))

    def test_accept_refuses_without_a_proof_pair(self) -> None:
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.accept(profile, "round-1")
        self.assertIn("not shown to discriminate", str(caught.exception))

    def test_accept_refuses_when_the_gate_passes_without_the_change(self) -> None:
        """The false green this whole mechanism exists for: a gate written
        against the implementation it is supposed to judge."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        # Both proofs over the candidate tree: nothing was reverted.
        agy_dispatch.prove(profile, "round-1", "mutant")
        agy_dispatch.prove(profile, "round-1", "candidate")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.accept(profile, "round-1")
        message = str(caught.exception)
        self.assertIn("passed with the product change reverted", message)
        self.assertIn("identical tree", message)

    def test_accept_refuses_when_the_tree_moved_after_the_proof(self) -> None:
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        (worker / "README.md").write_text("base\naccepted\nand more\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.accept(profile, "round-1")
        self.assertIn("changed after the `candidate` proof", str(caught.exception))

    def test_bounded_write_requires_a_gate_command(self) -> None:
        contract = self.contract_with_design_input()
        del contract["gate_command"]
        path = self.profile_path(task_contract=contract)
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(path))
        self.assertIn("task_contract.gate_command", str(caught.exception))

    def test_review_requires_a_derived_checkout(self) -> None:
        profile = json.loads(self.profile_path().read_text())
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.worker_touched_paths(profile)
        self.assertIn("worktree.base_sha", str(caught.exception))

    def test_protected_path_from_another_root_is_rejected(self) -> None:
        """A profile whose protected paths are absolute would silently pin the
        controller's tree instead of the worker's — missing a real protected
        mutation and voiding on the controller's own edits."""
        path = self.profile_path(
            task_contract=self.contract_with_design_input(),
            protected_artifacts=[
                {
                    "path": str(self.controller / "keep.md"),
                    "sha256": agy_dispatch.sha256(self.controller / "keep.md"),
                }
            ],
        )
        self.derive(path)
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(path))
        self.assertIn("outside the round root", str(caught.exception))

    def test_relative_protected_path_follows_the_round(self) -> None:
        path = self.profile_path(
            task_contract=self.contract_with_design_input(),
            protected_artifacts=[
                {
                    "path": "keep.md",
                    "sha256": agy_dispatch.sha256(self.controller / "keep.md"),
                }
            ],
        )
        profile = self.derive(path)
        loaded = agy_dispatch.load_profile(str(path))
        self.assertEqual(
            Path(loaded["protected_artifacts"][0]["path"]).resolve(),
            (Path(profile["worktree"]["path"]) / "keep.md").resolve(),
        )


CONFORMANT_ORACLE = """\
## Claim

Writing an AW-owned marker block does not move the WI contract digest.

## Measurements

| # | input | expected observation |
|---|---|---|
| 1 | baseline body | digest D |
| 2 | marker updated_at bumped | digest still D |
| 3 | prose edited (negative control) | digest != D |

## Gate

```
cargo test -p target --lib some_gate
```

## Fabrication tells

- A helper that is defined but never called from the reducer.
- A fixture that carries no marker block at all.
"""


class OracleContractTest(unittest.TestCase):
    """Each rule must fail on its own defect and on nothing else.

    A structural checker that cannot be shown to reject the thing it claims to
    reject is the same false green the dispatcher exists to prevent, so every
    rule below is measured against a single-defect mutation of one conformant
    oracle.
    """

    PROFILE = {
        "task_commands": {"allow": ["cargo test -p target --lib some_gate"]}
    }

    def findings(self, text: str) -> list[str]:
        return agy_dispatch.oracle_findings(self.PROFILE, text)

    def assert_single_finding(self, text: str, fragment: str) -> None:
        found = self.findings(text)
        self.assertEqual(len(found), 1, f"expected one finding, got {found}")
        self.assertIn(fragment, found[0])

    def test_conformant_oracle_has_no_findings(self) -> None:
        self.assertEqual(self.findings(CONFORMANT_ORACLE), [])

    def test_missing_section_is_reported_once(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "## Claim\n\nWriting an AW-owned marker block does not move the "
            "WI contract digest.\n\n",
            "",
        )
        self.assert_single_finding(text, "missing the `## Claim` section")

    def test_empty_claim_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "Writing an AW-owned marker block does not move the WI contract "
            "digest.",
            "",
        )
        self.assert_single_finding(text, "`## Claim` is empty")

    def test_table_below_two_rows_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "| 2 | marker updated_at bumped | digest still D |\n", ""
        ).replace("| 3 | prose edited (negative control) | digest != D |\n", "")
        self.assert_single_finding(text, "needs at least 2 measured rows")

    def test_missing_negative_control_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "prose edited (negative control)", "prose edited"
        )
        self.assert_single_finding(text, "no row marked `negative control`")

    def test_gate_outside_the_command_allowlist_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace("some_gate\n```", "other_gate\n```")
        self.assert_single_finding(text, "not authorized")

    def test_unfenced_gate_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "```\ncargo test -p target --lib some_gate\n```",
            "run the gate yourself",
        )
        self.assert_single_finding(text, "no fenced command block")

    def test_fabrication_tells_without_a_list_item_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "- A helper that is defined but never called from the reducer.\n"
            "- A fixture that carries no marker block at all.\n",
            "None known.\n",
        )
        self.assert_single_finding(text, "needs at least one list item")

    def test_out_of_order_sections_are_reported(self) -> None:
        text = (
            "## Claim\n\nX.\n\n"
            "## Gate\n\n```\ncargo test -p target --lib some_gate\n```\n\n"
            "## Measurements\n\n| # | in | out |\n|---|---|---|\n"
            "| 1 | a | b |\n| 2 | c (negative control) | d |\n\n"
            "## Fabrication tells\n\n- tell\n"
        )
        self.assert_single_finding(text, "out of order")

    def test_gate_is_not_cross_checked_when_the_round_grants_no_shell(
        self,
    ) -> None:
        """A measure-only round authorizes no command, so the gate names what
        the controller will run. Cross-checking it against an empty allowlist
        would reject every such oracle."""
        profile = {"task_commands": {"allow": []}}
        text = CONFORMANT_ORACLE.replace("some_gate\n```", "controller_gate\n```")
        self.assertEqual(agy_dispatch.oracle_findings(profile, text), [])


CONFORMANT_INJECTION = """\
## Task

Exclude AW-owned marker blocks from the contract digest.

## Current behavior

`src/thing.rs:12` hashes the whole body, markers included:

```
let digest = canonical_digest(body);
```

## Required change

Two bodies differing only inside an AW-owned marker block must produce the
same stored contract digest.

## Shape to follow

`strip_aw_marker_blocks` already knows the marker vocabulary; match it rather
than introducing a second notion of what AW owns.

## Reference

| path | why the worker must read it |
|---|---|
| `src/thing.rs` | the reducer that stores the digest |

## Out of scope

- The marker vocabulary itself; this round changes only what the digest covers.

## Definition of done

The new check joins the existing `mod tests` in `src/thing.rs`.

```
cargo test -p target --lib some_gate
```
"""


class InjectionContractTest(unittest.TestCase):
    """The half of the round that says what to do, held to the same standard.

    The oracle contract was written after a false green entered through free
    prose. This document is the other free-prose half, so each of its rules is
    likewise measured against a single-defect mutation of one conformant form.
    """

    def setUp(self) -> None:
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        (self.root / "src").mkdir()
        (self.root / "src" / "thing.rs").write_text("fn main() {}\n")
        self.profile = {
            "root": str(self.root),
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
        }

    def findings(self, text: str) -> list[str]:
        return agy_dispatch.injection_findings(
            self.profile, text, CONFORMANT_ORACLE
        )

    def assert_single_finding(self, text: str, fragment: str) -> None:
        found = self.findings(text)
        self.assertEqual(len(found), 1, f"expected one finding, got {found}")
        self.assertIn(fragment, found[0])

    def test_conformant_injection_has_no_findings(self) -> None:
        self.assertEqual(self.findings(CONFORMANT_INJECTION), [])

    def test_missing_section_is_reported_once(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "## Task\n\nExclude AW-owned marker blocks from the contract "
            "digest.\n\n",
            "",
        )
        self.assert_single_finding(text, "missing the `## Task` section")

    def test_out_of_order_sections_are_reported(self) -> None:
        sections = CONFORMANT_INJECTION.split("## Required change")
        text = (
            sections[0].split("## Current behavior")[0]
            + "## Required change"
            + sections[1].split("## Reference")[0]
            + "## Current behavior"
            + CONFORMANT_INJECTION.split("## Current behavior")[1].split(
                "## Required change"
            )[0]
            + "## Reference"
            + CONFORMANT_INJECTION.split("## Reference")[1]
        )
        self.assert_single_finding(text, "out of order")

    def test_empty_task_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "Exclude AW-owned marker blocks from the contract digest.", ""
        )
        self.assert_single_finding(text, "`## Task` is empty")

    def test_quoteless_current_behavior_is_reported(self) -> None:
        """Without a verbatim quote the round can be authored from memory, which
        is exactly how a stale line number reaches a worker."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "it hashes everything.",
        )
        self.assert_single_finding(text, "no non-empty fenced quote")

    def test_empty_fence_does_not_count_as_a_quote(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```", "```\n```"
        )
        self.assert_single_finding(text, "no non-empty fenced quote")

    def test_reference_naming_nothing_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "| path | why the worker must read it |\n|---|---|\n"
            "| `src/thing.rs` | the reducer that stores the digest |",
            "read whatever seems relevant.",
        )
        self.assert_single_finding(text, "names nothing to read")

    def test_out_of_scope_without_a_list_item_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "- The marker vocabulary itself; this round changes only what the "
            "digest covers.",
            "Nothing in particular.",
        )
        self.assert_single_finding(text, "`## Out of scope` needs at least one")

    def test_unfenced_definition_of_done_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "```\ncargo test -p target --lib some_gate\n```",
            "the suite should pass.",
        )
        self.assert_single_finding(text, "has no fenced command block")

    def test_gate_drift_between_the_two_halves_is_reported(self) -> None:
        """The instruction and the judgement must name the same command; when
        they drifted apart, each was satisfiable without the other."""
        text = CONFORMANT_INJECTION.replace("some_gate", "other_gate")
        self.assert_single_finding(text, "names a different gate")

    def test_unfilled_scaffold_slot_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "Exclude AW-owned marker blocks from the contract digest.",
            "<!-- fill: one imperative sentence -->",
        )
        self.assert_single_finding(text, "unfilled")

    def test_path_absent_from_the_checkout_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace("src/thing.rs", "src/gone.rs")
        self.assert_single_finding(text, "do not exist in the worker's checkout")

    def test_pasted_implementation_is_reported(self) -> None:
        """A round whose answer is already written has nothing left to dispatch;
        the worker is then paid to retype what the controller already derived."""
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest:\n\n```\n"
            "let digest = canonical_digest(&strip_aw_marker_blocks(body));\n```",
        )
        self.assert_single_finding(text, "contains a fenced block")

    def test_numbered_recipe_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "1. Strip the marker blocks.\n2. Hash what is left.",
        )
        self.assert_single_finding(text, "reads as numbered steps")

    def test_shape_naming_no_existing_symbol_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "`strip_aw_marker_blocks` already knows the marker vocabulary; match "
            "it rather\nthan introducing a second notion of what AW owns.",
            "Follow whatever convention seems most natural in the file.",
        )
        self.assert_single_finding(text, "names no existing symbol or file")

    def test_shape_that_grew_into_a_design_is_reported(self) -> None:
        """Past a few lines the constraint has become the answer, which is the
        work the round was dispatched to buy."""
        text = CONFORMANT_INJECTION.replace(
            "`strip_aw_marker_blocks` already knows the marker vocabulary; match "
            "it rather\nthan introducing a second notion of what AW owns.",
            "\n".join(
                ["Follow `strip_aw_marker_blocks`."]
                + [f"Then handle case {n} of the rewrite." for n in range(6)]
            ),
        )
        self.assert_single_finding(text, "keep it within")

    def test_definition_of_done_without_a_landing_spot_is_reported(self) -> None:
        text = CONFORMANT_INJECTION.replace(
            "The new check joins the existing `mod tests` in `src/thing.rs`.\n\n",
            "",
        )
        self.assert_single_finding(text, "not where its check lands")

    def test_identifiers_are_not_mistaken_for_paths(self) -> None:
        """`cli::chain::tests` and `--lib` are not files; flagging them would
        make the check noise the controller learns to ignore."""
        text = CONFORMANT_INJECTION.replace(
            "the reducer that stores the digest",
            "reducer `fold_wi_create`, module `cli::chain::tests`, flag `--lib`",
        )
        self.assertEqual(self.findings(text), [])


class ScaffoldTest(unittest.TestCase):
    """The blank form must be unusable until it is filled."""

    def setUp(self) -> None:
        self.state = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.state, ignore_errors=True)
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        (self.root / "design.md").write_text("design\n")
        self.profile = {
            "root": str(self.root),
            "state_dir": str(self.state),
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "inject_prompt_file": str(self.state / "injections" / "r1.md"),
            "task_contract": {
                "gate_command": "cargo test -p target --lib some_gate",
                "design_inputs": [{"path": "design.md", "sha256": "x"}],
            },
        }

    def scaffold(self) -> None:
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.scaffold(self.profile, "r1")

    def test_scaffold_writes_both_halves_prefilled_with_the_gate(self) -> None:
        self.scaffold()
        oracle = (self.state / "oracles" / "r1.md").read_text()
        injection = (self.state / "injections" / "r1.md").read_text()
        self.assertEqual(
            agy_dispatch.gate_commands_in(
                agy_dispatch.oracle_sections(oracle)["Gate"]
            ),
            ["cargo test -p target --lib some_gate"],
        )
        self.assertEqual(
            agy_dispatch.gate_commands_in(
                agy_dispatch.oracle_sections(injection)["Definition of done"]
            ),
            ["cargo test -p target --lib some_gate"],
        )
        self.assertIn("`design.md`", injection)

    def test_a_blank_form_cannot_be_dispatched(self) -> None:
        """The point of handing out slots is lost if an unfilled form passes."""
        self.scaffold()
        findings = agy_dispatch.round_findings(self.profile, "r1")
        self.assertTrue(findings, "a blank scaffold produced no findings")
        self.assertTrue(
            any("unfilled" in item for item in findings),
            f"blank slots were not what blocked it: {findings}",
        )

    def test_scaffold_never_overwrites_authored_content(self) -> None:
        self.scaffold()
        oracle = self.state / "oracles" / "r1.md"
        oracle.write_text(CONFORMANT_ORACLE)
        self.scaffold()
        self.assertEqual(oracle.read_text(), CONFORMANT_ORACLE)

    def test_a_declared_but_missing_injection_blocks_the_round(self) -> None:
        (self.state / "oracles").mkdir(parents=True)
        (self.state / "oracles" / "r1.md").write_text(CONFORMANT_ORACLE)
        findings = agy_dispatch.round_findings(self.profile, "r1")
        self.assertEqual(len(findings), 1)
        self.assertIn("declared injection is missing", findings[0])

    def test_an_undeclared_injection_is_not_required(self) -> None:
        """A measure-only round can carry its whole instruction in the oracle."""
        profile = dict(self.profile)
        profile.pop("inject_prompt_file")
        (self.state / "oracles").mkdir(parents=True)
        (self.state / "oracles" / "r1.md").write_text(CONFORMANT_ORACLE)
        self.assertEqual(agy_dispatch.round_findings(profile, "r1"), [])


if __name__ == "__main__":
    unittest.main()
