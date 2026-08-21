#!/usr/bin/env python3
from __future__ import annotations

import ast
import contextlib

import importlib.util
import io
import json
import os
import shutil
import sqlite3
import subprocess
import sys
import tempfile
import unittest
import unittest.mock
from pathlib import Path


SCRIPT = Path(__file__).with_name("agy_dispatch.py")
MAKE_PROFILE = Path(__file__).with_name("make_profile.py")
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

    def test_a_write_target_that_is_also_frozen_blocks_dispatch(self) -> None:
        """The round would end in a finding whichever way it went: writing the
        path changes a protected artifact, and not writing it leaves a declared
        path unwritten. A finding that fires on correct work teaches the
        controller to skim the list the protected set depends on (#3428)."""
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [{"path": "src/a.py", "sha256": "deadbeef"}]
        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(report["dispatch_ready"])
        self.assertTrue(
            any("src/a.py" in blocker for blocker in report["blockers"]),
            report["blockers"],
        )

    def test_a_frozen_path_the_round_does_not_write_is_fine(self) -> None:
        """The other half. Freezing the complement is the whole design, so the
        blocker must key on the intersection and not on a profile having both
        lists non-empty."""
        profile = self.profile(self.repo_a, "project-a", "1")
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [{"path": "src/b.py", "sha256": "deadbeef"}]
        report = agy_dispatch.project_policy_report(profile)
        self.assertTrue(report["dispatch_ready"], report["blockers"])

    def test_loaded_profile_with_repo_relative_protected_write_target_blocks_dispatch(
        self,
    ) -> None:
        """A profile written to disk with repo-relative protected_artifacts
        matching allowed_repo_writes, read back with load_profile, then passed
        to project_policy_report, blocks dispatch and names the offending path."""
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["mode"] = "bounded-write"
        profile["task_contract"]["kind"] = "implementation"
        profile["task_contract"]["gate_command"] = "pwd"
        profile["task_contract"]["design_inputs"] = [
            {"path": "SKILL.md", "sha256": "deadbeef"}
        ]
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [{"path": "src/a.py", "sha256": "deadbeef"}]
        profile_path = self.root / "profile-relative.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(
            str(profile_path), require_injection=False, validate_design=False
        )
        report = agy_dispatch.project_policy_report(loaded)
        self.assertFalse(report["dispatch_ready"])
        self.assertTrue(
            any("src/a.py" in blocker for blocker in report["blockers"]),
            report["blockers"],
        )

    def test_loaded_profile_with_absolute_protected_write_target_blocks_dispatch(
        self,
    ) -> None:
        """A profile written to disk with absolute protected_artifacts under root
        matching allowed_repo_writes, read back with load_profile, then passed
        to project_policy_report, blocks dispatch and names the offending path."""
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["mode"] = "bounded-write"
        profile["task_contract"]["kind"] = "implementation"
        profile["task_contract"]["gate_command"] = "pwd"
        profile["task_contract"]["design_inputs"] = [
            {"path": "SKILL.md", "sha256": "deadbeef"}
        ]
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [
            {"path": str(self.repo_a / "src/a.py"), "sha256": "deadbeef"}
        ]
        profile_path = self.root / "profile-absolute.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(
            str(profile_path), require_injection=False, validate_design=False
        )
        report = agy_dispatch.project_policy_report(loaded)
        self.assertFalse(report["dispatch_ready"])
        self.assertTrue(
            any("src/a.py" in blocker for blocker in report["blockers"]),
            report["blockers"],
        )

    def test_loaded_profile_frozen_path_not_in_allowed_writes_is_fine(
        self,
    ) -> None:
        """A profile whose frozen set names a path the round does not write,
        read back with load_profile, is dispatch_ready (negative control)."""
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["mode"] = "bounded-write"
        profile["task_contract"]["kind"] = "implementation"
        profile["task_contract"]["gate_command"] = "pwd"
        profile["task_contract"]["design_inputs"] = [
            {"path": "SKILL.md", "sha256": "deadbeef"}
        ]
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [{"path": "src/b.py", "sha256": "deadbeef"}]
        profile_path = self.root / "profile-neg.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(
            str(profile_path), require_injection=False, validate_design=False
        )
        report = agy_dispatch.project_policy_report(loaded)
        self.assertTrue(report["dispatch_ready"], report["blockers"])
        self.assertFalse(
            any("src/a.py" in blocker or "src/b.py" in blocker for blocker in report["blockers"]),
            report["blockers"],
        )

    def test_loaded_profile_frozen_namesake_in_another_directory_is_fine(
        self,
    ) -> None:
        """The negative control that the two-directory case needs. Resolving
        both sides is not the only way to make the same-path case fire: comparing
        basenames does too, and it also blocks every profile that freezes a
        `mod.rs` while writing a different `mod.rs`, which is most of them. A
        control keyed on distinct filenames cannot tell the two apart, so this
        one keeps the filename and moves the directory (#3439)."""
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["mode"] = "bounded-write"
        profile["task_contract"]["kind"] = "implementation"
        profile["task_contract"]["gate_command"] = "pwd"
        profile["task_contract"]["design_inputs"] = [
            {"path": "SKILL.md", "sha256": "deadbeef"}
        ]
        profile["allowed_repo_writes"] = ["src/cli/a.py"]
        profile["protected_artifacts"] = [
            {"path": "src/lib/a.py", "sha256": "deadbeef"}
        ]
        profile_path = self.root / "profile-namesake.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(
            str(profile_path), require_injection=False, validate_design=False
        )
        report = agy_dispatch.project_policy_report(loaded)
        self.assertTrue(report["dispatch_ready"], report["blockers"])
        self.assertFalse(
            any("a.py" in blocker for blocker in report["blockers"]),
            report["blockers"],
        )

    def test_doctor_exits_non_zero_and_reports_blocker_for_loaded_profile(
        self,
    ) -> None:
        """doctor's exit status and emitted report for a profile loaded from disk
        with a frozen write target: exits with 2 and includes the blocker."""
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["mode"] = "bounded-write"
        profile["task_contract"]["kind"] = "implementation"
        profile["task_contract"]["gate_command"] = "pwd"
        profile["task_contract"]["design_inputs"] = [
            {"path": "SKILL.md", "sha256": "deadbeef"}
        ]
        profile["allowed_repo_writes"] = ["src/a.py"]
        profile["protected_artifacts"] = [{"path": "src/a.py", "sha256": "deadbeef"}]
        profile_path = self.root / "profile-doctor.json"
        profile_path.write_text(json.dumps(profile))
        loaded = agy_dispatch.load_profile(
            str(profile_path), require_injection=False, validate_design=False
        )
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            with self.assertRaises(SystemExit) as caught:
                agy_dispatch.doctor(loaded)
        self.assertEqual(caught.exception.code, 2)
        output = buffer.getvalue()
        self.assertIn("src/a.py", output)
        self.assertIn("declared write target(s) are also frozen", output)

    def authored_round(self) -> tuple[dict, Path, Path]:
        """A round whose permission surface is ready and whose authoring is not.

        The gate is `pwd` rather than the conformant oracle's cargo line so that
        the round stays a *permission*-clean one: a gate the Project surface has
        no rule for is a blocker, and a blocker would exit `doctor` before it
        ever reached the question these rows ask.
        """
        profile = self.one_shot_profile(self.repo_a, "project-a")
        profile["task_contract"]["gate_command"] = "pwd"
        oracle = Path(profile["state_dir"]) / "oracles" / "adhoc-1.md"
        oracle.parent.mkdir(parents=True)
        snapshot = Path(profile["state_dir"]) / "snapshots" / "adhoc-1.json"
        return profile, oracle, snapshot

    def run_doctor(self, profile: dict) -> dict:
        with contextlib.redirect_stdout(io.StringIO()):
            return agy_dispatch.doctor(profile)

    def test_doctor_does_not_call_a_round_ready_before_its_snapshot_exists(
        self,
    ) -> None:
        """`dispatch` refuses without a snapshot, so `dispatch_ready` must too.

        The failure this replaces was silent and expensive: nothing is
        misconfigured, so the report printed `true`, and the round was authored
        and dispatched against a preflight that had said go.
        """
        profile, oracle, _ = self.authored_round()
        oracle.write_text(
            CONFORMANT_ORACLE.replace("cargo test -p target --lib some_gate", "pwd")
        )
        report = self.run_doctor(profile)
        self.assertFalse(report["dispatch_ready"], report)
        self.assertEqual(
            report["pending_steps"],
            ["no pre-dispatch snapshot: run `snapshot PROFILE adhoc-1`"],
        )
        # Not a blocker, and not an exit. A blocker is a misconfiguration the
        # controller must go and fix with `/permissions`; this is the flow
        # working, at exactly the step `doctor` is documented to run before.
        self.assertEqual(report["blockers"], [])

    def test_doctor_names_every_authoring_step_the_round_has_not_reached(
        self,
    ) -> None:
        """All three of `dispatch`'s authoring preconditions, each with its verb.

        A report that named only the last one reached would send the controller
        back once per step.
        """
        profile, oracle, snapshot = self.authored_round()
        self.assertEqual(
            self.run_doctor(profile)["pending_steps"],
            [
                "no oracle yet: run `scaffold PROFILE adhoc-1` and fill both "
                "documents",
                "no pre-dispatch snapshot: run `snapshot PROFILE adhoc-1`",
            ],
        )
        oracle.write_text("## Claim\n\n<!-- fill: what this round claims -->\n")
        snapshot.parent.mkdir(parents=True)
        snapshot.write_text("{}")
        pending = self.run_doctor(profile)["pending_steps"]
        self.assertEqual(len(pending), 1, pending)
        self.assertIn("run `lint PROFILE adhoc-1`", pending[0])

    def test_doctor_calls_the_round_ready_once_every_step_is_done(self) -> None:
        """The negative control: readiness still becomes true, and by doing them.

        Without this row the whole change is satisfied by a `dispatch_ready`
        that is simply always false.
        """
        profile, oracle, snapshot = self.authored_round()
        oracle.write_text(
            CONFORMANT_ORACLE.replace("cargo test -p target --lib some_gate", "pwd")
        )
        snapshot.parent.mkdir(parents=True)
        snapshot.write_text("{}")
        report = self.run_doctor(profile)
        self.assertEqual(report["pending_steps"], [])
        self.assertTrue(report["dispatch_ready"], report)

    def test_a_missing_snapshot_names_the_verb_that_creates_it(self) -> None:
        """The refusal is read by a controller who does not have the source open.

        `snapshot` takes the same two positional arguments as whatever refused,
        so the recovery is one line -- but only if the message says so.
        """
        profile, _, _ = self.authored_round()
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_snapshot(profile, "adhoc-1")
        message = str(caught.exception)
        self.assertIn("snapshot PROFILE adhoc-1", message)

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
            b'{"CommandLine":"rg -c NATIVE_FUNC_ADDRS\\\\.with apps/mamba",'
            b'"Cwd":"/repo","WaitMsBeforeAsync":5000}'
            b"\x00trailer"
        )
        self.assertEqual(
            agy_dispatch.extract_run_command_lines(payload),
            [r"rg -c NATIVE_FUNC_ADDRS\.with apps/mamba"],
        )

    def write_conversation(
        self,
        conversation_id: str,
        commands: list[tuple[int, str]],
        outcomes: list[tuple[int, int, str]] | None = None,
    ) -> None:
        """A conversation store shaped like AGY's.

        `commands` are request rows; `outcomes` are the result rows that say
        what became of them, and a request is joined to the outcome at the very
        next index, so callers space their own indices. Omitting an outcome is
        the honest fixture for a process that died mid-command, not a shortcut.
        """
        database = self.conversation_dir / f"{conversation_id}.db"
        connection = sqlite3.connect(database)

        def payload_for(command: str) -> bytes:
            return (
                b"prefix"
                + json.dumps({"CommandLine": command}, separators=(",", ":")).encode()
                + b"suffix"
            )

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
                connection.execute(
                    "insert into steps values (?, 15, 3, ?)",
                    (idx, payload_for(command)),
                )
            for idx, status, command in outcomes or []:
                connection.execute(
                    "insert into steps values (?, 21, ?, ?)",
                    (idx, status, payload_for(command)),
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
        audited, findings, _notes, every = agy_dispatch.audit_task_commands(
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
        self.assertEqual(findings, [])
        # The fourth value is every command the round saw, floor included: the
        # pre-snapshot line is absent here too, so a caller reading it to order
        # writes against runs sees the same window the audit did.
        self.assertEqual(
            [item["command"] for item in every],
            ["pwd", "rg -n TODO src"],
        )

    def test_a_command_only_the_project_surface_admits_is_a_note(self) -> None:
        """This used to void the round, and voiding it was the defect.

        `task_commands` is the controller's vocabulary and `project_permissions`
        is the worker's actual surface. A command the round's own Project rules
        allow but its allowlist forgot to name is a gap in the controller's
        writing-down, not in the worker's conduct, and #3427 reclassified it
        from fatal to a note for exactly that reason. What must not happen is
        that it goes unsaid: the note is how the controller learns to close the
        gap, so this asserts the note names the command and the round lives."""
        profile = self.profile(self.repo_a, "project-a", "8")
        state = Path(profile["state_dir"])
        runs = state / "runs"
        runs.mkdir(parents=True)
        (runs / "8.conversation").write_text("conversation-8\n")
        # `project_permissions` allows `command(rg)`; `task_commands.allow`
        # names only `rg -n TODO src`, which this is not.
        self.write_conversation(
            "conversation-8",
            [(1, "rg -n SECRET unrelated")],
        )
        audited, findings, notes, _every = agy_dispatch.audit_task_commands(
            profile,
            "8",
            {
                "conversation_id": None,
                "conversation_step_floor": -1,
            },
        )
        self.assertEqual(audited, [])
        self.assertEqual(findings, [])
        self.assertEqual(len(notes), 1)
        self.assertIn("rg -n SECRET unrelated", notes[0])
        self.assertIn("task_commands", notes[0])

    def test_a_denied_command_is_a_finding_not_a_void(self) -> None:
        """The VOID protects the evidence: an unaudited command may have left
        state in the tree that nobody can reconstruct. A denied request left
        none, so voiding the round discards a candidate to punish an intention
        -- and makes `denied`'s own remedy, tighten the prompt and resume,
        reachable only on a round that is already dead (#3427)."""
        profile = self.profile(self.repo_a, "project-a", "9")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "9.conversation").write_text("conversation-9\n")
        self.write_conversation(
            "conversation-9",
            [(1, "pwd"), (3, "rg -n SECRET unrelated")],
            outcomes=[(2, 3, "pwd"), (4, 7, "rg -n SECRET unrelated")],
        )
        audited, findings, _notes, _every = agy_dispatch.audit_task_commands(
            profile,
            "9",
            {"conversation_id": None, "conversation_step_floor": -1},
        )
        self.assertEqual([item["command"] for item in audited], ["pwd"])
        self.assertEqual(len(findings), 1)
        self.assertIn("rg -n SECRET unrelated", findings[0])
        self.assertIn("nothing ran", findings[0])

    def test_a_round_with_no_conversation_audits_to_the_same_shape(self) -> None:
        """Both exits from `audit_task_commands` return the same arity.

        The no-conversation branch is the one a fixture that builds a
        conversation can never reach, so it went out of step with the other exit
        and every caller unpacking the result died on a ValueError -- `verify`
        included, which is the verb that reads it. Callers are the whole point
        of the return shape, so the shape is what this asserts."""
        profile = self.profile(self.repo_a, "project-a", "42")
        result = agy_dispatch.audit_task_commands(
            profile,
            "42",
            {"conversation_id": None, "conversation_step_floor": -1},
        )
        self.assertEqual(len(result), 4)
        self.assertEqual(list(result), [[], [], [], []])

    def test_an_unlisted_command_that_ran_still_voids(self) -> None:
        """The other half of the pair. Same request, same allowlist; the only
        difference is that the permission layer let this one through, and that
        difference is the whole reason the VOID exists.

        The command is `curl`, not `rg`, because this fixture's
        `project_permissions` allow `command(rg)` -- so an `rg` line is one the
        round itself authorized and #3427 correctly demoted to a note. Keeping
        `rg` here would have made the test assert the demotion it was written to
        rule out, and for a while it silently did."""
        profile = self.profile(self.repo_a, "project-a", "10")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "10.conversation").write_text("conversation-10\n")
        self.write_conversation(
            "conversation-10",
            [(1, "curl https://example.test")],
            outcomes=[(2, 3, "curl https://example.test")],
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.audit_task_commands(
                profile,
                "10",
                {"conversation_id": None, "conversation_step_floor": -1},
            )
        self.assertIn("nothing authorized", str(caught.exception))
        self.assertIn("curl https://example.test", str(caught.exception))

    def test_a_command_with_no_outcome_row_is_read_as_having_run(self) -> None:
        """A process killed mid-command leaves the request recorded and the
        outcome never written. Reading that absence as a denial would let a
        crash launder an escape, so an unknown outcome takes the fatal branch."""
        profile = self.profile(self.repo_a, "project-a", "11")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "11.conversation").write_text("conversation-11\n")
        self.write_conversation(
            "conversation-11",
            [(1, "curl https://example.test")],
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.audit_task_commands(
                profile,
                "11",
                {"conversation_id": None, "conversation_step_floor": -1},
            )
        self.assertIn("nothing authorized", str(caught.exception))
        self.assertIn("curl https://example.test", str(caught.exception))

    def test_a_later_denial_does_not_excuse_an_earlier_request(self) -> None:
        """The outcome is the row immediately after the request, not the next
        denial anywhere ahead of it. Scanning forward would let one refused
        command launder every escape that preceded it, which is the exact shape
        of a run that died with its own outcome row unwritten."""
        profile = self.profile(self.repo_a, "project-a", "13")
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "13.conversation").write_text("conversation-13\n")
        self.write_conversation(
            "conversation-13",
            [(1, "curl https://example.test"), (2, "wget https://example.test")],
            outcomes=[(3, 7, "wget https://example.test")],
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.audit_task_commands(
                profile,
                "13",
                {"conversation_id": None, "conversation_step_floor": -1},
            )
        self.assertIn("curl https://example.test", str(caught.exception))
        self.assertNotIn("wget https://example.test", str(caught.exception))

    def test_a_denied_command_the_profile_forbids_by_name_still_voids(self) -> None:
        """Asking for a command the profile listed as forbidden is a different
        act from asking for one it merely did not list: the round named that
        command as out of bounds, so the request is the finding regardless of
        who stopped it."""
        profile = self.profile(self.repo_a, "project-a", "12")
        profile["task_commands"]["deny"] = ["git push"]
        runs = Path(profile["state_dir"]) / "runs"
        runs.mkdir(parents=True)
        (runs / "12.conversation").write_text("conversation-12\n")
        self.write_conversation(
            "conversation-12",
            [(1, "git push")],
            outcomes=[(2, 7, "git push")],
        )
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.audit_task_commands(
                profile,
                "12",
                {"conversation_id": None, "conversation_step_floor": -1},
            )
        self.assertIn("git push", str(caught.exception))

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

    def round_documents(self, issue: str) -> tuple[dict, Path, Path]:
        """A profile whose oracle and injection both exist on disk."""
        profile = self.profile(self.repo_a, "project-a", issue)
        state = Path(profile["state_dir"])
        oracle = state / "oracles" / f"{issue}.md"
        injection = state / "injections" / f"{issue}.md"
        oracle.parent.mkdir(parents=True, exist_ok=True)
        injection.parent.mkdir(parents=True, exist_ok=True)
        oracle.write_text("## Claim\n\nthe judge\n")
        injection.write_text("## Task\n\nthe delta contract\n")
        profile["inject_prompt_file"] = str(injection)
        return profile, oracle, injection

    def test_edited_oracle_after_snapshot_is_void(self) -> None:
        """The oracle exists to be the judge the controller cannot retro-fit.

        `verify` used to print its sha256 without ever comparing it, which
        reads like a freeze and is not one.
        """
        profile, oracle, _ = self.round_documents("40")
        frozen = {"round_documents": agy_dispatch.round_document_digests(profile, "40")}

        self.assertTrue(
            agy_dispatch.assert_round_documents_unchanged(profile, "40", frozen)
        )

        oracle.write_text("## Claim\n\nthe judge, softened\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.assert_round_documents_unchanged(profile, "40", frozen)
        self.assertIn("oracle changed after snapshot", str(caught.exception))

    def test_edited_injection_after_snapshot_is_void(self) -> None:
        profile, _, injection = self.round_documents("41")
        frozen = {"round_documents": agy_dispatch.round_document_digests(profile, "41")}

        injection.write_text("## Task\n\na different delta contract\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.assert_round_documents_unchanged(profile, "41", frozen)
        self.assertIn("injection changed after snapshot", str(caught.exception))

    def test_injection_appearing_after_snapshot_is_void(self) -> None:
        """A round that grew a second document mid-flight is not the round
        that was snapshotted, so an absent document has to stay absent."""
        profile = self.profile(self.repo_a, "project-a", "42")
        state = Path(profile["state_dir"])
        (state / "oracles").mkdir(parents=True, exist_ok=True)
        (state / "oracles" / "42.md").write_text("## Claim\n\nthe judge\n")
        frozen = {"round_documents": agy_dispatch.round_document_digests(profile, "42")}
        self.assertIsNone(frozen["round_documents"]["injection"])

        injection = state / "injections" / "42.md"
        injection.parent.mkdir(parents=True, exist_ok=True)
        injection.write_text("## Task\n\nsmuggled in after the freeze\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.assert_round_documents_unchanged(profile, "42", frozen)
        self.assertIn("injection changed after snapshot", str(caught.exception))

    def test_pre_freeze_snapshot_reports_that_it_was_not_compared(self) -> None:
        """A snapshot taken before this check must not silently read as a
        match -- 'unchanged' and 'never compared' are different facts."""
        profile, _, _ = self.round_documents("44")
        self.assertFalse(
            agy_dispatch.assert_round_documents_unchanged(profile, "44", {})
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
        # A whole fixture AGY installation, in AGY's own layout, reachable by
        # both routes. The globals serve callers in this process; the
        # environment variable serves the ones in a child, which is the only
        # way `make_profile.py` -- a subprocess that imports agy_dispatch --
        # can be pointed anywhere but the operator's real `~/.gemini` (#3495).
        self.agy_home = self.root / "agy-home"
        self.project_dir = self.agy_home / ".gemini" / "config" / "projects"
        self.conversation_dir = (
            self.agy_home / ".gemini" / "antigravity-cli" / "conversations"
        )
        self.settings = self.agy_home / ".gemini" / "antigravity-cli" / "settings.json"
        self.global_config = self.agy_home / ".gemini" / "config" / "config.json"
        self.project_dir.mkdir(parents=True)
        self.conversation_dir.mkdir(parents=True)
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
        os.environ["AGY_DISPATCH_HOME"] = str(self.agy_home)
        self.addCleanup(os.environ.pop, "AGY_DISPATCH_HOME", None)
        agy_dispatch.PROJECT_DIR = self.project_dir
        agy_dispatch.CONVERSATION_DIR = self.conversation_dir
        agy_dispatch.SETTINGS = self.settings
        agy_dispatch.GLOBAL = self.global_config
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
                            # The live surface a round starts from is the one
                            # `grant` installed, so it carries the gate too.
                            "allow": [
                                "command(pwd)",
                                "command(grep -q accepted README.md)",
                            ],
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
            # Shaped like `make_profile.py --gate` output: the round's gate
            # appears in all three places, so the worker is authorized to run
            # the one command it is judged by.
            "project_permissions": {
                "allow": ["command(pwd)", "command(grep -q accepted README.md)"],
                "deny": ["command(git push)"],
                "ask": [],
                "require_empty_global": True,
            },
            "task_commands": {
                "allow": ["grep -q accepted README.md"],
                "deny": [],
            },
            "protected_artifacts": [],
            "snapshot_paths": [],
            "allowed_repo_writes": ["README.md"],
            "path_change_budgets": {},
            **overrides,
        }
        path = self.root / "profile.json"
        path.write_text(json.dumps(profile, indent=2))
        return path

    def isolate_permission_files(self) -> None:
        """Point the readiness check at empty surfaces.

        Otherwise it reads the real user's inherited grants and the test's
        result depends on whoever ran it.
        """
        settings = self.root / "settings.json"
        global_config = self.root / "config.json"
        settings.write_text(
            json.dumps({"permissions": {"allow": [], "deny": [], "ask": []}})
        )
        global_config.write_text(
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
        previous = (agy_dispatch.SETTINGS, agy_dispatch.GLOBAL)
        agy_dispatch.SETTINGS = settings
        agy_dispatch.GLOBAL = global_config
        self.addCleanup(
            lambda: setattr(agy_dispatch, "SETTINGS", previous[0])
            or setattr(agy_dispatch, "GLOBAL", previous[1])
        )

    def snapshotted_round(self) -> tuple[dict, Path, Path]:
        """A round whose two documents exist and have been snapshotted."""
        self.isolate_permission_files()
        state = self.root / "state"
        oracle = state / "oracles" / "round-1.md"
        injection = state / "injections" / "round-1.md"
        oracle.parent.mkdir(parents=True, exist_ok=True)
        injection.parent.mkdir(parents=True, exist_ok=True)
        oracle.write_text("## Claim\n\nthe judge\n")
        injection.write_text("## Task\n\nthe delta contract\n")
        profile = agy_dispatch.load_profile(
            str(
                self.profile_path(
                    task_contract=self.contract_with_design_input(),
                    inject_prompt_file=str(injection),
                )
            )
        )
        agy_dispatch.snapshot(profile, "round-1")
        return profile, oracle, injection

    def frozen_write_target_round(self) -> dict:
        """A profile whose one write target is also frozen against writing.

        This is the derived-profile shape #3428 was filed on: the write set was
        edited for the new round and the frozen complement still describes the
        one it came from.
        """
        self.isolate_permission_files()
        readme = self.controller / "README.md"
        return agy_dispatch.load_profile(
            str(
                self.profile_path(
                    task_contract=self.contract_with_design_input(),
                    protected_artifacts=[
                        {"path": "README.md", "sha256": agy_dispatch.sha256(readme)}
                    ],
                )
            ),
            require_injection=False,
        )

    def test_snapshot_refuses_to_freeze_a_contradictory_profile(self) -> None:
        """`doctor` refusing is not enough on its own.

        `doctor` is advisory -- a controller reads it and decides. `snapshot` is
        the verb that writes the contradiction into the round's evidence, after
        which every later verb compares against a frozen claim that the round
        was told to violate. #3428's sequence is "doctor passes, snapshot
        freezes the contradiction, dispatch runs", so each of the three needs
        its own assertion; a suite that tests only `doctor` stays green while
        the guard is lifted off the other two.
        """
        profile = self.frozen_write_target_round()
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.snapshot(profile, "round-1")
        message = str(caught.exception)
        self.assertIn("declared write target(s) are also frozen", message)
        self.assertIn("README.md", message)
        self.assertFalse(
            (Path(profile["state_dir"]) / "snapshots" / "round-1.json").exists(),
            "refused and wrote the snapshot anyway",
        )

    def test_dispatch_refuses_to_start_a_contradictory_round(self) -> None:
        """The other of the two, and the expensive one.

        A worker spawned under this profile burns a run id to reach a finding
        that was visible in the profile before anything started, so the refusal
        has to land before the subprocess. The stub makes "reached the spawn" a
        distinguishable failure instead of a real run.
        """
        profile = self.frozen_write_target_round()

        def refuse(*args: object, **kwargs: object) -> None:
            raise AssertionError("spawned a worker under a contradictory profile")

        previous = agy_dispatch.subprocess.run
        agy_dispatch.subprocess.run = refuse
        self.addCleanup(
            lambda: setattr(agy_dispatch.subprocess, "run", previous)
        )

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.run_agent(profile, "round-1", resume=False)
        self.assertIn(
            "declared write target(s) are also frozen", str(caught.exception)
        )

    def test_snapshot_freezes_both_round_documents(self) -> None:
        """The digests have to reach the snapshot file, not just exist as a
        function. A freeze wired to nothing passes every test written against
        the helper alone, which is the shape of the unverified `oracle sha256=`
        line this replaces.
        """
        profile, oracle, injection = self.snapshotted_round()
        recorded = json.loads(
            (Path(profile["state_dir"]) / "snapshots" / "round-1.json").read_text()
        )["round_documents"]
        self.assertEqual(recorded["oracle"], agy_dispatch.sha256(oracle))
        self.assertEqual(recorded["injection"], agy_dispatch.sha256(injection))

    def test_verify_voids_on_an_oracle_edited_after_snapshot(self) -> None:
        """The freeze has to be reachable from the verb the controller runs.

        Proving `assert_round_documents_unchanged` in isolation says nothing
        about whether `verify` calls it, and an unwired check is exactly the
        defect being fixed.
        """
        profile, oracle, _ = self.snapshotted_round()
        agy_dispatch.verify(profile, "round-1")

        oracle.write_text("## Claim\n\nthe judge, softened to fit the answer\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.verify(profile, "round-1")
        self.assertIn("VOID: oracle changed after snapshot", str(caught.exception))

    def test_dispatch_refuses_to_start_with_a_drifted_oracle(self) -> None:
        """Catching the drift at `verify` is already too late.

        The window between `snapshot` and `dispatch` is the one where an edit
        actually changes what the worker is asked, so the check has to fire
        before AGY is ever spawned. The stub turns "reached the subprocess"
        into a distinguishable failure instead of a real 45-minute run.
        """
        profile, oracle, _ = self.snapshotted_round()
        oracle.write_text("## Claim\n\nedited between snapshot and dispatch\n")

        def refuse(*args: object, **kwargs: object) -> None:
            raise AssertionError("spawned a worker with a drifted oracle")

        previous = agy_dispatch.subprocess.run
        agy_dispatch.subprocess.run = refuse
        self.addCleanup(
            lambda: setattr(agy_dispatch.subprocess, "run", previous)
        )

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.run_agent(profile, "round-1", resume=False)
        self.assertIn("VOID: oracle changed after snapshot", str(caught.exception))

    def test_verify_voids_on_an_injection_edited_after_snapshot(self) -> None:
        profile, _, injection = self.snapshotted_round()
        injection.write_text("## Task\n\na contract the worker never saw\n")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.verify(profile, "round-1")
        self.assertIn("VOID: injection changed after snapshot", str(caught.exception))

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

    def test_grant_refuses_a_gate_the_declared_surface_would_still_ask_for(
        self,
    ) -> None:
        """A round's profile is derived from the previous round's and the gate
        command is what changes; the hand-maintained grant list is what gets
        left behind."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        loaded = json.loads(path.read_text())
        loaded["task_commands"]["allow"].append("cargo test --lib this_rounds_gate")
        path.write_text(json.dumps(loaded))
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn("cargo test --lib this_rounds_gate", str(caught.exception))
        self.assertIn("do not cover", str(caught.exception))

    def test_grant_reports_nothing_to_change_only_when_the_gate_is_covered(
        self,
    ) -> None:
        """The stale-grant bug reached the worker through the early return, so
        the coverage check has to run before it."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        loaded = json.loads(path.read_text())
        loaded["task_commands"]["allow"].append("cargo test --lib this_rounds_gate")
        loaded["project_permissions"]["allow"].append(
            "command(cargo test --lib this_rounds_gate)"
        )
        path.write_text(json.dumps(loaded))
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn(
            "command(cargo test --lib this_rounds_gate)",
            self.project_grants()["allow"],
        )

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

    def project_document_text(self, project_id: str = "project-a") -> str:
        return (self.project_dir / f"{project_id}.json").read_text()

    def unfilled_surface(self, **patch: object) -> Path:
        """A derived round whose profile is then patched and written back."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        loaded = json.loads(path.read_text())
        for key, value in patch.items():
            section, _, field = key.partition("__")
            loaded[section][field] = value
        path.write_text(json.dumps(loaded))
        return path

    def test_grant_refuses_a_surface_that_authorizes_no_command(self) -> None:
        """`make_profile.py` emitted `"allow": []` and `grant` installs the
        declared surface verbatim, so an unfilled profile revoked every
        permission the Project held — and every check downstream then iterated
        an empty list and found nothing wrong."""
        path = self.unfilled_surface(project_permissions__allow=[])
        before = self.project_document_text()
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn("authorizes no command at all", str(caught.exception))
        self.assertIn('"no_shell": true', str(caught.exception))
        self.assertEqual(
            self.project_document_text(),
            before,
            "a refused grant must leave the live Project byte-identical",
        )

    def test_grant_installs_the_empty_surface_a_round_declares_it_wants(
        self,
    ) -> None:
        """A measure-only round authorizes nothing on purpose (negative
        control). Without this the refusal would ban a legitimate shape rather
        than catch a profile nobody filled in."""
        path = self.unfilled_surface(
            project_permissions__allow=[],
            project_permissions__no_shell=True,
            task_commands__allow=[],
        )
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertEqual(self.project_grants()["allow"], [])

    def test_grant_refuses_when_the_worker_cannot_run_its_own_gate(self) -> None:
        """The gate is the one command every round has and the one field a
        derived profile always changes, so it is the one most often missing
        from the hand-maintained allowlist."""
        path = self.unfilled_surface(task_commands__allow=["pwd"])
        before = self.project_document_text()
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        self.assertIn("cannot run what it is judged by", str(caught.exception))
        self.assertIn("grep -q accepted README.md", str(caught.exception))
        self.assertEqual(
            self.project_document_text(),
            before,
            "a refused grant must leave the live Project byte-identical",
        )

    def test_doctor_blocks_on_a_gate_the_worker_cannot_run(self) -> None:
        """`grant` is not the only route to a live surface, so the preflight
        has to see the same omission. Reading ready-then-blocked off one
        profile keeps the blocker attributable to the gate."""
        self.isolate_permission_files()
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        ready = agy_dispatch.project_policy_report(
            agy_dispatch.load_profile(str(path), validate_design=False)
        )
        self.assertTrue(ready["dispatch_ready"], ready["blockers"])

        loaded = json.loads(path.read_text())
        loaded["task_commands"]["allow"] = ["pwd"]
        path.write_text(json.dumps(loaded))
        blocked = agy_dispatch.project_policy_report(
            agy_dispatch.load_profile(str(path), validate_design=False)
        )
        self.assertFalse(blocked["dispatch_ready"], blocked["blockers"])
        self.assertTrue(
            any(
                "grep -q accepted README.md" in blocker
                for blocker in blocked["blockers"]
            ),
            blocked["blockers"],
        )

    def test_a_narrowing_installs_and_is_printed_as_a_revocation(self) -> None:
        """Removals used to print as `- allow command(sed)`, the same shape as
        an addition. Narrowing stays legal; it has to announce itself, so the
        heading has to be there and has to come first."""
        path = self.profile_path(task_contract=self.contract_with_design_input())
        self.derive(path)
        self.widen_grants("command(cargo publish)")
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(path), validate_design=False)
            )
        printed = out.getvalue()
        self.assertIn("revocations (1)", printed)
        self.assertIn("- allow command(cargo publish)", printed)
        self.assertLess(
            printed.index("revocations (1)"),
            printed.index("- allow command(cargo publish)"),
            printed,
        )
        self.assertNotIn("command(cargo publish)", self.project_grants()["allow"])

    def seed_generated_round(self) -> None:
        """The tree a generated profile is generated against.

        Idempotent, because the two invocations are compared against each other
        inside one test and `git commit` with nothing staged is an error.
        """
        if (self.controller / "design.md").exists():
            return
        (self.controller / "design.md").write_text("frozen design\n")
        source = self.controller / "src"
        source.mkdir(exist_ok=True)
        (source / "a.py").write_text("x = 1\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "design")

    def make_profile(self, *extra: str) -> subprocess.CompletedProcess[str]:
        self.seed_generated_round()
        return subprocess.run(
            [
                sys.executable,
                str(MAKE_PROFILE),
                "--root", str(self.controller),
                "--repo", "owner/repo",
                "--project-id", "project-a",
                "--scope", "src",
                "--run-id", "generated-1",
                "--intent", "bounded change",
                "--design-input", "design.md",
                "--write", "src/a.py",
                "--out", str(self.root / "generated.json"),
                *extra,
            ],
            capture_output=True,
            text=True,
        )

    def make_profile_derived(
        self,
        *extra: str,
        origin: str = "git@github.com:owner/repo.git",
        cwd: Path | None = None,
    ) -> subprocess.CompletedProcess[str]:
        """The same round, naming only what describes it.

        Run from inside the checkout with no `--root`, `--repo`, `--project-id`
        or `--out`, which is the invocation #3432 asks for.
        """
        self.seed_generated_round()
        if origin and not self.git("remote").strip():
            self.git("remote", "add", "origin", origin)
        return subprocess.run(
            [
                sys.executable,
                str(MAKE_PROFILE),
                "--scope", "src",
                "--run-id", "generated-1",
                "--intent", "bounded change",
                "--design-input", "design.md",
                "--write", "src/a.py",
                "--gate", "cargo test -p target --lib some_gate",
                *extra,
            ],
            capture_output=True,
            text=True,
            cwd=str(cwd or self.controller),
        )

    def test_make_profile_derives_every_input_the_repository_already_knows(
        self,
    ) -> None:
        """Twelve flags produced a profile five of them described.

        The four dropped here are not conveniences equally: `--out` and
        `--inject` fail loudly when mistyped, at `worktree` and at `lint`, but
        `--project-id` names a real Project whatever the typo, passes every
        check, and runs the round against the wrong work area. The assertion is
        equality with the fully-flagged profile rather than a spot check on each
        field, because a derivation that is right about four inputs and wrong
        about the fifth is the failure being removed (#3432).
        """
        derived = self.make_profile_derived()
        self.assertEqual(derived.returncode, 0, derived.stderr)
        emitted = json.loads(self.derived_out().read_text())

        self.assertEqual(
            emitted["controller_root"], str(self.controller.resolve())
        )
        self.assertEqual(emitted["repo"], "owner/repo")
        self.assertEqual(emitted["agy_project_id"], "project-a")
        self.assertEqual(emitted["state_dir"], "/tmp/agy-dispatch/project-a")

        flagged = self.make_profile("--gate", "cargo test -p target --lib some_gate")
        self.assertEqual(flagged.returncode, 0, flagged.stderr)
        self.assertEqual(
            emitted, json.loads((self.root / "generated.json").read_text())
        )

    def derived_out(self) -> Path:
        """Where a profile lands when `--out` is not given.

        `revise` already writes its successor to
        `{state_dir}/rounds/{key}.profile.json`, so this is the default that
        makes a generated profile and a derived one the same kind of file.
        """
        path = (
            Path("/tmp/agy-dispatch/project-a")
            / "rounds"
            / "generated-1.profile.json"
        )
        self.addCleanup(path.unlink, missing_ok=True)
        self.assertTrue(path.exists(), path)
        return path

    def test_make_profile_derives_repo_from_an_https_origin_too(self) -> None:
        """git writes the remote in two spellings and the round does not choose
        which one the checkout was cloned with."""
        derived = self.make_profile_derived(
            origin="https://github.com/owner/repo.git"
        )
        self.assertEqual(derived.returncode, 0, derived.stderr)
        self.assertEqual(
            json.loads(self.derived_out().read_text())["repo"], "owner/repo"
        )

    def test_a_generated_profile_briefs_the_worker_it_dispatches(self) -> None:
        """The omission with no red anywhere.

        `scaffold` writes `injections/{task_key}.md`, and `render_prompt` reads
        `inject_prompt_file` rather than that convention -- so a profile that did
        not name one dispatched the worker the oracle and no delta contract.
        `scaffold` prints a note about it; `lint` reports the injection green,
        because `lint` reads the file and the wiring is what is broken. Nothing
        in the round goes red, and the worker is simply never told what to do.

        Asserting the field would be a weaker test than this: the property is
        that the text reaches the prompt, and the field is only how it gets
        there today.
        """
        derived = self.make_profile_derived()
        self.assertEqual(derived.returncode, 0, derived.stderr)
        profile = json.loads(self.derived_out().read_text())

        # `scaffold` writes the injection, so `scaffold` is what decides where it
        # lives. Naming the file here instead would make any consistent pair of
        # wrong paths pass.
        agy_dispatch.scaffold(profile, "generated-1")
        injection = agy_dispatch.injection_path(profile, "generated-1")
        self.addCleanup(injection.unlink, missing_ok=True)
        self.addCleanup(
            (Path(profile["state_dir"]) / "oracles" / "generated-1.md").unlink,
            missing_ok=True,
        )
        injection.write_text("## Task\n\nthe delta contract\n")

        prompt = agy_dispatch.render_prompt(
            profile, "generated-1", "oracle", {"number": 0, "state": "OPEN"}
        )
        self.assertIn("the delta contract", prompt)

    def test_two_rounds_in_one_state_dir_get_their_own_injection(self) -> None:
        """Keyed by the task key, for the same reason `revise` keys its own.

        A fixed filename briefs every round correctly in isolation and hands the
        second round of a `revise` chain the first round's delta contract, which
        is the failure that looks most like the worker ignoring instructions.
        """
        first = self.make_profile_derived()
        self.assertEqual(first.returncode, 0, first.stderr)
        second = self.make_profile_derived("--run-id", "generated-2")
        self.assertEqual(second.returncode, 0, second.stderr)

        rounds = Path("/tmp/agy-dispatch/project-a") / "rounds"
        paths = []
        for key in ("generated-1", "generated-2"):
            emitted = rounds / f"{key}.profile.json"
            self.addCleanup(emitted.unlink, missing_ok=True)
            self.assertTrue(emitted.exists(), emitted)
            paths.append(json.loads(emitted.read_text())["inject_prompt_file"])
        self.assertNotEqual(*paths)

    def test_a_generated_profile_declares_the_surface_it_will_run_under(
        self,
    ) -> None:
        """The declared surface has to equal the effective one, not a subset.

        `doctor` blocks on inherited `allow` rules the profile does not declare,
        and it is right to: they widen the worker past what the round says it
        authorized, so `verify` would measure the round against a surface it did
        not run under. The Project's `deny` rules were already carried for the
        mirror-image reason; this is the other half (#3431).

        Transcribing them by hand is what produced the gap. The list lives on the
        machine, so a copy in the profile goes stale the first time the machine
        is edited, and the omission is a strict narrowing -- which reads exactly
        like a deliberate one.
        """
        self.settings.write_text(
            json.dumps(
                {
                    "permissions": {
                        "allow": ["command(rg)", "command(uv)"],
                        "deny": [],
                        "ask": [],
                    }
                }
            )
        )
        derived = self.make_profile_derived()
        self.assertEqual(derived.returncode, 0, derived.stderr)
        profile = json.loads(self.derived_out().read_text())

        allow = profile["project_permissions"]["allow"]
        self.assertIn("command(rg)", allow)
        self.assertIn("command(uv)", allow)
        self.assertIn("command(cargo test -p target --lib some_gate)", allow)

        report = agy_dispatch.project_policy_report(profile)
        self.assertFalse(
            [b for b in report["blockers"] if "inherited global rules" in b],
            report["blockers"],
        )

    def test_a_shell_less_round_does_not_claim_to_be_one_while_authorized(
        self,
    ) -> None:
        """`no_shell` is an exemption, so it cannot be emitted on reflex.

        It turns off the refusal that a round must authorize its own gate. A
        gate-less round on a machine whose global surface already allows
        thirteen commands is not shell-less -- the worker can run all thirteen --
        and saying so would hand back the escape hatch #3479 closed.
        """
        self.settings.write_text(
            json.dumps(
                {
                    "permissions": {
                        "allow": ["command(rg)"],
                        "deny": [],
                        "ask": [],
                    }
                }
            )
        )
        # Measure-only and gate-less, which is the only shape that reaches the
        # branch: `make_profile.py` refuses a bounded-write round with no gate.
        self.seed_generated_round()
        self.git("remote", "add", "origin", "git@github.com:owner/repo.git")
        derived = subprocess.run(
            [
                sys.executable,
                str(MAKE_PROFILE),
                "--scope", "src",
                "--run-id", "measure-1",
                "--intent", "measure without running anything",
            ],
            capture_output=True,
            text=True,
            cwd=str(self.controller),
        )
        self.assertEqual(derived.returncode, 0, derived.stderr)
        emitted = (
            Path("/tmp/agy-dispatch/project-a") / "rounds"
            / "measure-1.profile.json"
        )
        self.addCleanup(emitted.unlink, missing_ok=True)
        permissions = json.loads(emitted.read_text())["project_permissions"]
        self.assertNotIn("no_shell", permissions)

    def test_make_profile_derives_the_repository_root_not_the_directory(
        self,
    ) -> None:
        """The invocation is `--scope libs/service-auth` from wherever the
        controller happens to be standing.

        `Path.cwd()` passes every test run from the top of the checkout and is
        wrong for every other one: the profile would freeze the complement of a
        subdirectory, and `worktree` would derive the round's checkout from a
        path that is not a repository.
        """
        derived = self.make_profile_derived(cwd=self.controller / "src")
        self.assertEqual(derived.returncode, 0, derived.stderr)
        self.assertEqual(
            json.loads(self.derived_out().read_text())["controller_root"],
            str(self.controller.resolve()),
        )

    def test_make_profile_refuses_to_pick_between_two_projects_on_one_root(
        self,
    ) -> None:
        """Choosing the first match silently is worse than the typo it replaces.

        Two Projects bound to one root is what a round pile-up looks like, and
        picking either one runs this round under a permission surface some other
        round installed.
        """
        self.write_project("project-b", self.controller)
        derived = self.make_profile_derived()
        self.assertEqual(derived.returncode, 2, derived.stdout)
        self.assertIn("project-a", derived.stderr)
        self.assertIn("project-b", derived.stderr)

    def test_make_profile_refuses_rather_than_guessing_a_repo(self) -> None:
        """A derivation that has nothing to read has to say so.

        Falling back to a placeholder would put a plausible `owner/name` in the
        profile, and the round would only find out at the point something tries
        to reach the tracker.
        """
        derived = self.make_profile_derived(origin="")
        self.assertEqual(derived.returncode, 2, derived.stdout)
        self.assertIn("origin", derived.stderr)

    def test_make_profile_reads_an_unmatched_project_as_an_undiscarded_round(
        self,
    ) -> None:
        """The lookup comes back empty in one situation, and the message has to
        name it.

        `worktree` rebinds the Project to the round's checkout, so a round that
        was never discarded leaves the binding pointing at a worktree instead of
        the controller root. Reported as "pass --project-id" that reads as a
        missing flag, and the controller supplies the id by hand -- which is how
        a second round gets dispatched into the first round's tree.
        """
        stale = self.root / "some-other-round"
        stale.mkdir()
        self.write_project("project-a", stale)
        derived = self.make_profile_derived()
        self.assertEqual(derived.returncode, 2, derived.stdout)
        self.assertIn("never discarded", derived.stderr)
        self.assertIn("--project-id", derived.stderr)

    def test_make_profile_output_is_grantable_without_hand_editing(self) -> None:
        """The empty surface came from the generator, not from a user, so the
        generator's own output is what has to survive the new refusals — and
        surviving them means reaching the live Project, not just satisfying the
        predicate in isolation."""
        gate = "cargo test -p target --lib some_gate"
        result = self.make_profile("--gate", gate)
        self.assertEqual(result.returncode, 0, result.stderr)
        generated = self.root / "generated.json"
        profile = json.loads(generated.read_text())
        self.assertEqual(profile["task_contract"]["gate_command"], gate)
        self.assertIn(gate, profile["task_commands"]["allow"])
        self.assertIn(f"command({gate})", profile["project_permissions"]["allow"])

        agy_dispatch.worktree(str(generated), "generated-1")
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.grant(
                agy_dispatch.load_profile(str(generated), require_injection=False)
            )
        self.assertIn(f"command({gate})", self.project_grants()["allow"])

    def test_make_profile_refuses_a_bounded_write_round_without_a_gate(
        self,
    ) -> None:
        """Emitting a gateless bounded-write profile is what left the gate out
        of the allowlist in the first place."""
        result = self.make_profile()
        self.assertEqual(result.returncode, 2, result.stdout)
        self.assertIn("needs --gate", result.stderr)
        self.assertFalse((self.root / "generated.json").exists())

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

    def test_a_regenerated_profile_still_records_the_controller_as_home(
        self,
    ) -> None:
        """The reported route in, and the only one that reaches the fallback.

        The home root is read from the binding a moment before it moves, which
        is the controller on a first run and this very worktree on a second. A
        profile that has been through `worktree` carries the answer in its own
        block; a regenerated one does not -- and SKILL.md prescribes exactly
        that regeneration when a round's write set has to widen.
        """
        path = self.profile_path()
        self.derive(path)
        regenerated = self.profile_path()
        profile = self.derive(regenerated)
        self.assertEqual(
            Path(profile["worktree"]["project_home_root"]).resolve(),
            self.controller.resolve(),
        )
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.discard(str(regenerated), "round-1")
        self.assertEqual(self.project_binding(), self.controller.resolve())

    def test_a_grafted_home_root_that_is_the_worktree_is_repaired(self) -> None:
        """Grafting the spent round's `worktree` block is the documented route.

        So a value this defect already wrote propagates the same way, and the
        block a controller grafts is not one they authored.
        """
        path = self.profile_path()
        profile = self.derive(path)
        profile["worktree"]["project_home_root"] = profile["worktree"]["path"]
        path.write_text(json.dumps(profile, indent=2))
        profile = self.derive(path)
        self.assertEqual(
            Path(profile["worktree"]["project_home_root"]).resolve(),
            self.controller.resolve(),
        )

    def test_a_project_registered_outside_the_controller_keeps_its_own_home(
        self,
    ) -> None:
        """The negative control: `previous` still wins when it carries something.

        Falling back to `controller_root` unconditionally would be a smaller
        fix that loses the case the field exists for -- a work area whose AGY
        project is not registered at the checkout the round derives from.
        """
        elsewhere = self.root / "elsewhere"
        elsewhere.mkdir()
        self.write_project("project-a", elsewhere)
        path = self.profile_path()
        profile = self.derive(path)
        self.assertEqual(
            Path(profile["worktree"]["project_home_root"]).resolve(),
            elsewhere.resolve(),
        )
        # And a second run keeps it. Repairing every recorded home root rather
        # than only a self-referential one would read as the same fix and quietly
        # replace this answer with the controller checkout.
        profile = self.derive(path)
        self.assertEqual(
            Path(profile["worktree"]["project_home_root"]).resolve(),
            elsewhere.resolve(),
        )

    def test_discard_refuses_a_home_root_that_is_the_worktree(self) -> None:
        """Profiles written before this was fixed are still on disk.

        `discard` reads the field back with `or controller_root`, so the
        correct value is unreachable exactly when the recorded one is wrong.
        """
        path = self.profile_path()
        profile = self.derive(path)
        worker = Path(profile["worktree"]["path"])
        profile["worktree"]["project_home_root"] = str(worker)
        path.write_text(json.dumps(profile, indent=2))
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            agy_dispatch.discard(str(path), "round-1")
        self.assertEqual(self.project_binding(), self.controller.resolve())
        self.assertIn("home root is the worktree itself", buffer.getvalue())

    def dirty_round(self) -> tuple[Path, Path]:
        """A derived round whose worker checkout holds one uncommitted file.

        This is the ordinary state of a candidate: `accept` is the only verb
        that commits, so between `dispatch` and `accept` the worktree is the
        only copy of the work.
        """
        path = self.profile_path()
        profile = self.derive(path)
        worker = Path(profile["worktree"]["path"])
        (worker / "candidate.py").write_text("the only copy\n")
        return path, worker

    def test_discard_refuses_while_the_candidate_is_in_no_commit(self) -> None:
        """`--force` on `worktree remove` made the loss unconditional and silent.

        On #3351 R6 `accept` committed one of two files and the documented next
        step was `discard`; nothing in between said the second file was about
        to go.
        """
        path, worker = self.dirty_round()
        with self.assertRaises(SystemExit) as caught:
            with contextlib.redirect_stdout(io.StringIO()):
                agy_dispatch.discard(str(path), "round-1")
        message = str(caught.exception)
        self.assertIn("candidate.py", message)
        self.assertIn("--drop-uncommitted", message)
        # Refused before anything moved. A discard that repointed the project
        # and then stopped would strand the candidate in a checkout the project
        # no longer opens -- this defect, one step over.
        self.assertTrue((worker / "candidate.py").is_file())
        self.assertEqual(self.project_binding(), worker.resolve())

    def test_discard_destroys_the_candidate_only_when_told_to(self) -> None:
        """The negative control: the opt-in still works, and says what it cost.

        Driven through `main` rather than the function, because a guard whose
        opt-in never reaches it is a guard that cannot be cleared at all.
        """
        path, worker = self.dirty_round()
        output = self.cli("discard", str(path), "round-1", "--drop-uncommitted")
        self.assertFalse(worker.exists())
        self.assertIn("candidate.py", output)
        self.assertEqual(self.project_binding(), self.controller.resolve())

    def commit_candidate(self, worker: Path) -> str:
        """Commit the worker's tree on its own branch, as `accept` does."""
        subprocess.run(["git", "-C", str(worker), "add", "-A"], check=True)
        subprocess.run(
            ["git", "-C", str(worker), "commit", "-q", "-m", "candidate"],
            check=True,
        )
        return subprocess.run(
            ["git", "-C", str(worker), "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True,
        ).stdout.strip()

    def branch_exists(self, branch: str) -> bool:
        return subprocess.run(
            ["git", "-C", str(self.controller), "rev-parse", "--verify",
             "--quiet", f"refs/heads/{branch}"],
            capture_output=True,
        ).returncode == 0

    def test_a_committed_candidate_discards_without_an_opt_in(self) -> None:
        """Throwing away a round whose work is safe is what `discard` is for.

        A guard that fired on every round would be routed around within a day.
        Safe here means integrated: a commit the controller reaches survives the
        branch deletion, which is the whole point of the cherry-pick line.
        """
        path, worker = self.dirty_round()
        sha = self.commit_candidate(worker)
        subprocess.run(
            ["git", "-C", str(self.controller), "cherry-pick", sha],
            capture_output=True, check=True,
        )
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.discard(str(path), "round-1")
        self.assertFalse(worker.exists())
        self.assertFalse(self.branch_exists("agy/round-1"))

    def test_discard_refuses_while_a_commit_reaches_no_other_ref(self) -> None:
        """`accept` commits and prints a cherry-pick line; nothing runs it.

        So the documented round order minus one manual step reaches `discard`
        with the accepted commit referenced by the worker branch alone, and
        `branch -D` does not ask whether anything else holds it.
        """
        path, worker = self.dirty_round()
        sha = self.commit_candidate(worker)
        with self.assertRaises(SystemExit) as caught:
            with contextlib.redirect_stdout(io.StringIO()):
                agy_dispatch.discard(str(path), "round-1")
        message = str(caught.exception)
        self.assertIn(sha, message)
        # And what each one was. A list of hashes alone sends the controller to
        # git before they can tell an accepted candidate from a stray commit.
        self.assertIn("candidate", message)
        self.assertIn("cherry-pick", message)
        self.assertIn("--keep-branch", message)
        # Refused before anything moved, as the uncommitted guard beside it does.
        self.assertTrue(self.branch_exists("agy/round-1"))
        self.assertEqual(self.project_binding(), worker.resolve())

    def test_a_cherry_picked_commit_is_not_stranded_under_a_new_sha(
        self,
    ) -> None:
        """The documented integration produces a different object.

        `cherry-pick` rewrites the committer timestamp, and `-x` the message
        too, so asking only whether some ref reaches this sha would refuse every
        round that did exactly what `accept` told it to -- and a guard that
        fires after the recovery it names is one nobody runs twice.
        """
        path, worker = self.dirty_round()
        sha = self.commit_candidate(worker)
        subprocess.run(
            ["git", "-C", str(self.controller), "cherry-pick", "-x", sha],
            capture_output=True, check=True,
        )
        landed = subprocess.run(
            ["git", "-C", str(self.controller), "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True,
        ).stdout.strip()
        self.assertNotEqual(landed, sha)
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.discard(str(path), "round-1")
        self.assertFalse(self.branch_exists("agy/round-1"))

    def test_a_commit_another_branch_holds_is_not_stranded(self) -> None:
        """Integration is not always onto HEAD.

        A controller who parked the commit on a branch of their own has not lost
        it, and the reachability half of the question is what sees that.
        """
        path, worker = self.dirty_round()
        sha = self.commit_candidate(worker)
        subprocess.run(
            ["git", "-C", str(self.controller), "branch", "parked", sha],
            check=True,
        )
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.discard(str(path), "round-1")
        self.assertFalse(self.branch_exists("agy/round-1"))
        self.assertTrue(self.branch_exists("parked"))

    def test_keep_branch_is_the_only_way_past_an_unreferenced_commit(
        self,
    ) -> None:
        """No opt-in of its own, on purpose.

        A `--drop-commits` twin would be a second way to lose an accepted
        candidate, and the flag that keeps them already exists.
        """
        path, worker = self.dirty_round()
        sha = self.commit_candidate(worker)
        output = self.cli("discard", str(path), "round-1", "--keep-branch")
        self.assertIn("kept branch agy/round-1", output)
        self.assertFalse(worker.exists())
        self.assertTrue(self.branch_exists("agy/round-1"))
        reached = subprocess.run(
            ["git", "-C", str(self.controller), "rev-parse", "agy/round-1"],
            capture_output=True, text=True, check=True,
        ).stdout.strip()
        self.assertEqual(reached, sha)

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
        outcomes: tuple[tuple[int, int], ...] = (),
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
            for idx, status in outcomes:
                connection.execute(
                    "insert into steps values (?, 21, ?, ?)",
                    (idx, status, b""),
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

    def test_abandon_releases_a_round_whose_only_command_was_denied(self) -> None:
        """`abandon` releases a run that provably produced nothing, and a
        request the permission layer refused produced nothing. Counting it as a
        run strands the id behind a command that never started a process."""
        profile = self.dead_round(
            commands=((0, "pwd"),),
            outcomes=((1, 7),),
        )

        agy_dispatch.abandon(profile, "round-1")

        self.assertIsNone(
            agy_dispatch.conversation_id_for_task(profile, "round-1")
        )

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

    def delta_document(self, task: str = "what was wrong") -> str:
        """A delta contract carrying every section a round form carries.

        The fixture used to be a lone `## Task`, which is the exact shape a
        revision written from memory takes -- so it had to become the shape a
        real correction takes before it could stand in for one.
        """
        return (
            f"## Task\n\n{task}\n\n"
            "## Current behavior\n\n`README.md:1` still carries only the base "
            "line:\n\n```\nbase\n```\n\n"
            "## Required change\n\n- The gate's line must be present in "
            "`README.md`.\n\n"
            "## Shape to follow\n\n`README.md` already holds one line per fact; "
            "append rather than rewrite.\n\n"
            "## Reference\n\n| path | why the worker must read it |\n|---|---|\n"
            "| `README.md` | the file the gate reads |\n\n"
            "## Out of scope\n\n- Anything the first round already landed.\n\n"
            "## Definition of done\n\n```\ngrep -q accepted README.md\n```\n"
        )

    def revisable_round(
        self, candidate: bool = True, **overrides: object
    ) -> tuple[dict, Path]:
        """A dispatched round whose candidate is still uncommitted."""
        state = self.root / "state"
        (state / "oracles").mkdir(parents=True, exist_ok=True)
        (state / "injections").mkdir(parents=True, exist_ok=True)
        (state / "oracles" / "round-1.md").write_text("## Claim\n\nthe judge\n")
        (state / "injections" / "round-1.md").write_text("## Task\n\nthe first ask\n")
        overrides.setdefault("path_change_budgets", {"README.md": 40})
        path = self.profile_path(
            inject_prompt_file=str(state / "injections" / "round-1.md"),
            **overrides,
        )
        profile = self.derive(path)
        if candidate:
            (Path(profile["worktree"]["path"]) / "README.md").write_text(
                "base\naccepted\n"
            )
        delta = self.root / "delta.md"
        delta.write_text(self.delta_document())
        return profile, delta

    def revisable_ticketed_round(self) -> tuple[dict, Path]:
        """The same round, driven from an issue instead of a run id.

        `ticketed` is the documented default and every round driven from GitHub
        has it, so this is the common shape rather than a variant.
        """
        self.seed_generated_round()
        state = self.root / "state"
        (state / "oracles").mkdir(parents=True, exist_ok=True)
        (state / "injections").mkdir(parents=True, exist_ok=True)
        (state / "oracles" / "3458.md").write_text("## Claim\n\nthe judge\n")
        (state / "injections" / "3458.md").write_text("## Task\n\nthe first ask\n")
        path = self.profile_path(
            inject_prompt_file=str(state / "injections" / "3458.md"),
            task_contract={
                "kind": "implementation",
                "session_policy": "ticketed",
                "issue": "3458",
                "design_inputs": [
                    {
                        "path": "design.md",
                        "sha256": agy_dispatch.sha256(self.controller / "design.md"),
                    }
                ],
                "gate_command": "grep -q accepted README.md",
            },
        )
        profile = self.derive(path, "3458")
        (Path(profile["worktree"]["path"]) / "README.md").write_text(
            "base\naccepted\n"
        )
        delta = self.root / "delta-ticketed.md"
        delta.write_text(self.delta_document())
        return profile, delta

    def revise_ticket(self, profile: dict, task_key: str, next_key: str) -> dict:
        raw = (
            self.root / "profile.json"
            if task_key == "3458"
            else self.root / "state" / "rounds" / f"{task_key}.profile.json"
        )
        agy_dispatch.revise(
            profile,
            str(raw),
            task_key,
            next_key,
            str(self.root / "delta-ticketed.md"),
        )
        return agy_dispatch.load_profile(
            str(self.root / "state" / "rounds" / f"{next_key}.profile.json")
        )

    def test_revision_refuses_when_copied_oracle_disagrees_with_loosened_profile_budget(
        self,
    ) -> None:
        """A revision carrying a copied oracle refuses dispatch if the profile's budget is loosened."""
        profile, delta = self.revisable_ticketed_round()
        (self.root / "state" / "oracles" / "3458.md").write_text(
            "## Claim\n\nthe judge\n\n"
            "## Measurements\n\n| # | input | expected observation |\n|---|---|---|\n"
            "| 1 | baseline | ok |\n| 2 | test (negative control) | FAIL |\n\n"
            "## Gate\n\n```\ngrep -q accepted README.md\n```\n\n"
            "## Scope\n\n| Path | Line budget |\n|---|---|\n| README.md | 40 |\n\n"
            "## Fabrication tells\n\n- tell\n"
        )
        raw = self.root / "profile.json"
        agy_dispatch.revise(
            profile,
            str(raw),
            "3458",
            "3458-r2",
            str(delta),
        )
        revised_profile_path = self.root / "state" / "rounds" / "3458-r2.profile.json"
        revised_profile = agy_dispatch.load_profile(str(revised_profile_path))
        revised_profile["path_change_budgets"] = {"README.md": 100}
        revised_profile_path.write_text(json.dumps(revised_profile, indent=2))
        loaded = agy_dispatch.load_profile(str(revised_profile_path))
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.snapshot(loaded, "3458-r2")
            with self.assertRaises(SystemExit) as cm:
                agy_dispatch.dispatch(loaded, "3458-r2")
        msg = str(cm.exception)
        self.assertIn("README.md", msg)
        self.assertIn("40", msg)
        self.assertIn("100", msg)

    def test_revising_a_ticket_yields_a_profile_every_verb_can_load(self) -> None:
        """The revision used to be refused by the loader that runs on all of them.

        Stamping `run_id` onto a carried ticketed contract produced a task that
        was both, and `validate_task_identity` refuses exactly that -- so
        `revise` printed a next-step sequence none of whose steps could run.
        The ticket id is the ticketed identity and is spent once a conversation
        exists against it, so a round needing a fresh dispatch cannot keep it.
        """
        profile, _ = self.revisable_ticketed_round()

        revised = self.revise_ticket(profile, "3458", "3458-r2")

        agy_dispatch.validate_task_key(revised, "3458-r2")
        contract = revised["task_contract"]
        self.assertEqual(contract["session_policy"], "one-shot")
        self.assertEqual(contract["run_id"], "3458-r2")
        self.assertNotIn("issue", contract)
        self.assertEqual(contract["revision_of"], "3458")
        # Carried exactly as a one-shot revision is: same tree, same claim.
        self.assertEqual(revised["root"], profile["worktree"]["path"])
        self.assertEqual(
            (self.root / "state" / "oracles" / "3458-r2.md").read_text(),
            "## Claim\n\nthe judge\n",
        )

    def test_a_revised_ticket_reaches_the_worker_as_a_delta_on_that_ticket(
        self,
    ) -> None:
        """`resume` is not the ticketed alternative it looks like.

        It re-renders the *same* injection under a continuation framing, so it
        can say "finish what you started" and cannot say "that was wrong". The
        revision has to carry both the delta and the ticket it descends from.
        """
        profile, _ = self.revisable_ticketed_round()
        revised = self.revise_ticket(profile, "3458", "3458-r2")

        task_state = agy_dispatch.frozen_task_state(revised, "3458-r2")
        prompt = agy_dispatch.render_prompt(
            revised, "3458-r2", "the sealed claim", task_state
        )

        self.assertEqual(task_state["revision_of"], "3458")
        self.assertIn("revising the round dispatched for issue #3458", prompt)
        self.assertIn("what was wrong", prompt)

    def test_a_ticketless_revision_is_left_exactly_as_it_was(self) -> None:
        """The one-shot path already worked, so the conversion must not reach it.

        Its `intent` is the author's, and a machine sentence written over it
        would lose the only statement of what the round is for.
        """
        profile, delta = self.revisable_round()

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )

        contract = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )["task_contract"]
        self.assertEqual(contract["session_policy"], "one-shot")
        self.assertNotIn("revision_of", contract)
        self.assertEqual(contract["intent"], "bounded change")

    def test_a_second_revision_still_names_the_ticket_not_its_parent(self) -> None:
        """Descent is from the ticket, not from the previous run id.

        Carrying the parent's `revision_of` forward is what keeps r3 pointing at
        #3458 rather than at r2, which is a run id no tracker knows.
        """
        profile, _ = self.revisable_ticketed_round()
        second = self.revise_ticket(profile, "3458", "3458-r2")

        third = self.revise_ticket(second, "3458-r2", "3458-r3")

        self.assertEqual(third["task_contract"]["revision_of"], "3458")
        # Rewritten, not carried, so the sentence cannot outlive its own facts.
        self.assertIn(
            "Revision 3458-r3 of round 3458-r2",
            third["task_contract"]["intent"],
        )
        self.assertIn("issue #3458", third["task_contract"]["intent"])

    def test_a_contract_cannot_be_a_ticket_and_descend_from_one(self) -> None:
        """Both fields set names two rounds, and every verb would have to guess."""
        profile, _ = self.revisable_ticketed_round()
        profile["task_contract"]["revision_of"] = "3458"

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.validate_task_identity(profile)

        self.assertIn("must not set task_contract.revision_of", str(error.exception))

    def test_a_descent_is_held_to_the_rule_the_identity_it_names_was(self) -> None:
        """It ends up in a commit trailer, so an unchecked value is a bad `Refs #`."""
        profile, _ = self.revisable_ticketed_round()
        contract = profile["task_contract"]
        del contract["issue"]
        contract.update(
            session_policy="one-shot",
            run_id="3458-r2",
            intent="carried",
            revision_of="../elsewhere",
        )

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.validate_task_identity(profile)

        self.assertIn("revision_of must be the task identity", str(error.exception))

    def test_the_accepted_commit_of_a_revised_ticket_still_refs_it(self) -> None:
        """The trailer is the only place the descent outlives the state dir.

        `/tmp/agy-dispatch` is transient by contract; the commit is not.
        """
        self.isolate_permission_files()
        profile, _ = self.revisable_ticketed_round()
        worker = Path(profile["worktree"]["path"])

        revised = self.revise_ticket(profile, "3458", "3458-r2")
        agy_dispatch.snapshot(revised, "3458-r2")
        self.record_proofs(revised, worker, task_key="3458-r2")
        agy_dispatch.accept(revised, "3458-r2")

        self.assertIn(
            "Refs #3458",
            agy_dispatch.git_output(worker, "log", "-1", "--format=%B"),
        )

    def test_a_revision_keeps_the_checkout_and_mints_a_new_run_id(self) -> None:
        """The whole point: the worker's uncommitted candidate survives.

        A fresh round would run `worktree`, which branches from HEAD and
        silently discards it.
        """
        profile, delta = self.revisable_round()
        worker = Path(profile["worktree"]["path"])

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )

        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        self.assertEqual(revised["task_contract"]["run_id"], "round-2")
        # Carried, not re-derived: same tree, same branch, same base.
        self.assertEqual(revised["root"], str(worker))
        self.assertEqual(revised["worktree"], profile["worktree"])
        # The candidate is still there to be revised.
        self.assertEqual((worker / "README.md").read_text(), "base\naccepted\n")
        # The budget is inherited so the revision is judged against the same
        # ceiling as the round it continues, not given a second one.
        self.assertEqual(revised["path_change_budgets"], {"README.md": 40})
        injection = self.root / "state" / "injections" / "round-2.md"
        self.assertEqual(revised["inject_prompt_file"], str(injection))
        self.assertEqual(injection.read_text(), self.delta_document())
        self.assertEqual(
            (self.root / "state" / "oracles" / "round-2.md").read_text(),
            "## Claim\n\nthe judge\n",
        )
        # The round being revised is untouched; its logs stay the record.
        self.assertEqual(
            (self.root / "state" / "injections" / "round-1.md").read_text(),
            "## Task\n\nthe first ask\n",
        )

    def test_a_revision_refuses_to_reuse_the_spent_run_id(self) -> None:
        profile, delta = self.revisable_round()

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-1",
                str(delta),
            )

        self.assertIn("needs its own run id", str(error.exception))

    def test_a_revision_needs_a_candidate_to_carry(self) -> None:
        """No candidate means nothing is being preserved, and a fresh round is
        both cheaper to author and honest about starting from HEAD."""
        profile, delta = self.revisable_round(candidate=False)

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        self.assertIn("the worker changed nothing", str(error.exception))
        self.assertFalse(
            (self.root / "state" / "rounds" / "round-2.profile.json").exists()
        )

    def test_a_revision_refuses_a_profile_that_never_derived_a_checkout(self) -> None:
        profile, delta = self.revisable_round()
        # A profile still pointing at the controller has no worker tree to keep.
        profile["root"] = str(self.controller)

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        self.assertIn("no round in progress to revise", str(error.exception))

    def test_a_revision_refuses_to_overwrite_a_round_in_flight(self) -> None:
        profile, delta = self.revisable_round()
        taken = self.root / "state" / "rounds" / "round-2.profile.json"
        taken.parent.mkdir(parents=True, exist_ok=True)
        taken.write_text("{}\n")

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        self.assertIn("refusing to overwrite an existing round", str(error.exception))
        self.assertEqual(taken.read_text(), "{}\n")

    def test_the_round_is_refused_before_its_document_is_graded(self) -> None:
        """Both are wrong; only one of them can be acted on.

        A structural finding asks the author to add sections to a delta that
        has nowhere to go. Grading the document first therefore sends them to
        fix the thing that is not blocking them, and the real refusal arrives
        only on the second attempt.
        """
        profile, delta = self.revisable_round()
        delta.write_text("## Task\n\nboth wrong at once\n")
        taken = self.root / "state" / "rounds" / "round-2.profile.json"
        taken.parent.mkdir(parents=True, exist_ok=True)
        taken.write_text("{}\n")

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        self.assertIn("refusing to overwrite an existing round", str(error.exception))
        self.assertNotIn("refusing to carry", str(error.exception))

    def test_a_revision_inherits_a_sealed_claim_or_refuses(self) -> None:
        profile, delta = self.revisable_round()
        (self.root / "state" / "oracles" / "round-1.md").unlink()

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        self.assertIn("no sealed claim", str(error.exception))

    def test_a_revision_refuses_a_delta_contract_that_does_not_exist(self) -> None:
        profile, _ = self.revisable_round()

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(self.root / "absent.md"),
            )

        self.assertIn("revision injection does not exist", str(error.exception))

    def test_a_revision_with_no_delta_named_is_handed_the_blank_form(self) -> None:
        """The revision is the round most likely to be authored from memory.

        Its author holds the previous round in their head, so the natural thing
        to write is a paragraph saying what changed. `scaffold` exists precisely
        so no other round can be authored that way; before this, `revise` was
        the one path around it.
        """
        profile, _ = self.revisable_round()

        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.revise(
                profile, str(self.root / "profile.json"), "round-1", "round-2"
            )

        injection = self.root / "state" / "injections" / "round-2.md"
        self.assertEqual(
            injection.read_text(), agy_dispatch.blank_round_forms(profile)[1]
        )
        # A form, not a document: `lint` refuses it while a slot is unfilled, so
        # the round cannot reach a worker on the strength of having been minted.
        self.assertIn("<!-- fill", injection.read_text())
        self.assertEqual(
            agy_dispatch.missing_or_misordered(
                injection.read_text(), agy_dispatch.INJECTION_SECTIONS, "injection"
            ),
            [],
        )
        self.assertIn("(blank delta form; fill it)", out.getvalue())
        self.assertIn("next     : fill the injection, then lint", out.getvalue())

    def test_a_delta_missing_the_round_form_is_refused_before_a_round_exists(
        self,
    ) -> None:
        """Named here rather than at `lint`, which is two verbs and a snapshot on.

        The section that goes missing is `## Current behavior` -- the one
        carrying the quoted lines `lint` checks against the checkout -- so a
        delta without it is not a shorter contract, it is one whose central
        check has nothing to run against.
        """
        profile, delta = self.revisable_round()
        delta.write_text("## Task\n\nthe candidate wrote the wrong line\n")

        with self.assertRaises(SystemExit) as error:
            agy_dispatch.revise(
                profile,
                str(self.root / "profile.json"),
                "round-1",
                "round-2",
                str(delta),
            )

        message = str(error.exception)
        self.assertIn("refusing to carry", message)
        for section in agy_dispatch.INJECTION_SECTIONS[1:]:
            self.assertIn(f"`## {section}` section", message)
        self.assertIn("Omit the argument", message)
        # Refused before the round exists, so there is nothing half-minted to
        # clean up and no id spent on a contract that cannot be dispatched.
        self.assertFalse(
            (self.root / "state" / "rounds" / "round-2.profile.json").exists()
        )
        self.assertFalse((self.root / "state" / "injections" / "round-2.md").exists())

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

    def test_accept_commits_the_whole_candidate_not_the_last_revision(self) -> None:
        """A revised round's candidate spans every revision, and all of it lands.

        `revise` exists to carry uncommitted work forward under a new run id,
        so the last revision's writes are a strict subset of the candidate.
        Committing only that subset lands a tree missing the half that
        `discard` is about to delete -- observed on #3351 R6, where the commit
        held the caller and not the file defining the symbol it calls.
        """
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        worker = Path(profile["worktree"]["path"])
        # The first revision's write. The second never touches it again, so a
        # diff taken against this round's snapshot cannot see it.
        self.assertEqual((worker / "README.md").read_text(), "base\naccepted\n")

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")
        (worker / "HELPER.md").write_text("helper\nsecond revision\n")
        self.record_proofs(revised, worker, task_key="round-2")

        agy_dispatch.accept(revised, "round-2")

        self.assertEqual(
            sorted(
                agy_dispatch.git_output(
                    worker, "show", "--name-only", "--format=", "HEAD"
                ).split()
            ),
            ["HELPER.md", "README.md"],
        )
        self.assertEqual(agy_dispatch.git_output(worker, "status", "--porcelain"), "")

    def overrun_round(self) -> tuple[dict, dict, Path]:
        """A round sent back while one of its paths is over budget.

        The overrun is the reason the round is coming back, so the revision that
        answers it starts with the finding still open and the candidate that
        earned it still in the tree.
        """
        profile, delta = self.revisable_round(
            path_change_budgets={"README.md": 2}
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text(
            "base\n" + "".join(f"line {n}\n" for n in range(6))
        )
        first = agy_dispatch.scope_findings(
            profile, agy_dispatch.worker_touched_paths(profile, "round-1"), "round-1"
        )
        self.assertTrue(
            any("exceeds the 2-line budget" in item for item in first), first
        )

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")
        return profile, revised, worker

    def revision_findings(self, revised: dict) -> list[str]:
        return agy_dispatch.scope_findings(
            revised,
            agy_dispatch.worker_touched_paths(revised, "round-2"),
            "round-2",
        )

    def test_a_revision_does_not_re_baseline_an_open_overrun(self) -> None:
        """A budget is a total, so a revision cannot spend it a second time.

        `revise` carries the ceiling and used to re-baseline what it applied to,
        which returns the whole allowance on every revision. A round can be
        revised any number of times, so the round's real diff is unbounded while
        every `verify` after the first reads clean.
        """
        _, revised, _ = self.overrun_round()

        # This revision wrote nothing at all, which is what makes the finding
        # invisible: its own delta for the path is zero.
        self.assertEqual(agy_dispatch.worker_touched_paths(revised, "round-2"), [])
        findings = self.revision_findings(revised)
        self.assertTrue(
            any("README.md: 6 changed lines exceeds the 2-line budget" in item
                for item in findings),
            findings,
        )

    def test_a_narrowed_revision_does_not_report_what_it_carried(self) -> None:
        """Narrowing the write set on a revision is the normal case.

        A revision that only adds a test should not be allowed to edit the
        files it is asserting about, so the carried paths leave
        `allowed_repo_writes` and are frozen at their carried digests instead.
        The bytes are then in the tree and outside the declared set, which is
        the exact shape of a real out-of-scope write.
        """
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "HELPER.md").write_text("helper\nfirst revision\n")

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        # The narrowing as the reported round did it: the carried path leaves
        # the write set and is frozen as a design input at the bytes it was
        # carried with. `design_inputs` is the set `load_profile` hash-checks,
        # so drift there refuses the profile outright rather than reporting.
        revised["allowed_repo_writes"] = ["README.md"]
        revised["task_contract"]["design_inputs"] = [
            *revised["task_contract"]["design_inputs"],
            {
                "path": "HELPER.md",
                "sha256": agy_dispatch.sha256(worker / "HELPER.md"),
            },
        ]
        (self.root / "state" / "rounds" / "round-2.profile.json").write_text(
            json.dumps(revised, indent=2) + "\n"
        )
        agy_dispatch.snapshot(revised, "round-2")

        findings = self.revision_findings(revised)
        self.assertFalse(
            any("outside allowed_repo_writes" in item for item in findings), findings
        )
        # ...and a real write to the same path is still a finding.
        (worker / "HELPER.md").write_text("helper\nthis round wrote here\n")
        after = self.revision_findings(revised)
        self.assertTrue(
            any("outside allowed_repo_writes: HELPER.md" in item for item in after),
            after,
        )

    def test_a_carried_protected_path_is_not_attributed_to_this_worker(self) -> None:
        """The protected half of the baseline, keyed the way it is really keyed.

        `load_profile` absolutizes `protected_artifacts`, so the snapshot records
        their content under absolute paths while `writable_contents` is keyed
        repo-relative. Merging both maps as if repo-relative leaves the protected
        half matching nothing, which is invisible at `scope_findings` -- it
        suppresses protected paths by name anyway -- and visible here, where
        `review` asks which paths this worker wrote.
        """
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "HELPER.md").write_text("helper\nfirst revision\n")

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        revised["allowed_repo_writes"] = ["README.md"]
        # Absolute, because that is what the profile carries by the time any
        # verb reads it -- writing it relative here would test a shape that
        # never reaches a snapshot.
        revised["protected_artifacts"] = [
            *revised["protected_artifacts"],
            {
                "path": str(worker / "HELPER.md"),
                "sha256": agy_dispatch.sha256(worker / "HELPER.md"),
            },
        ]
        (self.root / "state" / "rounds" / "round-2.profile.json").write_text(
            json.dumps(revised, indent=2) + "\n"
        )
        agy_dispatch.snapshot(revised, "round-2")

        self.assertNotIn(
            "HELPER.md", agy_dispatch.worker_touched_paths(revised, "round-2")
        )
        # ...and this worker writing it is still this worker's write.
        (worker / "HELPER.md").write_text("helper\nthis round wrote here\n")
        self.assertIn(
            "HELPER.md", agy_dispatch.worker_touched_paths(revised, "round-2")
        )

    def test_lines_a_round_removed_count_against_its_budget(self) -> None:
        """Otherwise the cheapest way under a budget is to delete the file.

        A budget bounds how much of the tree a round moves, and a deletion
        moves it exactly as far as an insertion does. Found by a surviving
        mutant: `added + removed` was never distinguished from `added`.
        """
        (self.controller / "LONG.md").write_text(
            "".join(f"line {n}\n" for n in range(8))
        )
        self.git("add", "-A")
        self.git("commit", "-qm", "long")
        self.isolate_permission_files()
        profile = self.derive(
            self.profile_path(
                allowed_repo_writes=["LONG.md"],
                path_change_budgets={"LONG.md": 2},
            )
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "LONG.md").write_text("line 0\n")

        findings = agy_dispatch.scope_findings(
            profile, agy_dispatch.worker_touched_paths(profile)
        )
        self.assertTrue(
            any("LONG.md: 7 changed lines exceeds the 2-line budget" in item
                for item in findings),
            findings,
        )

    def test_a_file_the_predecessor_created_still_counts_against_its_budget(
        self,
    ) -> None:
        """The case the removed branch was written for, decided the other way.

        An untracked file the previous revision created is invisible to
        `git diff base`, so its whole length is the round's diff for that path.
        The old comment called billing it a bug, but it was answering "which
        worker wrote this" with a budget, which is a question about the round.
        Attribution stays where it belongs: `worker_touched_paths` still leaves
        the path out of what this revision wrote.
        """
        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "NEW.md"],
            path_change_budgets={"NEW.md": 2},
        )
        worker = Path(profile["worktree"]["path"])
        (worker / "NEW.md").write_text("".join(f"line {n}\n" for n in range(5)))

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")

        self.assertNotIn(
            "NEW.md", agy_dispatch.worker_touched_paths(revised, "round-2")
        )
        findings = self.revision_findings(revised)
        self.assertTrue(
            any("NEW.md: 5 changed lines exceeds the 2-line budget" in item
                for item in findings),
            findings,
        )

    def test_a_revision_that_gets_back_under_budget_stops_reporting(self) -> None:
        """The finding is withdrawn by the work, never by the re-baselining."""
        _, revised, worker = self.overrun_round()
        (worker / "README.md").write_text("base\nline 0\n")

        findings = self.revision_findings(revised)
        self.assertFalse(
            any("budget" in item for item in findings), findings
        )

    def test_review_still_asks_what_this_revision_wrote(self) -> None:
        """Scope is a per-revision question even though acceptance is not.

        A finding says "this worker wrote outside its contract", so charging a
        revision for its predecessor's paths would report the carried-forward
        candidate as overreach on every revision after the first.
        """
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        worker = Path(profile["worktree"]["path"])

        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")
        (worker / "HELPER.md").write_text("helper\nsecond revision\n")

        self.assertEqual(
            agy_dispatch.worker_touched_paths(revised, "round-2"), ["HELPER.md"]
        )
        self.assertEqual(
            agy_dispatch.worker_touched_paths(revised), ["HELPER.md", "README.md"]
        )

        # ...and says so, so the header cannot be read as the whole candidate.
        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.review(revised, "round-2")
        printed = out.getvalue()
        self.assertIn("touched  : 1 path(s) written this revision", printed)
        self.assertIn("carried  : 1 path(s) from an earlier revision", printed)
        self.assertRegex(printed, r"carried  :[^\n]*\n    README\.md")

    def test_review_does_not_call_a_carried_path_unwritten(self) -> None:
        """#3424: the finding must not fire on work the candidate contains.

        A scope finding that fires on a correct revised round trains the
        controller to accept over a non-empty findings list, which is the one
        habit the check exists to prevent.
        """
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        worker = Path(profile["worktree"]["path"])
        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")
        (worker / "HELPER.md").write_text("helper\nsecond revision\n")

        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.review(revised, "round-2")
        self.assertIn("findings: none", out.getvalue())

    def test_a_declared_path_nobody_wrote_is_still_a_finding(self) -> None:
        """The negative control for #3424: widening the measurement must not
        retire the check. Nothing in either revision writes `HELPER.md`."""
        (self.controller / "HELPER.md").write_text("helper\n")
        self.git("add", "-A")
        self.git("commit", "-qm", "helper")
        self.isolate_permission_files()

        profile, delta = self.revisable_round(
            allowed_repo_writes=["README.md", "HELPER.md"]
        )
        agy_dispatch.revise(
            profile, str(self.root / "profile.json"), "round-1", "round-2", str(delta)
        )
        revised = json.loads(
            (self.root / "state" / "rounds" / "round-2.profile.json").read_text()
        )
        agy_dispatch.snapshot(revised, "round-2")

        with contextlib.redirect_stdout(io.StringIO()) as out:
            with self.assertRaises(SystemExit):
                agy_dispatch.review(revised, "round-2")
        self.assertIn(
            "declared but did not write 1 path(s): HELPER.md", out.getvalue()
        )

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

    def failing_to_compile_gate(self) -> dict:
        """A gate whose red is a build failure rather than a failed assertion."""
        contract = self.contract_with_design_input()
        # `sys.exit(str)` writes the message to stderr and exits non-zero.
        contract["gate_command"] = (
            "python3 -c \"import sys; sys.exit('error: could not compile it')\""
        )
        return contract

    def test_prove_records_whether_the_gate_compiled(self) -> None:
        """Exit code alone cannot tell a failed assertion from a build failure,
        and the two are not the same evidence."""
        profile = self.derive(self.profile_path(task_contract=self.failing_to_compile_gate()))
        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.prove(profile, "round-1", "mutant")
        record = json.loads(
            agy_dispatch.proof_path(profile, "round-1", "mutant").read_text()
        )
        self.assertNotEqual(record["exit_code"], 0)
        self.assertFalse(record["compiled"])
        self.assertIn("says nothing about behaviour", out.getvalue())

    def compound_gate(self) -> dict:
        """The ordinary gate shape: build, and only if that worked, test."""
        contract = self.contract_with_design_input()
        contract["gate_command"] = (
            "python3 -c \"print('built')\" && python3 -c \"print('tested')\""
        )
        return contract

    def test_prove_runs_a_compound_gate_as_one_command_line(self) -> None:
        """A gate is a command line, not an argv.

        Split into tokens, `&&` is handed to the first command as an argument;
        it refuses, the second half never runs, and the refusal says nothing
        about compilation -- so the proof records a red that looks behavioural
        and is not. Every round in a repository whose gate is compound recorded
        that pair.
        """
        profile = self.derive(self.profile_path(task_contract=self.compound_gate()))
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.prove(profile, "round-1", "candidate")
        record = json.loads(
            agy_dispatch.proof_path(profile, "round-1", "candidate").read_text()
        )
        tail = "\n".join(record["output_tail"])
        self.assertEqual(record["exit_code"], 0)
        self.assertIn("built", tail)
        self.assertIn("tested", tail, "the half after `&&` never ran")

    def test_prove_does_not_call_a_gate_the_shell_never_found_compiled(self) -> None:
        """127 is the shell saying the gate does not exist.

        That belongs with the build failures: no behaviour was observed, so a
        red carrying it must not be counted as the gate noticing anything.
        """
        contract = self.contract_with_design_input()
        contract["gate_command"] = "agy-dispatch-gate-that-does-not-exist"
        profile = self.derive(self.profile_path(task_contract=contract))
        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.prove(profile, "round-1", "mutant")
        record = json.loads(
            agy_dispatch.proof_path(profile, "round-1", "mutant").read_text()
        )
        self.assertEqual(record["exit_code"], 127)
        self.assertFalse(record["compiled"])
        self.assertIn("says nothing about behaviour", out.getvalue())

    def cli(self, *argv: str) -> str:
        """Drive the real entry point, because the relaxation lives in it.

        `prove` receives a profile that was already loaded, so a test calling it
        directly can never see the check this is about -- `load_profile` is
        where the freeze is refused, and the verb-and-label it is refused for is
        only known at the command line.
        """
        out = io.StringIO()
        argv = ("agy_dispatch.py",) + argv
        with contextlib.redirect_stdout(out):
            with unittest.mock.patch.object(sys, "argv", list(argv)):
                agy_dispatch.main()
        return out.getvalue()

    def guarded_round(self) -> tuple[Path, dict, Path]:
        """A round whose only real falsifier lives where it may not write.

        The reported shape: the worker adds an assertion about a file it is
        forbidden to edit, so the file is frozen and the write set is the
        assertion alone. Breaking the assertion means breaking the frozen file.
        """
        contract = self.contract_with_design_input()
        contract["gate_command"] = (
            "bash -c 'grep -q accepted README.md && grep -q frozen keep.md'"
        )
        path = self.profile_path(task_contract=contract)
        profile = self.derive(path)
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        return path, profile, worker

    def test_only_prove_mutant_runs_against_an_unfrozen_tree(self) -> None:
        """The predicate, stated once, because everything else reads it."""
        self.assertFalse(agy_dispatch.validates_freeze("prove", "mutant"))
        self.assertTrue(agy_dispatch.validates_freeze("prove", "candidate"))
        self.assertTrue(agy_dispatch.validates_freeze("accept", None))
        self.assertTrue(agy_dispatch.validates_freeze("sweep", None))
        # Not a blanket exemption for the verb, and not one for the label.
        self.assertTrue(agy_dispatch.validates_freeze("snapshot", "mutant"))

    def test_prove_mutant_records_a_falsifier_the_round_may_not_write(self) -> None:
        """#3484/#3489: the controller plants the break, after the worker is gone.

        A round whose claim is about a frozen file has its falsifier out there
        and nowhere else. Refused, the round records whatever weaker mutant fits
        inside the write scope, and its durable artifact then understates what
        was measured.
        """
        path, profile, worker = self.guarded_round()
        (worker / "keep.md").write_text("thawed\n")

        out = self.cli("prove", str(path), "round-1", "mutant")

        record = json.loads(
            agy_dispatch.proof_path(profile, "round-1", "mutant").read_text()
        )
        self.assertNotEqual(record["exit_code"], 0)
        # R2: the proof names what was perturbed, so a reader can tell a planted
        # mutant from a worker that edited a frozen file.
        self.assertEqual([entry["path"] for entry in record["perturbed"]], ["keep.md"])
        self.assertIn("keep.md", out)
        self.assertIn("differ from this round's freeze", out)

    def test_prove_candidate_still_refuses_a_perturbed_freeze(self) -> None:
        """The half that decides. Relaxing the mutant only works because this
        one does not move: the pair cannot be completed until the controller's
        perturbation is restored."""
        path, _, worker = self.guarded_round()
        (worker / "keep.md").write_text("thawed\n")
        with self.assertRaises(SystemExit) as caught:
            self.cli("prove", str(path), "round-1", "candidate")
        self.assertIn("keep.md", str(caught.exception))

    def test_a_pair_that_moved_only_the_frozen_half_is_a_real_pair(self) -> None:
        """`candidate_tree_digest` covers `allowed_repo_writes` and nothing else,
        so this pair is byte-identical in everything it looked at."""
        path, profile, worker = self.guarded_round()
        (worker / "keep.md").write_text("thawed\n")
        self.cli("prove", str(path), "round-1", "mutant")
        (worker / "keep.md").write_text("frozen\n")
        self.cli("prove", str(path), "round-1", "candidate")

        mutant, candidate = (
            json.loads(agy_dispatch.proof_path(profile, "round-1", label).read_text())
            for label in ("mutant", "candidate")
        )
        self.assertEqual(mutant["tree_digest"], candidate["tree_digest"])
        findings = agy_dispatch.proof_findings(profile, "round-1")
        self.assertEqual(findings, [], findings)

    def test_accept_still_refuses_while_a_frozen_path_is_perturbed(self) -> None:
        """A recorded pair is not a licence to commit a tree that drifted."""
        path, profile, worker = self.guarded_round()
        (worker / "keep.md").write_text("thawed\n")
        self.cli("prove", str(path), "round-1", "mutant")
        (worker / "keep.md").write_text("frozen\n")
        self.cli("prove", str(path), "round-1", "candidate")

        (worker / "keep.md").write_text("perturbed again\n")
        with self.assertRaises(SystemExit) as caught:
            self.cli("accept", str(path), "round-1")
        self.assertIn("keep.md", str(caught.exception))

    def test_accept_says_a_compile_error_mutant_measured_no_behaviour(self) -> None:
        """A round that introduces a new symbol can only answer the revert with
        a build failure, so this cannot block acceptance -- but recorded as an
        ordinary non-zero exit it reads exactly like a behavioural kill."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        path = agy_dispatch.proof_path(profile, "round-1", "mutant")
        record = json.loads(path.read_text())
        record["compiled"] = False
        path.write_text(json.dumps(record))

        with contextlib.redirect_stdout(io.StringIO()) as out:
            agy_dispatch.accept(profile, "round-1")
        self.assertIn("did not compile", out.getvalue())
        self.assertIn("still compile", out.getvalue())

    def test_a_compiling_mutant_earns_no_compile_note(self) -> None:
        """Scoped to the compile note. Asserting the whole list is empty makes
        this test speak for every note class there will ever be, and it silently
        started failing the moment the sweep note was added -- a red that says
        nothing about whether a compiling mutant is treated correctly."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        notes = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual([n for n in notes if "did not compile" in n], [])

    def test_a_round_without_a_sweep_earns_a_note(self) -> None:
        """The proof pair shows the gate notices *this* change. It cannot show
        the gate would notice a different defect in the same code, which is what
        the sweep is for, so its absence has to be said out loud."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        notes = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual(len([n for n in notes if "no mutation sweep" in n]), 1)

    def record_sweep(self, profile: dict, **overrides: object) -> Path:
        record = {"restored": True, "exit_code": 0}
        record.update(overrides)
        path = agy_dispatch.sweep_path(profile, "round-1")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(record))
        return path

    def test_a_recorded_sweep_clears_the_note(self) -> None:
        """Recording a sweep removes the sweep note and adds none of its own.

        This asserted that `proof_notes` returned nothing at all, which is a
        claim about every note the function can emit and not about the sweep.
        222f519c9d added a correct one -- this fixture's gate is
        `grep -q accepted README.md`, which names no failing test, so its red
        mutant genuinely cannot be attributed -- and the bare assertion went red
        for behaviour working as designed. A test that breaks when unrelated
        correct behaviour is added is a test that gets skimmed, which is what
        happened to it and to seven others (#3495).

        So it names both halves: the sweep note is gone, and what remains is
        exactly the one note this fixture is known to earn. That is stricter
        than filtering for the sweep note, because a new unexplained note still
        fails it -- it just fails it with the note's own text in the message."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        before = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual(len([n for n in before if "no mutation sweep" in n]), 1)
        self.record_sweep(profile)
        after = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual([n for n in after if "mutation sweep" in n], [])
        # Required, not merely tolerated. Tolerating it left the note 222f519c9d
        # added with no test at all: a mutation removing it passed the whole
        # suite, so the one thing separating a kill by this round's rows from an
        # unrelated flake could have been deleted without anything noticing.
        self.assertEqual(
            len([n for n in after if "without the gate naming a failing test" in n]),
            1,
            after,
        )
        self.assertEqual(
            [n for n in after if "without the gate naming a failing test" not in n],
            [],
            after,
        )

    def test_a_sweep_that_did_not_restore_the_tree_still_earns_a_note(self) -> None:
        """`sweep` refuses an unrestored tree, but it writes the record before
        refusing, so the file proving the sweep was worthless was the same file
        that cleared the note."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        self.record_sweep(profile, restored=False)
        notes = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual(len([n for n in notes if "did not restore" in n]), 1)

    def test_an_unreadable_sweep_record_earns_a_note(self) -> None:
        """A truncated or hand-edited record is the one file standing in for the
        whole sweep, so failing to read it has to be louder than reading a clean
        one -- not quieter, which is what returning early gives."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        path = agy_dispatch.sweep_path(profile, "round-1")
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text('{"restored": tru')
        notes = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual(len([n for n in notes if "cannot be read" in n]), 1)

    def test_a_sweep_that_exited_non_zero_earns_a_note(self) -> None:
        """A sweep that crashed halfway and a sweep whose mutant survived both
        exit non-zero, and the round has to say which it was."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        self.record_proofs(profile, worker)
        self.record_sweep(profile, exit_code=2)
        notes = agy_dispatch.proof_notes(profile, "round-1")
        self.assertEqual(len([n for n in notes if "exited 2" in n]), 1)

    def test_sweep_refuses_when_the_script_leaves_the_tree_mutated(self) -> None:
        """A sweep that does not put back what it took out measures every later
        mutant against the wrong baseline, and reports them all as killed."""
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        script = Path(profile["state_dir"]) / "leaky_sweep.py"
        script.write_text(
            "from pathlib import Path\n"
            "Path('README.md').write_text('base\\nmutated\\n')\n"
        )
        with contextlib.redirect_stdout(io.StringIO()):
            with self.assertRaises(SystemExit) as caught:
                agy_dispatch.sweep(profile, "round-1", str(script))
        self.assertIn("did not restore what it mutated", str(caught.exception))
        self.assertTrue(agy_dispatch.sweep_path(profile, "round-1").exists())

    def test_sweep_accepts_a_script_that_restores_the_tree(self) -> None:
        profile = self.derive(self.profile_path())
        worker = Path(profile["worktree"]["path"])
        (worker / "README.md").write_text("base\naccepted\n")
        script = Path(profile["state_dir"]) / "clean_sweep.py"
        script.write_text(
            "from pathlib import Path\n"
            "p = Path('README.md')\n"
            "original = p.read_text()\n"
            "p.write_text('base\\nmutated\\n')\n"
            "p.write_text(original)\n"
        )
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.sweep(profile, "round-1", str(script))
        record = json.loads(agy_dispatch.sweep_path(profile, "round-1").read_text())
        self.assertTrue(record["restored"])
        self.assertEqual(record["exit_code"], 0)

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

## Scope

| Path | Line budget | Line ranges |
|---|---|---|

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

    def test_oracle_sections_tuple_has_scope_at_index_3(self) -> None:
        self.assertEqual(
            agy_dispatch.ORACLE_SECTIONS,
            ("Claim", "Measurements", "Gate", "Scope", "Fabrication tells"),
        )

    def test_missing_scope_section_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "## Scope\n\n| Path | Line budget | Line ranges |\n|---|---|---|\n\n",
            "",
        )
        self.assert_single_finding(text, "missing the `## Scope` section")

    def test_scope_mismatch_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "| Path | Line budget | Line ranges |\n|---|---|---|",
            "| Path | Line budget | Line ranges |\n|---|---|---|\n| src/writable.py | 50 | any |",
        )
        profile = {
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "allowed_repo_writes": ["src/writable.py"],
            "path_change_budgets": {"src/writable.py": 100},
            "path_line_ranges": {"src/writable.py": "any"},
        }
        found = agy_dispatch.oracle_findings(profile, text)
        expected = "`## Scope` write scope mismatch for `src/writable.py`: oracle states 50, profile carries 100"
        self.assertEqual(found, [expected])

    def test_scope_mismatch_range_only_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "| Path | Line budget | Line ranges |\n|---|---|---|",
            "| Path | Line budget | Line ranges |\n|---|---|---|\n| src/writable.py | 50 | 1-10 |",
        )
        profile = {
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "allowed_repo_writes": ["src/writable.py"],
            "path_change_budgets": {"src/writable.py": 50},
            "path_line_ranges": {"src/writable.py": "1-20"},
        }
        found = agy_dispatch.oracle_findings(profile, text)
        expected = "`## Scope` write scope mismatch for `src/writable.py`: oracle states 1-10, profile carries 1-20"
        self.assertEqual(found, [expected])

    def test_scope_mismatch_budget_and_range_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "| Path | Line budget | Line ranges |\n|---|---|---|",
            "| Path | Line budget | Line ranges |\n|---|---|---|\n| src/writable.py | 50 | 1-10 |",
        )
        profile = {
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "allowed_repo_writes": ["src/writable.py"],
            "path_change_budgets": {"src/writable.py": 100},
            "path_line_ranges": {"src/writable.py": "1-20"},
        }
        found = agy_dispatch.oracle_findings(profile, text)
        expected = "`## Scope` write scope mismatch for `src/writable.py`: oracle states 50 1-10, profile carries 100 1-20"
        self.assertEqual(found, [expected])

    def test_scope_mismatch_path_only_in_oracle_is_reported(self) -> None:
        text = CONFORMANT_ORACLE.replace(
            "| Path | Line budget | Line ranges |\n|---|---|---|",
            "| Path | Line budget | Line ranges |\n|---|---|---|\n| src/only_in_oracle.py | 50 | any |",
        )
        found = agy_dispatch.oracle_findings(self.PROFILE, text)
        expected = "`## Scope` write scope mismatch for `src/only_in_oracle.py`: oracle states 50, profile carries absent"
        self.assertEqual(found, [expected])

    def test_scope_mismatch_path_only_in_profile_is_reported(self) -> None:
        profile = {
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "allowed_repo_writes": ["src/extra_in_profile.py"],
            "path_change_budgets": {"src/extra_in_profile.py": 30},
            "path_line_ranges": {"src/extra_in_profile.py": "any"},
        }
        found = agy_dispatch.oracle_findings(profile, CONFORMANT_ORACLE)
        expected = "`## Scope` write scope mismatch for `src/extra_in_profile.py`: oracle states absent, profile carries 30"
        self.assertEqual(found, [expected])

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

    def test_a_control_named_only_in_a_rationale_cell_is_reported(self) -> None:
        """"Unlike the negative control, this row..." is a true sentence a
        controller writes about a *different* row. It must not be what makes
        the table conformant."""
        text = CONFORMANT_ORACLE.replace(
            "| # | input | expected observation |\n"
            "|---|---|---|\n"
            "| 1 | baseline body | digest D |\n"
            "| 2 | marker updated_at bumped | digest still D |\n"
            "| 3 | prose edited (negative control) | digest != D |",
            "| # | input | expected observation | why |\n"
            "|---|---|---|---|\n"
            "| 1 | baseline body | digest D | the rule fires |\n"
            "| 2 | marker updated_at bumped | digest still D | idempotent |\n"
            "| 3 | prose edited | digest != D | unlike the negative control |",
        )
        self.assert_single_finding(text, "no row marked `negative control`")

    def test_a_control_marked_in_its_observation_is_accepted(self) -> None:
        """A control is defined by what it feeds and what that must not
        produce; only a trailing rationale cell is prose about other rows."""
        text = CONFORMANT_ORACLE.replace(
            "| 3 | prose edited (negative control) | digest != D |",
            "| 3 | prose edited | digest != D (negative control) |",
        )
        self.assertEqual(self.findings(text), [])

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
            "## Scope\n\n| Path | Line budget |\n|---|---|\n\n"
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

    def test_a_second_gate_command_prove_never_runs_is_reported(self) -> None:
        """`prove` runs `task_contract.gate_command` and nothing else, so an
        oracle listing a second command carries a row no proof ever covers."""
        profile = {
            "task_commands": {
                "allow": [
                    "cargo test -p target --lib some_gate",
                    "cargo build -p target --lib",
                ]
            },
            "task_contract": {
                "gate_command": "cargo test -p target --lib some_gate"
            },
        }
        text = CONFORMANT_ORACLE.replace(
            "cargo test -p target --lib some_gate\n```",
            "cargo build -p target --lib\ncargo test -p target --lib some_gate\n```",
        )
        found = agy_dispatch.oracle_findings(profile, text)
        self.assertEqual(len(found), 1, f"expected one finding, got {found}")
        self.assertIn("`prove` will never run", found[0])
        self.assertIn("cargo build -p target --lib", found[0])

    def test_the_judged_gate_alone_is_not_reported(self) -> None:
        """The check must fire on the extra command, not on every profile that
        happens to declare a `gate_command`."""
        profile = {
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
            "task_contract": {
                "gate_command": "cargo test -p target --lib some_gate"
            },
        }
        self.assertEqual(
            agy_dispatch.oracle_findings(profile, CONFORMANT_ORACLE), []
        )


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
        # The conformant injection quotes this line under `## Current behavior`,
        # and a quote is only conformant when the file actually carries it.
        (self.root / "src" / "thing.rs").write_text(
            "fn store(body: &str) {\n    let digest = canonical_digest(body);\n}\n"
        )
        self.profile = {
            "root": str(self.root),
            "task_commands": {"allow": ["cargo test -p target --lib some_gate"]},
        }
        # What `capture` would have stored for the transcript these rows show.
        self.captures = {
            "target/debug/thing show": ["digest: 9f2c (markers included)"]
        }

    def findings(self, text: str) -> list[str]:
        return agy_dispatch.injection_findings(
            self.profile, text, CONFORMANT_ORACLE, self.captures
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

    def test_a_quote_no_file_carries_is_reported(self) -> None:
        """A fence satisfies the quote rule whether or not the code is still
        there, so a document re-based onto a moved tree keeps its old excerpt and
        the worker greps for a line that no longer exists."""
        text = CONFORMANT_INJECTION.replace(
            "let digest = canonical_digest(body);",
            "let digest = canonical_digest(body, opts);",
        )
        self.assert_single_finding(text, "appear in none of the round's files")

    def test_a_re_indented_quote_is_not_reported(self) -> None:
        """Indentation is not content: a quote lifted out of its block is still
        the code that is there, and flagging it would train the controller to
        satisfy the check by pasting whitespace rather than by re-reading."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\n        let digest = canonical_digest(body);\n```",
        )
        self.assertEqual(self.findings(text), [])

    def test_an_elision_marker_is_not_reported(self) -> None:
        """An excerpt that skips lines has to say so, and the marker saying so is
        not itself a claim about the file."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n// ...\n```",
        )
        self.assertEqual(self.findings(text), [])

    def test_a_stale_comment_line_is_still_reported(self) -> None:
        """The elision marker is a comment, so the exemption is one careless
        widening away from skipping every comment -- and a doc comment quoted
        from a version where it said something else is exactly the excerpt a
        worker would trust hardest."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\n// markers are excluded here\n"
            "let digest = canonical_digest(body);\n```",
        )
        self.assert_single_finding(text, "appear in none of the round's files")

    def test_a_console_transcript_is_not_compared_against_the_files(self) -> None:
        """Current behavior is often what the binary prints, and no file carries
        those lines. Comparing them against the source reported every real
        transcript as stale, and the only fix available was to delete the fence
        and paraphrase -- trading a verbatim observation for prose."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n```\n\n"
            "```console\n$ target/debug/thing show\ndigest: 9f2c (markers included)\n```",
        )
        self.assertEqual(self.findings(text), [])

    def test_a_console_transcript_without_its_command_is_reported(self) -> None:
        """The exemption removes this block's only check, so it has to carry
        another. Output pasted alone is indistinguishable from output remembered,
        from an older build, or from a different argument."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n```\n\n"
            "```console\ndigest: 9f2c (markers included)\n```",
        )
        self.assert_single_finding(text, "does not open with")

    def test_a_transcript_alone_does_not_satisfy_the_quote_rule(self) -> None:
        """Otherwise the exemption is a way out of the quote rule itself: tag the
        one fence `console` and the section is never checked against the tree the
        worker is about to open."""
        self.captures = {"target/debug/thing show": ["digest: 9f2c"]}
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```console\n$ target/debug/thing show\ndigest: 9f2c\n```",
        )
        self.assert_single_finding(text, "no non-empty fenced quote")

    def test_a_transcript_nobody_captured_is_reported(self) -> None:
        """The prompt line says a command was run, and saying so is free. Two
        paraphrases shipped past it in one round -- one naming a flag the verb
        does not accept, one whose behavior existed only in a build newer than
        the installed binary -- and lint called the round clean (#3426)."""
        self.captures = {}
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n```\n\n"
            "```console\n$ target/debug/thing show\ndigest: 9f2c (markers included)\n```",
        )
        self.assert_single_finding(text, "this round never captured")

    def test_a_transcript_edited_after_its_capture_is_reported(self) -> None:
        """Capturing and then editing the block is the same defect wearing the
        record as cover, and it is the one a capture step invites: the honest
        run happened, and the pasted lines are still not what it printed."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n```\n\n"
            "```console\n$ target/debug/thing show\ndigest: 9f2c (markers excluded)\n```",
        )
        self.assert_single_finding(text, "differs from what the captured run")

    def test_copy_paste_whitespace_does_not_count_as_an_edit(self) -> None:
        """The comparison has to survive the paste it exists to encourage.
        Trailing spaces and a blank line before the fence carry none of the
        observation, and a rule that reads them as divergence teaches the
        controller to stop capturing and go back to typing."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```\nlet digest = canonical_digest(body);\n```\n\n"
            "```console\n$ target/debug/thing show   \n"
            "digest: 9f2c (markers included)  \n\n```",
        )
        self.assertEqual(self.findings(text), [])

    def test_a_stale_quote_in_a_non_console_fence_is_still_reported(self) -> None:
        """The exemption keys on the info string alone, so a language-tagged
        fence must stay in scope -- otherwise ```rust becomes the same escape
        hatch by a different name."""
        text = CONFORMANT_INJECTION.replace(
            "```\nlet digest = canonical_digest(body);\n```",
            "```rust\nlet digest = canonical_digest(body, opts);\n```",
        )
        self.assert_single_finding(text, "appear in none of the round's files")

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

    def test_a_definition_of_done_may_show_the_gate_as_a_transcript(self) -> None:
        """The same gate written as a transcript is the same gate.

        Showing the command with its prompt and the green it produces is the
        most useful form of a done condition, and it was the one form the
        cross-check rejected: the prompt made it a different string, and the
        expected output counted as a second gate.
        """
        text = CONFORMANT_INJECTION.replace(
            "```\ncargo test -p target --lib some_gate\n```",
            "```console\n$ cargo test -p target --lib some_gate\n"
            "test result: ok. 12 passed; 0 failed\n```",
        )
        self.assertEqual(self.findings(text), [])

    def test_a_transcript_still_reports_a_gate_that_drifted(self) -> None:
        """The prompt must not become a way to smuggle a different command
        past the cross-check."""
        text = CONFORMANT_INJECTION.replace(
            "```\ncargo test -p target --lib some_gate\n```",
            "```console\n$ cargo test -p target --lib other_gate\n"
            "test result: ok. 12 passed; 0 failed\n```",
        )
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

    def test_prose_wrapped_before_a_number_is_not_a_recipe(self) -> None:
        """The reported false positive: a paragraph, not a list.

        Wrapping at column 79 puts a number at the head of a continuation line
        whenever a sentence breaks before one ending a clause. The rule then
        fired on a section holding no list at all, which teaches the author to
        reflow text and to skim the list the finding arrives in.
        """
        text = CONFORMANT_INJECTION.replace(
            "`strip_aw_marker_blocks` already knows the marker vocabulary; "
            "match it rather\nthan introducing a second notion of what AW owns.",
            "`strip_aw_marker_blocks` already knows the marker vocabulary, at "
            "`src/thing.rs`\nlines 12, 40, and\n2860. Match it rather than "
            "introducing a second notion of what AW owns.",
        )
        self.assertEqual(self.findings(text), [])

    def test_a_recipe_introduced_by_prose_is_still_reported(self) -> None:
        """A list interrupting a paragraph is a list; CommonMark says so too.

        The narrow reading of the fix -- "require a blank line above" -- would
        let the recipe back in under one line of preamble, which is how a
        section actually grows into one.
        """
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "Do this:\n1. Strip the marker blocks.\n2. Hash what is left.",
        )
        self.assert_single_finding(text, "reads as numbered steps")

    def test_a_loose_recipe_with_wrapped_steps_is_still_reported(self) -> None:
        """Blank and indented lines carry the run; only prose ends it.

        Otherwise a two-step recipe escapes by being written the way markdown
        renders best -- one blank line between items, continuations indented.
        """
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "1. Strip the marker blocks, leaving the body otherwise\n"
            "   untouched.\n\n2. Hash what is left.",
        )
        self.assert_single_finding(text, "reads as numbered steps")

    def test_one_item_is_not_an_order_to_type_it_in(self) -> None:
        """A list of one, and a wrapped number later in the same section.

        Both halves matter. One numbered line has no order to it, which is the
        whole of what the rule objects to. And the wrapped number after it is
        only separated from that item by prose -- so a rule that let the run
        survive a prose line would read the two as a two-step recipe spanning
        half the section.
        """
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "Exactly one thing must become true:\n\n"
            "1. Two bodies differing only inside a marker block produce the "
            "same digest.\n\n"
            "The rendered digest is a separate concern, as is the whitespace "
            "case at line\n2860. Neither changes here.",
        )
        self.assertEqual(self.findings(text), [])

    def test_consecutively_wrapped_numbers_are_not_a_recipe(self) -> None:
        """The one paragraph a run alone cannot tell from a list.

        Two sentences wrapping before a number in a row put two numbered lines
        adjacent with no prose between them, which is exactly the shape the run
        counts. What separates them from a recipe is that neither opens at one.
        """
        text = CONFORMANT_INJECTION.replace(
            "Two bodies differing only inside an AW-owned marker block must "
            "produce the\nsame stored contract digest.",
            "Two bodies differing only inside a marker block must agree, as at "
            "line\n1564. The same holds for bodies differing only in trailing "
            "whitespace at\n2860. Both are the stored digest, not the rendered "
            "one.",
        )
        self.assertEqual(self.findings(text), [])

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


class CaptureTest(unittest.TestCase):
    """A transcript costs a run, and the record is what the block is checked against."""

    def setUp(self) -> None:
        self.state = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.state, ignore_errors=True)
        self.root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.root, ignore_errors=True)
        self.profile = {
            "root": str(self.root),
            "state_dir": str(self.state),
            "task_contract": {
                "session_policy": "one-shot",
                "run_id": "r1",
                "intent": "measure what the binary prints",
            },
        }

    def capture(self, command: str, cwd: str | None = None) -> str:
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            agy_dispatch.capture(self.profile, "r1", command, cwd)
        return buffer.getvalue()

    def test_capture_records_what_the_command_printed(self) -> None:
        printed = self.capture("echo one; echo two")
        stored = json.loads(
            (self.state / "transcripts" / "r1.json").read_text()
        )
        self.assertEqual(len(stored), 1)
        self.assertEqual(stored[0]["command"], "echo one; echo two")
        self.assertEqual(stored[0]["output"], ["one", "two"])
        self.assertEqual(stored[0]["exit_code"], 0)
        self.assertEqual(stored[0]["cwd"], str(self.root))
        self.assertIn("```console\n$ echo one; echo two\none\ntwo\n```", printed)

    def test_a_failing_command_is_captured_with_its_exit_code(self) -> None:
        """Current behavior is often a refusal. A capture step that only records
        successes would push exactly those observations back into prose."""
        self.capture("echo boom >&2; exit 3")
        stored = json.loads(
            (self.state / "transcripts" / "r1.json").read_text()
        )
        self.assertEqual(stored[0]["exit_code"], 3)
        self.assertEqual(stored[0]["output"], ["boom"])

    def test_recapturing_a_command_replaces_its_record(self) -> None:
        """Otherwise the file accumulates every attempt and `load_captures` keys
        on the command, so which run the block is checked against would depend on
        dict ordering rather than on which one is current."""
        self.capture("echo first")
        (self.root / "marker").write_text("changed\n")
        self.capture("echo first")
        self.capture("echo second")
        stored = json.loads(
            (self.state / "transcripts" / "r1.json").read_text()
        )
        self.assertEqual(
            [record["command"] for record in stored],
            ["echo first", "echo second"],
        )

    def test_captures_load_keyed_by_command(self) -> None:
        self.capture("echo one")
        self.assertEqual(
            agy_dispatch.load_captures(self.profile, "r1"),
            {"echo one": ["one"]},
        )

    def test_a_round_that_captured_nothing_loads_an_empty_set(self) -> None:
        """A round whose current behavior is entirely a code quote captures
        nothing, and that is a complete round, not a broken one."""
        self.assertEqual(agy_dispatch.load_captures(self.profile, "r1"), {})

    def test_capture_runs_where_it_is_told(self) -> None:
        """The observation usually lives in a fixture outside the round root, and
        a capture taken in the wrong directory is the stale transcript again."""
        elsewhere = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, elsewhere, ignore_errors=True)
        self.capture("pwd", cwd=str(elsewhere))
        stored = json.loads(
            (self.state / "transcripts" / "r1.json").read_text()
        )
        self.assertEqual(stored[0]["cwd"], str(elsewhere))
        self.assertEqual(
            stored[0]["output"], [str(Path(elsewhere).resolve())]
        )


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
                # A closed identity, because `scaffold` now refuses a key the
                # contract forbids -- the form it hands out has to be the one
                # the rest of the round opens.
                "session_policy": "one-shot",
                "run_id": "r1",
                "intent": "exercise the blank form",
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

    def lint(self, task_key: str) -> str:
        buf = io.StringIO()
        try:
            with contextlib.redirect_stdout(buf):
                agy_dispatch.lint(self.profile, task_key)
        except SystemExit as exit_code:
            # `lint` exits on findings, which is not what these rows are about.
            if not isinstance(exit_code.code, int):
                raise
        return buf.getvalue()

    def test_lint_refuses_a_key_the_contract_identity_forbids(self) -> None:
        """`lint` is the pre-dispatch gate, so it linted a different pair.

        Observed on the #3448 round: documents authored as
        `3448-emitted-obligations-must-run.md` against a contract carrying
        `issue: 3448` linted `findings: none`, and `snapshot` then refused the
        same key. The documents that passed the gate were not the documents any
        later verb would open.
        """
        self.scaffold()
        with self.assertRaises(SystemExit) as caught:
            self.lint("r1-emitted-obligations-must-run")
        # Both halves of the refusal, spelled out rather than probed for `r1`:
        # the identity is a substring of the key that was asked for, so a
        # message that dropped either name would still satisfy a loose `assertIn`.
        # And an unguarded `lint` also raises -- on the oracle it cannot find --
        # so `assertRaises` alone does not say the guard is what fired.
        self.assertIn("task identity r1 does not match", str(caught.exception))
        self.assertIn(
            "requested key=r1-emitted-obligations-must-run",
            str(caught.exception),
        )

    def test_scaffold_refuses_a_key_no_later_verb_will_open(self) -> None:
        """One step earlier: `scaffold` is what creates the mis-keyed pair.

        Refusing only at `lint` still hands the controller a form to author at a
        path this round never opens, and authoring is the expensive part.
        """
        with self.assertRaises(SystemExit) as caught:
            with contextlib.redirect_stdout(io.StringIO()):
                agy_dispatch.scaffold(self.profile, "r1-with-a-description")
        self.assertIn("task identity r1 does not match", str(caught.exception))
        self.assertIn(
            "requested key=r1-with-a-description", str(caught.exception)
        )
        self.assertFalse(
            (self.state / "oracles" / "r1-with-a-description.md").exists()
        )

    def test_lint_still_reads_the_documents_under_the_identity(self) -> None:
        """The negative control: the key the contract fixes still lints.

        A guard that refused every key would take the pre-dispatch gate out of
        the flow entirely, which is worse than the defect it replaces.
        """
        self.scaffold()
        (self.state / "oracles" / "r1.md").write_text(CONFORMANT_ORACLE)
        output = self.lint("r1")
        self.assertIn(str(self.state / "oracles" / "r1.md"), output)

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


class AdjudicateScopeFindingTest(unittest.TestCase):
    """Scope finding adjudication lifecycle tests (measurements 1-9)."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/tmp")
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-b", "main"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test"], cwd=self.repo, check=True)
        subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=self.repo, check=True)

        self.design_file = self.repo / "design.md"
        self.design_file.write_text("frozen design input content\n")

        self.protected1 = self.repo / "src" / "protected1.py"
        self.protected1.parent.mkdir(parents=True, exist_ok=True)
        self.protected1.write_text("protected1 base content\n")

        self.protected2 = self.repo / "src" / "protected2.py"
        self.protected2.write_text("protected2 base content\n")

        self.writable = self.repo / "src" / "writable.py"
        self.writable.write_text("writable base content\n")

        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-m", "initial commit"], cwd=self.repo, check=True)

        self.project_dir = self.root / "projects"
        self.project_dir.mkdir()
        self.project_id = "test-project"
        (self.project_dir / f"{self.project_id}.json").write_text(
            json.dumps(
                {
                    "id": self.project_id,
                    "name": self.project_id,
                    "projectResources": {
                        "resources": [{"gitFolder": {"folderUri": self.repo.resolve().as_uri()}}]
                    },
                    "permissionGrants": {
                        "permissionGrants": {
                            "allow": ["command(*)"],
                            "deny": [],
                            "ask": [],
                        }
                    },
                }
            )
        )

        agy_dispatch.PROJECT_DIR = self.project_dir
        agy_dispatch.SETTINGS = self.root / "settings.json"
        agy_dispatch.GLOBAL = self.root / "config.json"
        agy_dispatch.SETTINGS.write_text(json.dumps({"permissions": {"allow": [], "deny": [], "ask": []}}))
        agy_dispatch.GLOBAL.write_text(
            json.dumps({"userSettings": {"globalPermissionGrants": {"allow": [], "deny": [], "ask": []}}})
        )

        self.gate_cmd = "grep -q 'good edit' src/writable.py"

        self.state_dir = self.root / "state"
        self.profile_dict = {
            "root": str(self.repo),
            "repo": "owner/repo",
            "agy_project_id": self.project_id,
            "state_dir": str(self.state_dir),
            "mode": "bounded-write",
            "task_contract": {
                "kind": "implementation",
                "session_policy": "one-shot",
                "run_id": "r1",
                "intent": "Test adjudication",
                "gate_command": self.gate_cmd,
                "design_inputs": [{"path": "design.md", "sha256": agy_dispatch.sha256(self.design_file)}],
            },
            "project_permissions": {
                "allow": ["command(*)"],
                "deny": [],
                "ask": [],
                "require_empty_global": True,
            },
            "task_commands": {
                "allow": [self.gate_cmd],
                "deny": [],
            },
            "protected_artifacts": [
                {"path": "src/protected1.py", "sha256": agy_dispatch.sha256(self.protected1)},
                {"path": "src/protected2.py", "sha256": agy_dispatch.sha256(self.protected2)},
            ],
            "snapshot_paths": ["src"],
            "allowed_repo_writes": ["src/writable.py"],
            "path_change_budgets": {},
            "controller_root": str(self.repo),
        }

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def setup_round(self, task_key: str) -> tuple[dict, Path]:
        p_dict = dict(self.profile_dict)
        p_dict["task_contract"] = dict(self.profile_dict["task_contract"])
        p_dict["task_contract"]["run_id"] = task_key
        p_dict["state_dir"] = str(self.root / f"state-{task_key}")
        p_path = self.root / f"profile-{task_key}.json"
        p_path.write_text(json.dumps(p_dict, indent=2))

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.worktree(str(p_path), task_key)
        profile = agy_dispatch.load_profile(str(p_path), validate_design=False)

        oracle = Path(profile["state_dir"]) / "oracles" / f"{task_key}.md"
        injection = Path(profile["state_dir"]) / "injections" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True, exist_ok=True)
        injection.parent.mkdir(parents=True, exist_ok=True)
        oracle.write_text(
            f"## Claim\n\nclaim\n\n## Measurements\n\n| # | input | expected observation | why |\n|---|---|---|---|\n| 1 | x | y | z |\n| 2 | x (negative control) | FAIL | z |\n\n## Gate\n\n```\n{self.gate_cmd}\n```\n\n## Scope\n\n| Path | Line budget | Line ranges |\n|---|---|---|\n| src/writable.py | none | any |\n\n## Fabrication tells\n\n- tell\n"
        )
        injection.write_text(
            f"## Task\n\ntest task\n\n## Current behavior\n\n```\nwritable base content\n```\n\n## Required change\n\n- change\n\n## Shape to follow\n\n`src/writable.py`\n\n## Reference\n\n| path | why |\n|---|---|\n| `src/writable.py` | context |\n\n## Out of scope\n\n- none\n\n## Definition of done\n\n`src/writable.py`\n\n```\n{self.gate_cmd}\n```\n"
        )
        profile["inject_prompt_file"] = str(injection)
        p_path.write_text(json.dumps(profile, indent=2))
        profile = agy_dispatch.load_profile(str(p_path), validate_design=False)

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.snapshot(profile, task_key)
        return profile, p_path

    def run_proofs(self, profile: dict, task_key: str, worker_root: Path) -> None:
        """Helper to run valid mutant and candidate proofs for a round."""
        good_text = (worker_root / "src" / "writable.py").read_text()

        # Mutant: revert product edit
        (worker_root / "src" / "writable.py").write_text("writable base content\n")
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.prove(profile, task_key, "mutant")

        # Candidate: restore product edit
        (worker_root / "src" / "writable.py").write_text(good_text)
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.prove(profile, task_key, "candidate")

    def test_measurement_1_verify_reports_protected_artifact_finding_and_exits_2(
        self,
    ) -> None:
        """Row 1: verify reports protected artifact scope finding and exits 2."""
        profile, _ = self.setup_round("m1")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("modified by worker\n")

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.verify(profile, "m1")
        self.assertEqual(caught.exception.code, agy_dispatch.EXIT_FINDINGS)

    def test_measurement_2_and_3_adjudicated_finding_unblocks_accept_prove_sweep(
        self,
    ) -> None:
        """Rows 2 & 3: adjudicating a finding allows accept, prove, and sweep."""
        profile, _ = self.setup_round("m23")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("worker edit protected1\n")
        (worker_root / "src" / "writable.py").write_text("good edit\n")

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.verify(profile, "m23")
        self.assertEqual(caught.exception.code, agy_dispatch.EXIT_FINDINGS)

        # Adjudicate finding
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(
                profile, "m23", "admit", "protected artifact changed: src/protected1.py", "reason for admit"
            )

        # verify should now pass cleanly
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.verify(profile, "m23")

        # Prove mutant & candidate
        self.run_proofs(profile, "m23", worker_root)

        # sweep
        sweep_script = self.root / "sweep.py"
        sweep_script.write_text("import sys\nsys.exit(0)\n")
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.sweep(profile, "m23", str(sweep_script))

        # accept
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.accept(profile, "m23")

        head_commit = agy_dispatch.git_output(worker_root, "log", "-1", "--name-only")
        self.assertIn("accepted worker candidate", head_commit)
        self.assertIn("src/protected1.py", head_commit)
        self.assertIn("src/writable.py", head_commit)

    def test_measurement_4_second_unadjudicated_artifact_blocks_verbs(self) -> None:
        """Row 4: second unadjudicated artifact still blocks prove, sweep, accept."""
        profile, p_path = self.setup_round("m4")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("same edit\n")
        (worker_root / "src" / "protected2.py").write_text("same edit\n")

        # Adjudicate protected1 only
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "m4", "admit", "src/protected1.py", "reason for admit")

        # load_profile with validate_design=True should refuse naming protected2
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(p_path), validate_design=True, task_key="m4")
        self.assertIn("protected2.py", str(caught.exception))
        self.assertNotIn("protected1.py", str(caught.exception))

    def test_measurement_4_byte_identical_unadjudicated_artifact_blocks_verbs(self) -> None:
        """Row 4: byte-identical unadjudicated artifact still blocks prove, sweep, accept."""
        profile, p_path = self.setup_round("m4_byte_identical")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("byte identical edit\n")
        (worker_root / "src" / "protected2.py").write_text("byte identical edit\n")

        # Adjudicate protected1 only
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(
                profile, "m4_byte_identical", "admit", "src/protected1.py", "reason for admit"
            )

        # load_profile with validate_design=True should refuse naming protected2
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(
                str(p_path), validate_design=True, task_key="m4_byte_identical"
            )
        self.assertIn("protected2.py", str(caught.exception))
        self.assertNotIn("protected1.py", str(caught.exception))

    def test_measurement_5_adjudicated_artifact_changed_again_refuses(self) -> None:
        """Row 5: artifact changed again after decision is refused."""
        profile, p_path = self.setup_round("m5")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("first edit\n")

        # Adjudicate first edit
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "m5", "admit", "src/protected1.py", "reason for admit")

        # Change artifact again
        (worker_root / "src" / "protected1.py").write_text("second edit\n")

        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.load_profile(str(p_path), validate_design=True, task_key="m5")
        self.assertIn("protected1.py", str(caught.exception))

    def test_measurement_6_adjudication_on_nonexistent_finding_refused(self) -> None:
        """Row 6: adjudication recorded on round with no such finding is refused."""
        profile, _ = self.setup_round("m6")
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.adjudicate(profile, "m6", "admit", "src/protected1.py", "reason for admit")
        self.assertIn("no such finding exists", str(caught.exception))

    def test_measurement_7_rejected_decision_restores_artifact(self) -> None:
        """Row 7: decision reject restores artifact to frozen snapshot content."""
        profile, _ = self.setup_round("m7")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("bad edit\n")
        (worker_root / "src" / "writable.py").write_text("good edit\n")

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "m7", "reject", "src/protected1.py", "reason for reject")

        # Check content is restored
        self.assertEqual(
            (worker_root / "src" / "protected1.py").read_text(),
            "protected1 base content\n",
        )

        # Prove and accept
        self.run_proofs(profile, "m7", worker_root)

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.accept(profile, "m7")

        head_commit = agy_dispatch.git_output(worker_root, "log", "-1", "--name-only")
        self.assertIn("src/writable.py", head_commit)
        self.assertNotIn("src/protected1.py", head_commit)

    def test_measurement_8_admitted_decision_carried_in_commit(self) -> None:
        """Row 8: decision admit carries artifact in accepted commit."""
        profile, _ = self.setup_round("m8")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("admitted edit\n")
        (worker_root / "src" / "writable.py").write_text("good edit\n")

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "m8", "admit", "src/protected1.py", "reason for admit")

        self.run_proofs(profile, "m8", worker_root)

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.accept(profile, "m8")

        head_commit = agy_dispatch.git_output(worker_root, "log", "-1", "--name-only")
        self.assertIn("src/protected1.py", head_commit)

    def test_measurement_9_verify_rerun_removes_decided_finding(self) -> None:
        """Row 9: verify re-run removes decided finding while keeping others unchanged."""
        profile, _ = self.setup_round("m9")
        worker_root = Path(profile["root"])
        (worker_root / "src" / "protected1.py").write_text("edit 1\n")
        (worker_root / "src" / "protected2.py").write_text("edit 2\n")

        # Initial verify output
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            with self.assertRaises(SystemExit):
                agy_dispatch.verify(profile, "m9")
        out1 = buf.getvalue()
        target_line_p1 = [line for line in out1.splitlines() if "protected artifact changed:" in line and "protected1.py" in line][0]
        target_line_p2 = [line for line in out1.splitlines() if "protected artifact changed:" in line and "protected2.py" in line][0]

        # Adjudicate protected1
        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "m9", "admit", "src/protected1.py", "reason for admit")

        # Re-run verify
        buf2 = io.StringIO()
        with contextlib.redirect_stdout(buf2):
            with self.assertRaises(SystemExit):
                agy_dispatch.verify(profile, "m9")
        out2 = buf2.getvalue()
        self.assertNotIn(target_line_p1, out2)
        self.assertIn(target_line_p2, out2)


class SubfileLineRangesTest(unittest.TestCase):
    """Sub-file line range bound tests (Acceptance criteria 1 & 2)."""

    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(dir="/tmp")
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-b", "main"], cwd=self.repo, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "Test"], cwd=self.repo, check=True)
        subprocess.run(["git", "config", "user.email", "test@test.com"], cwd=self.repo, check=True)

        self.design_file = self.repo / "design.md"
        self.design_file.write_text("frozen design input\n")

        self.writable = self.repo / "src" / "writable.py"
        self.writable.parent.mkdir(parents=True, exist_ok=True)
        lines = [f"def func_{i}():\n    return {i}\n" for i in range(1, 15)]
        self.writable.write_text("".join(lines))

        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        subprocess.run(["git", "commit", "-m", "initial commit"], cwd=self.repo, check=True)

        self.project_dir = self.root / "projects"
        self.project_dir.mkdir()
        self.project_id = "test-project-ranges"
        (self.project_dir / f"{self.project_id}.json").write_text(
            json.dumps(
                {
                    "id": self.project_id,
                    "name": self.project_id,
                    "projectResources": {
                        "resources": [{"gitFolder": {"folderUri": self.repo.resolve().as_uri()}}]
                    },
                    "permissionGrants": {
                        "permissionGrants": {
                            "allow": ["command(*)"],
                            "deny": [],
                            "ask": [],
                        }
                    },
                }
            )
        )

        agy_dispatch.PROJECT_DIR = self.project_dir
        agy_dispatch.SETTINGS = self.root / "settings.json"
        agy_dispatch.GLOBAL = self.root / "config.json"
        agy_dispatch.SETTINGS.write_text(json.dumps({"permissions": {"allow": [], "deny": [], "ask": []}}))
        agy_dispatch.GLOBAL.write_text(
            json.dumps({"userSettings": {"globalPermissionGrants": {"allow": [], "deny": [], "ask": []}}})
        )

        self.gate_cmd = "grep -q 'good edit' src/writable.py"

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def setup_range_round(self, task_key: str, range_spec: str) -> tuple[dict, Path]:
        state_dir = self.root / f"state-{task_key}"
        p_dict = {
            "root": str(self.repo),
            "repo": "owner/repo",
            "agy_project_id": self.project_id,
            "state_dir": str(state_dir),
            "mode": "bounded-write",
            "task_contract": {
                "kind": "implementation",
                "session_policy": "one-shot",
                "run_id": task_key,
                "intent": "Test line ranges",
                "gate_command": self.gate_cmd,
                "design_inputs": [{"path": "design.md", "sha256": agy_dispatch.sha256(self.design_file)}],
            },
            "project_permissions": {
                "allow": ["command(*)"],
                "deny": [],
                "ask": [],
                "require_empty_global": True,
            },
            "task_commands": {
                "allow": [self.gate_cmd],
                "deny": [],
            },
            "protected_artifacts": [],
            "snapshot_paths": ["src"],
            "allowed_repo_writes": ["src/writable.py"],
            "path_change_budgets": {},
            "path_line_ranges": {"src/writable.py": range_spec},
            "controller_root": str(self.repo),
        }
        p_path = self.root / f"profile-{task_key}.json"
        p_path.write_text(json.dumps(p_dict, indent=2))

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.worktree(str(p_path), task_key)
        profile = agy_dispatch.load_profile(str(p_path), validate_design=False)

        oracle = Path(profile["state_dir"]) / "oracles" / f"{task_key}.md"
        injection = Path(profile["state_dir"]) / "injections" / f"{task_key}.md"
        oracle.parent.mkdir(parents=True, exist_ok=True)
        injection.parent.mkdir(parents=True, exist_ok=True)
        oracle.write_text(
            f"## Claim\n\nclaim\n\n## Measurements\n\n| # | input | expected observation | why |\n|---|---|---|---|\n| 1 | x | y | z |\n| 2 | x (negative control) | FAIL | z |\n\n## Gate\n\n```\n{self.gate_cmd}\n```\n\n## Scope\n\n| Path | Line budget | Line ranges |\n|---|---|---|\n| src/writable.py | none | {range_spec} |\n\n## Fabrication tells\n\n- tell\n"
        )
        injection.write_text(
            f"## Task\n\ntest task\n\n## Current behavior\n\n```\ndef func_1():\n```\n\n## Required change\n\n- change\n\n## Shape to follow\n\n`src/writable.py`\n\n## Reference\n\n| path | why |\n|---|---|\n| `src/writable.py` | context |\n\n## Out of scope\n\n- none\n\n## Definition of done\n\n`src/writable.py`\n\n```\n{self.gate_cmd}\n```\n"
        )
        profile["inject_prompt_file"] = str(injection)
        p_path.write_text(json.dumps(profile, indent=2))
        profile = agy_dispatch.load_profile(str(p_path), validate_design=False)

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.snapshot(profile, task_key)
        return profile, p_path

    def test_in_range_candidate_yields_no_finding(self) -> None:
        """Candidate whose diff falls entirely inside declared range yields no finding."""
        profile, _ = self.setup_range_round("r_in_range", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        lines[0] = "def func_1_edited(): # good edit\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        findings = agy_dispatch.scope_findings(profile, ["src/writable.py"], "r_in_range")
        self.assertEqual(findings, [])

    def test_extra_hunk_outside_range_yields_finding(self) -> None:
        """The same candidate with one extra hunk outside declared range yields exactly one finding."""
        profile, _ = self.setup_range_round("r_extra_hunk", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        lines[0] = "def func_1_edited(): # good edit\n"
        lines[24] = "def func_13_edited():\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        findings = agy_dispatch.scope_findings(profile, ["src/writable.py"], "r_extra_hunk")
        expected = "src/writable.py: hunk at baseline lines 25 falls outside declared ranges"
        self.assertEqual(findings, [expected])

    def test_hunk_starting_inside_and_running_past_end_yields_finding(self) -> None:
        """Hunk starting inside declared range and running past its end yields finding (Negative Control test target)."""
        profile, _ = self.setup_range_round("r_past_end", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        for i in range(4, 15):
            lines[i] = f"# edit {i}\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        findings = agy_dispatch.scope_findings(profile, ["src/writable.py"], "r_past_end")
        expected = "src/writable.py: hunk at baseline lines 5-15 falls outside declared ranges"
        self.assertEqual(findings, [expected])

    def test_second_out_of_range_hunk_yields_second_finding_while_first_admission_matches(self) -> None:
        """Second out-of-range hunk yields second finding while admission for first still matches."""
        profile, _ = self.setup_range_round("r_two_hunks", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        lines[14] = "# edit 15\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        finding1 = "src/writable.py: hunk at baseline lines 15 falls outside declared ranges"
        findings1 = agy_dispatch.scope_findings(profile, ["src/writable.py"], "r_two_hunks")
        self.assertEqual(findings1, [finding1])

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "r_two_hunks", "admit", finding1, "admitting line 15 edit")

        lines[24] = "# edit 25\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        findings2 = agy_dispatch.scope_findings(profile, ["src/writable.py"], "r_two_hunks")
        self.assertEqual(len(findings2), 2)
        self.assertIn(finding1, findings2)

        open_, admitted, rejected = agy_dispatch.split_scope_findings(profile, "r_two_hunks", findings2)
        self.assertEqual(admitted, [finding1])
        self.assertEqual(len(open_), 1)
        self.assertNotIn(finding1, open_)

    def test_adjudicate_refuses_empty_reason(self) -> None:
        """adjudicate refuses an empty or whitespace-only reason."""
        profile, _ = self.setup_range_round("r_empty_reason", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        lines[20] = "# edit 21\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        finding = "src/writable.py: hunk at baseline lines 21 falls outside declared ranges"
        with self.assertRaises(SystemExit) as caught:
            agy_dispatch.adjudicate(profile, "r_empty_reason", "admit", finding, "   ")
        self.assertIn("reason", str(caught.exception))

    def test_verify_and_review_print_recorded_reason(self) -> None:
        """verify and review print recorded reason beside each admitted finding."""
        profile, _ = self.setup_range_round("r_reason_print", "1-10")
        worker_root = Path(profile["root"])
        lines = (worker_root / "src" / "writable.py").read_text().splitlines(keepends=True)
        lines[0] = "def func_1_edited(): # good edit\n"
        lines[20] = "# edit 21\n"
        (worker_root / "src" / "writable.py").write_text("".join(lines))

        finding = "src/writable.py: hunk at baseline lines 21 falls outside declared ranges"
        reason_text = "admitting line 21 refactor"

        with contextlib.redirect_stdout(io.StringIO()):
            agy_dispatch.adjudicate(profile, "r_reason_print", "admit", finding, reason_text)

        buf_verify = io.StringIO()
        with contextlib.redirect_stdout(buf_verify):
            agy_dispatch.verify(profile, "r_reason_print")
        out_verify = buf_verify.getvalue()
        self.assertIn(finding, out_verify)
        self.assertIn(reason_text, out_verify)

        buf_review = io.StringIO()
        with contextlib.redirect_stdout(buf_review):
            agy_dispatch.review(profile, "r_reason_print")
        out_review = buf_review.getvalue()
        self.assertIn(finding, out_review)
        self.assertIn(reason_text, out_review)


class CoreExtractionTest(unittest.TestCase):
    MOVED_FUNCTIONS = (
        "task_session_policy",
        "revision_origin",
        "validate_task_identity",
        "validate_task_key",
        "split_rule_tokens",
        "rule_matches",
        "command_rule_matches",
        "task_allowlist_families",
        "task_allowlist_admits",
        "parse_line_ranges",
        "extract_exec_report",
        "reads_as_numbered_steps",
        "oracle_sections",
        "missing_or_misordered",
        "gate_commands_in",
        "unjudged_gate_commands",
        "unquoted_current_behavior_lines",
        "referenced_paths",
        "transcript_body",
        "transcript_findings",
        "document_findings",
        "marks_a_negative_control",
        "oracle_findings",
        "injection_findings",
        "injection_path",
        "oracle_path",
        "blank_round_forms",
        "scaffold",
        "round_findings",
        "lint",
        "capture_path",
        "load_captures",
    )

    def test_no_moved_function_defined_in_adapter(self) -> None:
        adapter_path = Path(agy_dispatch.__file__).resolve()
        tree = ast.parse(adapter_path.read_text())
        top_level_defs = {
            node.name
            for node in tree.body
            if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        }
        for name in self.MOVED_FUNCTIONS:
            self.assertNotIn(
                name,
                top_level_defs,
                f"Moved function {name} is still defined as a top-level FunctionDef in agy_dispatch.py",
            )

    def test_moved_functions_defined_exactly_once_in_core(self) -> None:
        core_dir = Path(__file__).resolve().parents[3] / "dispatch" / "core"

        counts: dict[str, int] = {name: 0 for name in self.MOVED_FUNCTIONS}
        for py_file in core_dir.glob("*.py"):
            tree = ast.parse(py_file.read_text())
            for node in tree.body:
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    if node.name in counts:
                        counts[node.name] += 1
        for name in self.MOVED_FUNCTIONS:
            self.assertEqual(
                counts[name],
                1,
                f"Function {name} is defined {counts[name]} times under dispatch.core (expected 1)",
            )

    def test_reexported_attributes_resolve_on_adapter(self) -> None:
        all_expected = self.MOVED_FUNCTIONS + (
            "EXIT_FINDINGS",
            "INJECTION_SECTIONS",
            "TASK_KEY_PATTERN",
        )
        for name in all_expected:
            self.assertTrue(
                hasattr(agy_dispatch, name),
                f"Attribute {name} not found on agy_dispatch module",
            )

    def test_adapter_reexports_are_identical_objects_from_core(self) -> None:
        import dispatch.core.cli
        import dispatch.core.documents
        import dispatch.core.identity
        import dispatch.core.rules
        import dispatch.core.scope

        core_modules = (
            dispatch.core.cli,
            dispatch.core.documents,
            dispatch.core.identity,
            dispatch.core.rules,
            dispatch.core.scope,
        )
        for name in self.MOVED_FUNCTIONS:
            adapter_obj = getattr(agy_dispatch, name)
            core_obj = None
            for mod in core_modules:
                if hasattr(mod, name):
                    core_obj = getattr(mod, name)
                    break
            self.assertIsNotNone(
                core_obj,
                f"Function {name} not found in any dispatch.core module",
            )
            self.assertIs(
                adapter_obj,
                core_obj,
                f"agy_dispatch.{name} is not identical to core attribute",
            )


if __name__ == "__main__":
    unittest.main()

