#!/usr/bin/env python3
from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
import unittest
from contextlib import ExitStack, contextmanager
from pathlib import Path
from unittest.mock import patch


SCRIPTS = Path(__file__).parents[2] / "scripts"
sys.path.insert(0, str(SCRIPTS))
SCRIPT = SCRIPTS / "execute_assignment.py"
SPEC = importlib.util.spec_from_file_location("execute_assignment", SCRIPT)
assert SPEC and SPEC.loader
execute = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(execute)


class ExecuteAssignmentTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.base = Path(self.temp.name)
        self.project = self.base / "project"
        self.worktree = self.base / "worker"
        self.project.mkdir()
        subprocess.run(["git", "init", "-q"], cwd=self.project, check=True)
        subprocess.run(
            ["git", "config", "user.email", "test@example.invalid"],
            cwd=self.project,
            check=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"],
            cwd=self.project,
            check=True,
        )
        (self.project / "README.md").write_text("base\n")
        subprocess.run(["git", "add", "README.md"], cwd=self.project, check=True)
        subprocess.run(["git", "commit", "-qm", "base"], cwd=self.project, check=True)
        subprocess.run(
            ["git", "worktree", "add", "-q", "--detach", str(self.worktree), "HEAD"],
            cwd=self.project,
            check=True,
        )
        self.oracle = self.base / "oracle.md"
        self.oracle.write_text("check the bounded result\n")
        self.assignment_path = self.base / "assignment.json"
        self.registry_path = self.base / "agy-projects.json"
        self.cache_path = self.base / "projects.json"
        self.project_config_dir = self.base / "config" / "projects"
        self.project_config_dir.mkdir(parents=True)
        self.state_root = self.base / "state"
        self.bin_dir = self.base / "bin"
        self.bin_dir.mkdir()
        self.agy = self.bin_dir / "agy"
        self.write_fake_agy_version("agy version 1.2.3")
        self.write_assignment()
        self.write_registry()
        self.write_project_config()
        self.write_cache()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def assignment(self) -> dict:
        return {
            "schema": execute.ASSIGNMENT_SCHEMA,
            "task_key": "T-1",
            "worktree_root": str(self.worktree),
            "repository": "owner/repo",
            "executor_role": "qa",
            "mode": "measure-only",
            "task_contract": {
                "kind": "measurement",
                "session_policy": "one-shot",
                "run_id": "T-1",
                "intent": "measure",
                "design_inputs": [],
            },
            "oracle": {
                "path": str(self.oracle),
                "sha256": hashlib.sha256(self.oracle.read_bytes()).hexdigest(),
            },
            "task_commands": {"allow": ["pwd"], "deny": ["git push"]},
            "protected_artifacts": [],
            "snapshot_paths": ["README.md"],
            "allowed_repo_writes": [],
            "path_change_budgets": {},
            "external_payload_consent": {
                "destination": "agy-headless",
                "approved": True,
                "approval_source": "explicit_user_after_risk_disclosure",
                "approval_record": "approved",
                "approved_payload_classes": [
                    "task_contract",
                    "oracle",
                    "repository_read_context",
                ],
            },
        }

    def registry_entry(self, *, project_id: str = "project-test") -> dict:
        common = execute.worktree_identity(self.worktree)[1]
        return {
            "git_common_dir": str(common),
            "backend": "agy-cli",
            "agy_project_root": str(self.project),
            "global_permissions": {"allow": [], "deny": [], "ask": []},
            "project_policy_observation": {
                "source": "official_cli_permissions",
                "project_id": project_id,
                "project_root": str(self.project),
                "observed_at": "2026-09-15T00:00:00Z",
                "permissions": {"allow": [], "deny": [], "ask": []},
            },
        }

    def write_assignment(self, value: dict | None = None) -> None:
        self.assignment_path.write_text(json.dumps(value or self.assignment()))

    def write_registry(self, entries: list[dict] | None = None) -> None:
        self.registry_path.write_text(
            json.dumps(
                {
                    "schema": execute.REGISTRY_SCHEMA,
                    "projects": [self.registry_entry()] if entries is None else entries,
                }
            )
        )

    def write_cache(self, value: object | None = None) -> None:
        if value is None:
            value = {}
        self.cache_path.write_text(json.dumps(value))

    def project_config(
        self,
        *,
        project_id: str = "project-test",
        resources: list[dict] | None = None,
    ) -> dict:
        return {
            "id": project_id,
            "name": "Axiom test Project",
            "workspace": {
                "resources": (
                    [{"folderUri": self.project.resolve().as_uri()}]
                    if resources is None
                    else resources
                )
            },
        }

    def write_project_config(
        self,
        value: object | None = None,
        *,
        project_id: str = "project-test",
    ) -> Path:
        path = self.project_config_dir / f"{project_id}.json"
        if value is None:
            value = self.project_config(project_id=project_id)
        path.write_text(json.dumps(value))
        return path

    def write_fake_agy_version(self, version: str) -> None:
        self.agy.write_text(f"#!/bin/sh\nprintf '%s\\n' '{version}'\n")
        self.agy.chmod(0o755)

    @contextmanager
    def with_paths(self):
        with ExitStack() as stack:
            stack.enter_context(patch.object(execute, "REGISTRY_PATH", self.registry_path))
            stack.enter_context(patch.object(execute, "STATE_ROOT", self.state_root.resolve()))
            stack.enter_context(
                patch.object(
                    execute,
                    "AGY_PROJECTS_CACHE_PATH",
                    self.cache_path,
                    create=True,
                )
            )
            stack.enter_context(
                patch.object(
                    execute,
                    "AGY_PROJECT_CONFIG_DIR",
                    self.project_config_dir,
                    create=True,
                )
            )
            stack.enter_context(
                patch.dict(
                    os.environ,
                    {"PATH": f"{self.bin_dir}{os.pathsep}{os.environ['PATH']}"},
                )
            )
            yield

    def loaded_assignment(self) -> tuple[dict, str]:
        _, assignment, digest = execute.load_assignment(str(self.assignment_path))
        return assignment, digest

    def resolved_registry(self) -> dict:
        common = execute.worktree_identity(self.worktree)[1]
        return execute.resolve_registry(common, self.worktree)

    def materialize(self) -> tuple[Path, dict]:
        assignment, digest = self.loaded_assignment()
        path = execute.materialize_profile(
            assignment,
            digest,
            self.worktree,
            self.resolved_registry(),
        )
        return path, json.loads(path.read_text())

    def test_v2_materializes_config_resolved_role_pair_and_oracle(self) -> None:
        self.assertEqual(execute.ASSIGNMENT_SCHEMA, "execution-assignment-v2")
        self.assertEqual(execute.REGISTRY_SCHEMA, "execution-agy-registry-v2")
        with self.with_paths():
            profile_path, profile = self.materialize()
        self.assertEqual(profile["model"], "gemini-3.8-flash-high")
        self.assertEqual(profile["effort"], "high")
        self.assertTrue(profile["sandbox"])
        self.assertEqual(
            (profile_path.parent / "oracles" / "T-1.md").read_text(),
            self.oracle.read_text(),
        )
        self.assertEqual(
            set(profile["backend_resolution"]),
            {
                "backend",
                "persistent_root",
                "project_id",
                "project_config_digest",
                "workspace_cache_observation_digest",
                "registry_entry_digest",
                "agy_version",
            },
        )
        self.assertEqual(profile["backend_resolution"]["backend"], "agy-cli")
        self.assertEqual(
            profile["backend_resolution"]["persistent_root"],
            str(self.project.resolve()),
        )
        self.assertEqual(profile["backend_resolution"]["project_id"], "project-test")
        self.assertEqual(profile["backend_resolution"]["agy_version"], "agy version 1.2.3")

    def test_dev_derives_medium_pair(self) -> None:
        value = self.assignment()
        value["executor_role"] = "dev"
        self.write_assignment(value)
        with self.with_paths():
            _, profile = self.materialize()
        self.assertEqual(
            (profile["model"], profile["effort"]),
            ("gemini-3.8-flash-medium", "medium"),
        )

    def test_assignment_rejects_sandbox_and_all_backend_overrides(self) -> None:
        for key, value in (
            ("sandbox", True),
            ("model", "override"),
            ("agy_project_id", "project-test"),
            ("permissions", {}),
        ):
            with self.subTest(key=key):
                assignment = self.assignment()
                assignment[key] = value
                self.write_assignment(assignment)
                with self.assertRaisesRegex(SystemExit, f"{key}|backend"):
                    execute.load_assignment(str(self.assignment_path))

    def test_assignment_rejects_non_string_timeout_before_launch(self) -> None:
        assignment = self.assignment()
        assignment["timeout"] = 300
        self.write_assignment(assignment)
        with self.assertRaisesRegex(SystemExit, "timeout.*duration string"):
            execute.load_assignment(str(self.assignment_path))

    def test_rejects_bad_oracle_digest(self) -> None:
        value = self.assignment()
        value["oracle"]["sha256"] = "0" * 64
        self.write_assignment(value)
        with self.with_paths(), self.assertRaisesRegex(SystemExit, "oracle digest"):
            assignment, digest = self.loaded_assignment()
            execute.materialize_profile(
                assignment,
                digest,
                self.worktree,
                self.resolved_registry(),
            )

    def test_rejects_wrong_cwd_and_persistent_root(self) -> None:
        assignment, _ = self.loaded_assignment()
        old = Path.cwd()
        os.chdir(self.base)
        try:
            with self.assertRaisesRegex(SystemExit, "cwd"):
                execute.validate_executor_worktree(assignment)
        finally:
            os.chdir(old)
        assignment["worktree_root"] = str(self.project)
        os.chdir(self.project)
        try:
            with self.assertRaisesRegex(SystemExit, "persistent"):
                execute.validate_executor_worktree(assignment)
        finally:
            os.chdir(old)

    def test_assignment_must_be_outside_every_repository_worktree(self) -> None:
        common = execute.worktree_identity(self.worktree)[1]
        execute.validate_assignment_location(
            self.assignment_path.resolve(), self.worktree.resolve(), common
        )
        for repository_root in (self.project, self.worktree):
            with self.subTest(repository_root=repository_root):
                inside = repository_root / "controller-assignment.json"
                inside.write_text(self.assignment_path.read_text())
                with self.assertRaisesRegex(SystemExit, "outside every repository"):
                    execute.validate_assignment_location(
                        inside.resolve(), self.worktree.resolve(), common
                    )
        symlink = self.worktree / "controller-assignment-link.json"
        symlink.symlink_to(self.assignment_path)
        with self.assertRaisesRegex(SystemExit, "outside every repository"):
            execute.validate_assignment_location(
                symlink, self.worktree.resolve(), common
            )

    def test_observed_project_id_resolves_config_when_cache_is_absent_or_empty(self) -> None:
        with self.with_paths():
            self.cache_path.unlink()
            missing = self.resolved_registry()
            self.assertEqual(missing["agy_project_id"], "project-test")

            self.write_cache({})
            empty = self.resolved_registry()
            self.assertEqual(empty["agy_project_id"], "project-test")

    def test_accepts_both_official_workspace_resource_forms(self) -> None:
        alias = self.base / "project-config-alias"
        alias.symlink_to(self.project, target_is_directory=True)
        forms = (
            {"folderUri": self.project.resolve().as_uri()},
            {"folderUri": alias.absolute().as_uri()},
            {
                "gitFolder": {
                    "folderUri": self.project.resolve().as_uri(),
                    "branch": "main",
                }
            },
        )
        with self.with_paths():
            for resource in forms:
                with self.subTest(resource=resource):
                    self.write_project_config(
                        self.project_config(resources=[resource])
                    )
                    resolved = self.resolved_registry()
                    self.assertEqual(resolved["agy_project_id"], "project-test")

    def test_accepts_cli_project_resources_container(self) -> None:
        config = self.project_config()
        config["projectResources"] = config.pop("workspace")
        with self.with_paths():
            self.write_project_config(config)
            resolved = self.resolved_registry()
        self.assertEqual(resolved["agy_project_id"], "project-test")

    def test_rejects_ambiguous_project_resource_containers(self) -> None:
        config = self.project_config()
        config["projectResources"] = dict(config["workspace"])
        with self.with_paths():
            self.write_project_config(config)
            with self.assertRaisesRegex(SystemExit, "exactly one.*container"):
                self.resolved_registry()

    def test_rejects_missing_malformed_or_mismatched_project_config(self) -> None:
        config_path = self.project_config_dir / "project-test.json"
        cases = (
            ("missing", None, "config.*missing|cannot read.*config"),
            ("malformed", "{bad json", "config"),
            ("not-object", [], "object"),
            (
                "wrong-id",
                self.project_config(project_id="other-project"),
                "config.*id|id.*observation|does not match",
            ),
            (
                "wrong-root",
                self.project_config(
                    resources=[{"folderUri": self.worktree.resolve().as_uri()}]
                ),
                "workspace.*root|resource.*root|does not match",
            ),
            (
                "zero-resources",
                self.project_config(resources=[]),
                "exactly one.*resource|resource.*exactly one",
            ),
            (
                "multiple-resources",
                self.project_config(
                    resources=[
                        {"folderUri": self.project.resolve().as_uri()},
                        {"folderUri": self.worktree.resolve().as_uri()},
                    ]
                ),
                "exactly one.*resource|resource.*exactly one",
            ),
        )
        with self.with_paths():
            for label, value, expected in cases:
                with self.subTest(label=label):
                    if config_path.exists():
                        config_path.unlink()
                    if value is not None:
                        if isinstance(value, str):
                            config_path.write_text(value)
                        else:
                            config_path.write_text(json.dumps(value))
                    with self.assertRaisesRegex(SystemExit, expected):
                        self.resolved_registry()

    def test_rejects_non_file_relative_and_hosted_workspace_uris(self) -> None:
        invalid_uris = (
            "relative/project",
            "https://example.invalid/project",
            "vscode-remote://host/project",
            "file://remote-host/absolute/project",
        )
        with self.with_paths():
            for uri in invalid_uris:
                for resource in (
                    {"folderUri": uri},
                    {"gitFolder": {"folderUri": uri}},
                ):
                    with self.subTest(resource=resource):
                        self.write_project_config(
                            self.project_config(resources=[resource])
                        )
                        with self.assertRaisesRegex(
                            SystemExit,
                            "file URI|file.*absolute|hosted|workspace resource",
                        ):
                            self.resolved_registry()

    def test_present_cache_must_be_a_valid_json_object(self) -> None:
        with self.with_paths():
            self.cache_path.write_text("{bad json")
            with self.assertRaisesRegex(SystemExit, "cache"):
                self.resolved_registry()
            self.write_cache([])
            with self.assertRaisesRegex(SystemExit, "object"):
                self.resolved_registry()

    def test_cache_rejects_relative_keys_and_invalid_values(self) -> None:
        cases = (
            ({"relative/project": "project-test"}, "absolute"),
            ({str(self.project): ""}, "Project id|Project ID|non-empty"),
            ({str(self.project): 42}, "Project id|Project ID|string"),
            ({str(self.project): "../escape"}, "unsafe"),
            ({str(self.project): "default-cli-project"}, "reserved|default"),
        )
        with self.with_paths():
            for value, message in cases:
                with self.subTest(value=value):
                    self.write_cache(value)
                    with self.assertRaisesRegex(SystemExit, message):
                        self.resolved_registry()

    def test_observation_rejects_unsafe_project_ids_but_accepts_opaque_id(self) -> None:
        unsafe = (
            "../escape",
            "has/slash",
            "white space",
            "default-cli-project",
            "outside-of-project",
        )
        with self.with_paths():
            for project_id in unsafe:
                with self.subTest(project_id=project_id):
                    self.write_registry([self.registry_entry(project_id=project_id)])
                    with self.assertRaisesRegex(SystemExit, "unsafe|reserved|default|outside"):
                        self.resolved_registry()

            opaque = "Project_42.alpha-beta"
            self.write_registry([self.registry_entry(project_id=opaque)])
            self.write_project_config(
                self.project_config(project_id=opaque),
                project_id=opaque,
            )
            _, profile = self.materialize()
            self.assertEqual(profile["agy_project_id"], opaque)

    def test_cache_rejects_duplicate_canonical_roots(self) -> None:
        alias = self.base / "project-alias"
        alias.symlink_to(self.project, target_is_directory=True)
        self.write_cache(
            {
                str(self.project): "project-test",
                str(alias): "project-test",
            }
        )
        with self.with_paths(), self.assertRaisesRegex(SystemExit, "duplicate|ambiguous"):
            self.resolved_registry()

    def test_registry_rejects_old_v1_identity_fields(self) -> None:
        entry = self.registry_entry()
        for key, value in (
            ("agy_project_id", "project-test"),
            ("project_settings", {}),
            ("matching_project_ids", ["project-test"]),
        ):
            with self.subTest(key=key):
                invalid = dict(entry)
                invalid[key] = value
                self.write_registry([invalid])
                with self.with_paths(), self.assertRaisesRegex(SystemExit, key):
                    self.resolved_registry()

    def test_registry_requires_one_cli_observation_and_uses_its_project_id(self) -> None:
        common = execute.worktree_identity(self.worktree)[1]
        with self.with_paths():
            self.write_registry([])
            with self.assertRaisesRegex(SystemExit, "exactly one"):
                execute.resolve_registry(common, self.worktree)

            entry = self.registry_entry()
            self.write_registry([entry, entry])
            with self.assertRaisesRegex(SystemExit, "exactly one"):
                execute.resolve_registry(common, self.worktree)

            entry = self.registry_entry(project_id="other-project")
            self.write_registry([entry])
            self.write_project_config(
                self.project_config(project_id="other-project"),
                project_id="other-project",
            )
            resolved = execute.resolve_registry(common, self.worktree)
            self.assertEqual(resolved["agy_project_id"], "other-project")

            self.write_cache({str(self.project): "project-test"})
            with self.assertRaisesRegex(SystemExit, "cache.*conflict|does not match"):
                execute.resolve_registry(common, self.worktree)

            entry = self.registry_entry()
            entry["project_policy_observation"]["source"] = "official_project_ui_or_permissions"
            self.write_registry([entry])
            with self.assertRaisesRegex(SystemExit, "official_cli_permissions|source"):
                execute.resolve_registry(common, self.worktree)

    def test_cache_mappings_may_be_absent_or_matching_but_not_conflicting(self) -> None:
        with self.with_paths():
            self.resolved_registry()
            self.write_cache({str(self.project): "project-test"})
            self.resolved_registry()
            self.write_cache({str(self.worktree): "project-test"})
            self.resolved_registry()
            self.write_cache(
                {
                    str(self.project): "project-test",
                    str(self.worktree): "project-test",
                }
            )
            self.resolved_registry()
            self.write_cache({str(self.project): "other-project"})
            with self.assertRaisesRegex(SystemExit, "persistent.*conflict|does not match"):
                self.resolved_registry()
            self.write_cache({str(self.worktree): "other-project"})
            with self.assertRaisesRegex(SystemExit, "worktree.*conflict|does not match"):
                self.resolved_registry()
            self.write_cache(
                {
                    str(self.project): "project-test",
                    str(self.worktree): "other-project",
                }
            )
            with self.assertRaisesRegex(SystemExit, "worktree.*Project|conflict"):
                self.resolved_registry()

    def test_cache_observation_digest_binds_absence_and_matching_presence(self) -> None:
        with self.with_paths():
            self.write_cache({})
            absent = self.resolved_registry()["backend_resolution"]
            self.write_cache({str(self.project): "project-test"})
            present = self.resolved_registry()["backend_resolution"]

        self.assertNotEqual(
            absent["workspace_cache_observation_digest"],
            present["workspace_cache_observation_digest"],
        )
        for field in (
            "backend",
            "persistent_root",
            "project_id",
            "project_config_digest",
            "registry_entry_digest",
            "agy_version",
        ):
            self.assertEqual(absent[field], present[field])

    def test_existing_state_rejects_assignment_backend_registry_and_cli_version_drift(self) -> None:
        with self.with_paths():
            changed = self.assignment()
            changed["task_key"] = "T-assignment-drift"
            changed["task_contract"]["run_id"] = "T-assignment-drift"
            self.write_assignment(changed)
            self.materialize()
            changed["task_commands"] = {"allow": ["rg"], "deny": ["git push"]}
            self.write_assignment(changed)
            with self.assertRaisesRegex(SystemExit, "different frozen assignment"):
                self.materialize()

            for drift in ("project-config", "registry", "cache-observation", "version"):
                with self.subTest(drift=drift):
                    self.write_registry()
                    self.write_project_config()
                    self.write_cache({})
                    self.write_fake_agy_version("agy version 1.2.3")
                    assignment = self.assignment()
                    assignment["task_key"] = f"T-{drift}-drift"
                    assignment["task_contract"]["run_id"] = f"T-{drift}-drift"
                    self.write_assignment(assignment)
                    profile_path, _ = self.materialize()
                    snapshot = profile_path.parent / "snapshots" / "frozen.json"
                    snapshot.parent.mkdir(parents=True, exist_ok=True)
                    snapshot.write_text("{}\n")

                    if drift == "project-config":
                        config = self.project_config()
                        config["name"] = "changed after snapshot"
                        self.write_project_config(config)
                    elif drift == "registry":
                        entry = self.registry_entry()
                        entry["global_permissions"]["allow"].append(
                            "command(git status)"
                        )
                        self.write_registry([entry])
                    elif drift == "cache-observation":
                        self.write_cache({str(self.project): "project-test"})
                    else:
                        self.write_fake_agy_version("agy version 1.2.4")

                    with self.assertRaisesRegex(
                        SystemExit,
                        "backend resolution|config|registry|cache|version",
                    ):
                        self.materialize()

    def test_resolve_and_all_lifecycle_verbs_never_write_cli_cache(self) -> None:
        with self.with_paths():
            before = self.cache_path.read_bytes()
            before_stat = self.cache_path.stat()
            old = Path.cwd()
            os.chdir(self.worktree)
            try:
                for verb in execute.VERBS:
                    with (
                        patch.object(execute, "invoke_adapter", return_value=0),
                        patch.object(
                            sys,
                            "argv",
                            [str(SCRIPT), verb, "--assignment", str(self.assignment_path)],
                        ),
                        self.assertRaises(SystemExit) as caught,
                    ):
                        execute.main()
                    self.assertEqual(caught.exception.code, 0)
            finally:
                os.chdir(old)
            after_stat = self.cache_path.stat()
            self.assertEqual(self.cache_path.read_bytes(), before)
            self.assertEqual(after_stat.st_mtime_ns, before_stat.st_mtime_ns)


if __name__ == "__main__":
    unittest.main()
