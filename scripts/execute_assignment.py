#!/usr/bin/env python3
"""Run one frozen controller assignment from its linked Git worktree.

This is the only executor-facing entrypoint.  Backend-specific identity,
permissions, and launch settings live in the user-local registry and never in
the controller assignment or fleet prompt.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any
from urllib.parse import unquote, urlsplit

import _agy_adapter as agy


ASSIGNMENT_SCHEMA = "execution-assignment-v2"
REGISTRY_SCHEMA = "execution-agy-registry-v2"
STATE_ROOT = Path("/tmp/execution/agy").resolve()
REGISTRY_PATH = Path.home() / ".codex" / "execution" / "agy-projects.json"
AGY_PROJECTS_CACHE_PATH = (
    Path.home() / ".gemini" / "antigravity-cli" / "cache" / "projects.json"
)
AGY_PROJECT_CONFIG_DIR = Path.home() / ".gemini" / "config" / "projects"
VERBS = ("doctor", "snapshot", "dispatch", "resume", "status", "verify", "denied")
REQUIRED_ASSIGNMENT = {
    "schema",
    "task_key",
    "worktree_root",
    "repository",
    "executor_role",
    "mode",
    "task_contract",
    "oracle",
    "task_commands",
    "protected_artifacts",
    "snapshot_paths",
    "allowed_repo_writes",
    "path_change_budgets",
    "external_payload_consent",
}
OPTIONAL_ASSIGNMENT = {"timeout", "inject_prompt_file"}
FORBIDDEN_BACKEND_KEYS = {
    "agy_project_id",
    "agy_project_root",
    "agy_root",
    "model",
    "effort",
    "state_dir",
    "worktree_layout",
    "launch_cwd",
    "project_policy_observation",
    "global_permissions",
    "project_permissions",
    "project_settings",
    "permission_observation",
    "permissions",
    "sandbox",
    "backend_resolution",
}

UNSAFE_PROJECT_IDS = {"default-cli-project", "outside-of-project"}


def fail(message: str) -> None:
    raise SystemExit(f"execution assignment refused: {message}")


def canonical_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def load_json(path: Path, what: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        fail(f"cannot read {what}: {error}")
    if not isinstance(value, dict):
        fail(f"{what} must be a JSON object")
    return value


def git_value(root: Path, *args: str) -> Path | str:
    result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", *args],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        fail(f"not a Git worktree: {root}")
    value = result.stdout.strip()
    if not value:
        fail(f"Git returned an empty value for {root}")
    return value


def absolute_git_path(root: Path, flag: str) -> Path:
    value = str(git_value(root, flag))
    path = Path(value)
    if not path.is_absolute():
        path = root / path
    return path.resolve()


def worktree_identity(root: Path) -> tuple[Path, Path, Path]:
    top = Path(str(git_value(root, "--show-toplevel"))).resolve()
    common = absolute_git_path(top, "--git-common-dir")
    git_dir = absolute_git_path(top, "--git-dir")
    return top, common, git_dir


def contains_forbidden_backend_key(value: Any) -> str | None:
    if isinstance(value, dict):
        for key, child in value.items():
            if key in FORBIDDEN_BACKEND_KEYS or key.startswith("agy_"):
                return key
            nested = contains_forbidden_backend_key(child)
            if nested:
                return nested
    elif isinstance(value, list):
        for child in value:
            nested = contains_forbidden_backend_key(child)
            if nested:
                return nested
    return None


def load_assignment(path_text: str) -> tuple[Path, dict[str, Any], str]:
    path = Path(path_text)
    if not path.is_absolute():
        fail("--assignment must be an absolute controller-state path")
    path = path.resolve()
    if not path.is_file():
        fail(f"assignment file is missing: {path}")
    assignment = load_json(path, "assignment")
    unknown = set(assignment) - REQUIRED_ASSIGNMENT - OPTIONAL_ASSIGNMENT
    missing = REQUIRED_ASSIGNMENT - set(assignment)
    if missing or unknown:
        fail(
            "assignment keys are invalid: "
            + ", ".join(sorted([*(f"missing {k}" for k in missing), *(f"unknown {k}" for k in unknown)]))
        )
    if assignment.get("schema") != ASSIGNMENT_SCHEMA:
        fail(f"schema must be {ASSIGNMENT_SCHEMA}")
    if assignment.get("executor_role") not in ("qa", "dev"):
        fail("executor_role must be qa or dev")
    if assignment.get("mode") not in ("measure-only", "bounded-write"):
        fail("mode must be measure-only or bounded-write")
    forbidden = contains_forbidden_backend_key(assignment)
    if forbidden:
        fail(f"assignment must not contain backend field: {forbidden}")
    if not isinstance(assignment["task_key"], str) or not assignment["task_key"]:
        fail("task_key must be a non-empty string")
    if not isinstance(assignment["repository"], str) or not assignment["repository"]:
        fail("repository must be a non-empty string")
    if not isinstance(assignment["task_contract"], dict):
        fail("task_contract must be an object")
    oracle = assignment["oracle"]
    if not isinstance(oracle, dict) or set(oracle) != {"path", "sha256"}:
        fail("oracle must contain exactly path and sha256")
    if not isinstance(oracle["path"], str) or not Path(oracle["path"]).is_absolute():
        fail("oracle.path must be absolute")
    if not isinstance(oracle["sha256"], str) or len(oracle["sha256"]) != 64:
        fail("oracle.sha256 must be a sha256 digest")
    if "timeout" in assignment:
        timeout = assignment["timeout"]
        if (
            not isinstance(timeout, str)
            or not timeout
            or timeout != timeout.strip()
        ):
            fail("timeout must be a non-empty AGY CLI duration string")
    return path, assignment, canonical_digest(assignment)


def validate_project_id(value: Any, what: str) -> str:
    if not isinstance(value, str) or not value or value != value.strip():
        fail(f"{what} must be a non-empty string")
    if value in UNSAFE_PROJECT_IDS:
        fail(f"{what} uses reserved Project ID: {value}")
    if not agy.TASK_KEY_PATTERN.fullmatch(value):
        fail(f"{what} uses an unsafe Project ID: {value!r}")
    return value


def load_projects_cache() -> tuple[dict[str, str], bool]:
    """Load AGY's optional read-only workspace mapping cache."""
    if not AGY_PROJECTS_CACHE_PATH.exists():
        return {}, False
    cache = load_json(AGY_PROJECTS_CACHE_PATH, "AGY projects cache")
    normalized: dict[str, str] = {}
    for raw_root, raw_project_id in cache.items():
        if not isinstance(raw_root, str) or not Path(raw_root).is_absolute():
            fail("AGY projects cache keys must be absolute paths")
        canonical_root = str(Path(raw_root).resolve())
        if canonical_root in normalized:
            fail(
                "AGY projects cache has duplicate or ambiguous canonical root: "
                f"{canonical_root}"
            )
        normalized[canonical_root] = validate_project_id(
            raw_project_id,
            "AGY projects cache Project ID",
        )
    return normalized, True


def project_config_workspace_root(resource: Any) -> Path:
    if not isinstance(resource, dict):
        fail("AGY Project config workspace resource must be an object")
    flat_uri = resource.get("folderUri")
    git_folder = resource.get("gitFolder")
    nested_uri = git_folder.get("folderUri") if isinstance(git_folder, dict) else None
    if (flat_uri is None) == (nested_uri is None):
        fail(
            "AGY Project config workspace resource must contain either "
            "folderUri or gitFolder.folderUri"
        )
    uri = flat_uri if flat_uri is not None else nested_uri
    if not isinstance(uri, str) or not uri:
        fail("AGY Project config workspace resource must use an absolute file URI")
    parsed = urlsplit(uri)
    if parsed.scheme != "file":
        fail("AGY Project config workspace resource must use a file URI")
    if parsed.netloc:
        fail("AGY Project config workspace resource must not use a hosted file URI")
    if parsed.query or parsed.fragment:
        fail("AGY Project config workspace resource file URI must not contain query data")
    try:
        path = Path(unquote(parsed.path))
        if not path.is_absolute():
            fail("AGY Project config workspace resource file URI must be absolute")
        return path.resolve()
    except (OSError, RuntimeError, ValueError) as error:
        fail(f"AGY Project config workspace resource file URI is invalid: {error}")


def stable_project_config(value: Any) -> Any:
    """Remove known CLI timestamps while retaining identity and policy data."""
    if isinstance(value, dict):
        return {
            key: stable_project_config(child)
            for key, child in value.items()
            if key not in {"updatedAt", "lastOpenedAt"}
        }
    if isinstance(value, list):
        return [stable_project_config(child) for child in value]
    return value


def project_config_resources(config: dict[str, Any]) -> tuple[str, list[Any]]:
    """Return the one supported AGY Project resource container."""
    containers: list[tuple[str, list[Any]]] = []
    for key in ("workspace", "projectResources"):
        if key not in config:
            continue
        container = config[key]
        if not isinstance(container, dict):
            fail(f"AGY Project config {key} must be an object")
        resources = container.get("resources")
        if not isinstance(resources, list):
            fail(f"AGY Project config {key}.resources must be a list")
        containers.append((key, resources))
    if len(containers) != 1:
        fail(
            "AGY Project config must contain exactly one workspace resource "
            "container"
        )
    key, resources = containers[0]
    if len(resources) != 1:
        fail("AGY Project config must contain exactly one workspace resource")
    return key, resources


def load_project_config(project_id: str, project_root: Path) -> tuple[dict[str, Any], str]:
    config_path = AGY_PROJECT_CONFIG_DIR / f"{project_id}.json"
    config = load_json(config_path, "AGY Project config")
    if config.get("id") != project_id:
        fail("AGY Project config id does not match the registry observation")
    resource_container, resources = project_config_resources(config)
    configured_root = project_config_workspace_root(resources[0])
    if configured_root != project_root:
        fail("AGY Project config workspace resource root does not match the registry")

    # Bind security and identity semantics, but ignore known CLI timestamps and
    # normalize aliases in the already-validated workspace URI.
    stable_config = stable_project_config(config)
    stable_resource = stable_config[resource_container]["resources"][0]
    if "folderUri" in stable_resource:
        stable_resource["folderUri"] = configured_root.as_uri()
    else:
        stable_resource["gitFolder"]["folderUri"] = configured_root.as_uri()
    return config, canonical_digest(stable_config)


def agy_version() -> str:
    result = subprocess.run(
        ["agy", "--version"],
        text=True,
        capture_output=True,
        check=False,
    )
    value = (result.stdout or result.stderr).strip()
    if result.returncode or not value:
        fail("cannot read `agy --version`")
    return value


def resolve_registry(common_dir: Path, task_root: Path) -> dict[str, Any]:
    common_dir = common_dir.resolve()
    task_root = task_root.resolve()
    registry = load_json(REGISTRY_PATH, "user-local AGY registry")
    if set(registry) != {"schema", "projects"}:
        fail("registry must contain exactly schema and projects")
    if registry.get("schema") != REGISTRY_SCHEMA:
        fail(f"registry schema must be {REGISTRY_SCHEMA}")
    entries = registry.get("projects")
    if not isinstance(entries, list):
        fail("registry projects must be a list")
    matches = []
    for entry in entries:
        if not isinstance(entry, dict) or entry.get("backend") != "agy-cli":
            continue
        raw_common = entry.get("git_common_dir")
        if not isinstance(raw_common, str) or not Path(raw_common).is_absolute():
            fail("registry git_common_dir must be absolute")
        if Path(raw_common).resolve() == common_dir:
            matches.append(entry)
    if len(matches) != 1:
        fail("registry must contain exactly one matching AGY Project")
    entry = matches[0]
    forbidden_v1 = {
        "agy_project_id",
        "project_settings",
        "matching_project_ids",
        "project_permissions",
    }
    present_v1 = sorted(forbidden_v1 & set(entry))
    if present_v1:
        fail("registry entry contains removed field: " + ", ".join(present_v1))
    required = {
        "git_common_dir",
        "backend",
        "agy_project_root",
        "project_policy_observation",
        "global_permissions",
    }
    missing = required - set(entry)
    unknown = set(entry) - required
    if missing or unknown:
        fail(
            "registry entry keys are invalid: "
            + ", ".join(
                sorted(
                    [
                        *(f"missing {key}" for key in missing),
                        *(f"unknown {key}" for key in unknown),
                    ]
                )
            )
        )
    if not isinstance(entry["git_common_dir"], str) or not Path(
        entry["git_common_dir"]
    ).is_absolute():
        fail("registry git_common_dir must be absolute")
    if not isinstance(entry["agy_project_root"], str) or not Path(
        entry["agy_project_root"]
    ).is_absolute():
        fail("registry agy_project_root must be absolute")
    project_root = Path(entry["agy_project_root"]).resolve()
    if not project_root.is_dir():
        fail("registered AGY Project root is missing")
    registered_top, registered_common, _ = worktree_identity(project_root)
    if registered_top != project_root or registered_common != common_dir:
        fail("registered AGY Project root does not match this Git repository")
    if project_root == task_root:
        fail("executor worktree must not be the persistent AGY Project root")
    observation = entry["project_policy_observation"]
    if not isinstance(observation, dict):
        fail("registry Project policy observation is missing or invalid")
    observation_keys = {
        "source",
        "project_id",
        "project_root",
        "observed_at",
        "permissions",
    }
    if set(observation) != observation_keys:
        fail("registry Project policy observation keys are invalid")
    if observation.get("source") != "official_cli_permissions":
        fail("registry Project policy observation has an invalid source")
    observed_project_id = observation.get("project_id")
    if (
        not isinstance(observed_project_id, str)
        or not observed_project_id
        or observed_project_id != observed_project_id.strip()
    ):
        fail("registry Project policy observation is missing its Project ID")
    project_id = validate_project_id(
        observed_project_id,
        "registry Project policy observation",
    )
    observed_root = observation.get("project_root")
    if (
        not isinstance(observed_root, str)
        or not Path(observed_root).is_absolute()
        or Path(observed_root).resolve() != project_root
    ):
        fail("registry Project policy observation does not match its Project root")
    if not isinstance(observation.get("observed_at"), str) or not observation["observed_at"]:
        fail("registry Project policy observation is stale or incomplete")
    permissions = observation.get("permissions")
    if not isinstance(permissions, dict):
        fail("registry Project policy observation permissions are invalid")

    _, project_config_digest = load_project_config(project_id, project_root)
    cache, cache_file_present = load_projects_cache()
    persistent_cache_id = cache.get(str(project_root))
    if persistent_cache_id is not None and persistent_cache_id != project_id:
        fail("persistent-root AGY projects cache mapping conflicts with Project config")
    task_cache_id = cache.get(str(task_root))
    if task_cache_id is not None and task_cache_id != project_id:
        fail("task worktree AGY Project cache mapping conflicts with Project config")
    cache_observation = {
        "cache_file_present": cache_file_present,
        "persistent_root": {
            "root": str(project_root),
            "project_id": persistent_cache_id,
        },
        "task_worktree": {
            "root": str(task_root),
            "project_id": task_cache_id,
        },
    }
    backend_resolution = {
        "backend": "agy-cli",
        "persistent_root": str(project_root),
        "project_id": project_id,
        "project_config_digest": project_config_digest,
        "workspace_cache_observation_digest": canonical_digest(cache_observation),
        "registry_entry_digest": canonical_digest(entry),
        "agy_version": agy_version(),
    }
    resolved = dict(entry)
    resolved["agy_project_id"] = project_id
    resolved["project_permissions"] = permissions
    resolved["backend_resolution"] = backend_resolution
    return resolved


def validate_executor_worktree(assignment: dict[str, Any]) -> tuple[Path, Path]:
    cwd = Path.cwd().resolve()
    declared = Path(str(assignment["worktree_root"])).resolve()
    if cwd != declared:
        fail("cwd must equal assignment worktree_root")
    top, common, git_dir = worktree_identity(cwd)
    if top != cwd:
        fail("cwd must be the assigned Git worktree root")
    if git_dir == common:
        fail("persistent repository root cannot execute an assignment")
    return top, common


def validate_assignment_location(
    assignment_path: Path, task_root: Path, common_dir: Path
) -> None:
    """Require controller state to live outside every linked worktree."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(task_root),
            "worktree",
            "list",
            "--porcelain",
            "-z",
        ],
        capture_output=True,
        check=False,
    )
    if result.returncode:
        fail("cannot enumerate linked Git worktrees")
    repository_roots: set[Path] = {common_dir.resolve()}
    for field in result.stdout.split(b"\0"):
        if field.startswith(b"worktree "):
            raw_root = os.fsdecode(field[len(b"worktree ") :])
            repository_roots.add(Path(os.path.abspath(raw_root)))
            repository_roots.add(Path(raw_root).resolve())
    if len(repository_roots) == 1:
        fail("Git reported no linked worktree roots")
    reference_path = Path(os.path.abspath(os.fspath(assignment_path)))
    assignment_locations = {assignment_path.resolve()}
    assignment_locations.update(
        candidate.resolve()
        for candidate in (reference_path, *reference_path.parents)
    )
    for location in assignment_locations:
        for repository_root in repository_roots:
            if location == repository_root or location.is_relative_to(repository_root):
                fail(
                    "assignment must be controller state outside every repository "
                    "worktree"
                )


def copy_oracle(assignment: dict[str, Any], state_dir: Path) -> None:
    source = Path(assignment["oracle"]["path"]).resolve()
    if not source.is_file():
        fail(f"oracle is missing: {source}")
    content = source.read_bytes()
    digest = hashlib.sha256(content).hexdigest()
    if digest != assignment["oracle"]["sha256"]:
        fail("oracle digest changed after controller freeze")
    destination = state_dir / "oracles" / f"{assignment['task_key']}.md"
    if destination.exists() and hashlib.sha256(destination.read_bytes()).hexdigest() != digest:
        fail("materialized oracle differs from frozen controller oracle")
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_bytes(content)


def materialize_profile(
    assignment: dict[str, Any], assignment_digest: str, task_root: Path, registry: dict[str, Any]
) -> Path:
    task_key = assignment["task_key"]
    if not agy.TASK_KEY_PATTERN.fullmatch(task_key):
        fail("task_key is unsafe for the execution state namespace")
    project_id = str(registry["agy_project_id"])
    state_dir = (STATE_ROOT / project_id / task_key).resolve()
    if not state_dir.is_relative_to(STATE_ROOT):
        fail("registry Project id produces an unsafe state path")
    profile_path = state_dir / "backend-profile.json"
    if STATE_ROOT.is_dir():
        for prior_path in STATE_ROOT.glob(f"*/{task_key}/backend-profile.json"):
            prior = load_json(prior_path, "existing materialized backend profile")
            if (
                prior.get("assignment_digest") == assignment_digest
                and prior.get("backend_resolution") != registry["backend_resolution"]
            ):
                fail(
                    "existing task state has a different backend resolution; "
                    "refusing Project identity drift"
                )
    if profile_path.is_file():
        previous = load_json(profile_path, "materialized backend profile")
        if previous.get("assignment_digest") != assignment_digest:
            fail("existing state belongs to a different frozen assignment")
        if previous.get("backend_resolution") != registry["backend_resolution"]:
            fail("existing state backend resolution changed; refusing registry or version drift")
    settings = agy.DISPATCH_ROLE_SETTINGS[assignment["executor_role"]]
    profile: dict[str, Any] = {
        "root": str(task_root),
        "repo": assignment["repository"],
        "state_dir": str(state_dir),
        "mode": assignment["mode"],
        "agy_project_root": registry["agy_project_root"],
        "agy_project_id": project_id,
        "backend_resolution": registry["backend_resolution"],
        "dispatch_role": assignment["executor_role"],
        "model": settings["model"],
        "effort": settings["effort"],
        "worktree_layout": agy.REQUIRED_WORKTREE_LAYOUT,
        "launch_cwd": agy.REQUIRED_LAUNCH_CWD,
        "global_permissions": registry["global_permissions"],
        "project_permissions": registry["project_permissions"],
        "project_policy_observation": registry["project_policy_observation"],
        "task_contract": assignment["task_contract"],
        "task_commands": assignment["task_commands"],
        "protected_artifacts": assignment["protected_artifacts"],
        "snapshot_paths": assignment["snapshot_paths"],
        "allowed_repo_writes": assignment["allowed_repo_writes"],
        "path_change_budgets": assignment["path_change_budgets"],
        "external_payload_consent": assignment["external_payload_consent"],
        "assignment_digest": assignment_digest,
    }
    for key in OPTIONAL_ASSIGNMENT:
        if key in assignment:
            profile[key] = assignment[key]
    profile["sandbox"] = True
    state_dir.mkdir(parents=True, exist_ok=True)
    copy_oracle(assignment, state_dir)
    profile_path.write_text(json.dumps(profile, indent=2, sort_keys=True) + "\n")
    return profile_path


def invoke_adapter(verb: str, profile_path: Path, task_key: str) -> int:
    command = [sys.executable, str(Path(__file__).with_name("_agy_adapter.py")), verb, str(profile_path)]
    if verb not in ("doctor", "status"):
        command.append(task_key)
    return subprocess.run(command, check=False).returncode


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("verb", choices=VERBS)
    parser.add_argument("--assignment", required=True)
    args = parser.parse_args()
    assignment_path, assignment, assignment_digest = load_assignment(args.assignment)
    task_root, common_dir = validate_executor_worktree(assignment)
    validate_assignment_location(Path(args.assignment), task_root, common_dir)
    registry = resolve_registry(common_dir, task_root)
    profile_path = materialize_profile(assignment, assignment_digest, task_root, registry)
    raise SystemExit(invoke_adapter(args.verb, profile_path, assignment["task_key"]))


if __name__ == "__main__":
    main()
