#!/usr/bin/env python3
"""Read-only project-policy controller for bounded headless AGY dispatches."""
from __future__ import annotations

import argparse
import base64
import difflib
import fcntl
import hashlib
import json
import os
import re
import selectors
import shlex
import shutil
import sqlite3
import stat
import subprocess
import sys
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path

import _agy_assignment_guard as assignment_guard


HOME = Path.home()
SETTINGS = HOME / ".gemini" / "antigravity-cli" / "settings.json"
CONVERSATION_DIR = HOME / ".gemini" / "antigravity-cli" / "conversations"
STANDING_CONSENT = HOME / ".codex" / "execution" / "standing-consent.json"
TEMP_ROOT = Path("/tmp/execution/agy").resolve()
PERMISSION_KINDS = ("allow", "deny", "ask")
TASK_KEY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
EXTERNAL_SERVICE = "agy-headless"
STANDING_CONSENT_MODE = "standing"
STANDING_CONSENT_SOURCE = "standing_explicit_user_authorization"
DEFAULT_STANDING_CONSENT_ID = "all-bounded-work-items-v1"
DISPATCH_ROLE_SETTINGS = {
    "qa": {"model": "gemini-3.8-flash-high", "effort": "high"},
    "dev": {"model": "gemini-3.8-flash-medium", "effort": "medium"},
}
REQUIRED_WORKTREE_LAYOUT = "in-project"
REQUIRED_LAUNCH_CWD = "task-worktree"
RUN_EVIDENCE_VERSION = 6
VERIFIED_MARKER_VERSION = 6
AUDIT_CONTRACT_VERSION = 3
INIT_TIMEOUT_SECONDS = 60
TERMINATE_GRACE_SECONDS = 5
AGY_STEP_STATUS_DENIED = 7
TOOL_PERMISSION_MODES = {
    "request-review",
    "proceed-in-sandbox",
    "strict",
    "always-proceed",
}
GLOBAL_HOOK_PATHS = (
    HOME / ".gemini" / "config" / "hooks.json",
    HOME / ".gemini" / "antigravity-cli" / "hooks.json",
)
STATIC_GUARD_DIR = HOME / ".codex" / "execution"
STATIC_GUARD_LOADER = STATIC_GUARD_DIR / "agy-assignment-guard-loader.py"
STATIC_GUARD_CORE = STATIC_GUARD_DIR / "agy-assignment-guard-core.py"
STATIC_GUARD_CHAIN = STATIC_GUARD_DIR / "agy-assignment-guard-chain.py"
GLOBAL_ASSIGNMENT_GUARD_NAME = "execution-assignment-guard"
ACTIVE_GUARD_REGISTRATION_SCHEMA = "execution-agy-active-guard-v2"
CANONICAL_GLOBAL_POLICY = {
    "allow": [
        "command(pwd)",
        "command(rg)",
        "command(sed)",
        "command(shasum)",
        "command(git log)",
        "command(git status)",
        "command(git diff)",
        "command(git show)",
        "command(git rev-parse)",
        "command(git ls-files)",
        "command(git merge-base)",
        "command(uv)",
        "command(python3)",
    ],
    "deny": [
        "command(git add)",
        "command(git commit)",
        "command(git push)",
        "command(git checkout)",
        "command(git switch)",
        "command(git reset)",
        "command(git restore)",
        "command(git stash)",
        "command(git worktree)",
        "command(git merge)",
        "command(git rebase)",
        "command(git cherry-pick)",
        "command(git revert)",
        "command(git clean)",
        "command(git tag)",
        "command(git update-ref)",
        "command(git apply)",
        "command(git am)",
        "command(git rm)",
        "command(git mv)",
        "command(gh issue close)",
        "command(gh issue edit)",
        "command(gh issue comment)",
        "command(gh pr create)",
        "command(gh pr merge)",
        "command(gh release create)",
        "command(gh api)",
        "command(rm -rf)",
    ],
    "ask": [],
}


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def json_digest(value: object) -> str:
    payload = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode()
    return hashlib.sha256(payload).hexdigest()


def relative_repo_path(root: Path, value: str, field: str) -> str:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts or value in ("", "."):
        raise SystemExit(f"{field} must contain exact repository-relative paths: {value}")
    return path.as_posix()


def validate_rule_list(value: object, field: str) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        raise SystemExit(f"{field} must be a list of permission-rule strings")
    return list(dict.fromkeys(value))


def normalize_permission_surface(value: dict | None) -> dict[str, list[str]]:
    source = value or {}
    return {
        kind: sorted(dict.fromkeys(source.get(kind, []) or []))
        for kind in PERMISSION_KINDS
    }


def canonical_global_policy() -> dict[str, list[str]]:
    """Return a copy of the documented cross-Project command baseline."""
    return {kind: list(CANONICAL_GLOBAL_POLICY[kind]) for kind in PERMISSION_KINDS}


def task_session_policy(profile: dict) -> str:
    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    policy = task.get("session_policy", "ticketed")
    if policy not in ("ticketed", "one-shot"):
        raise SystemExit(
            "task_contract.session_policy must be ticketed or one-shot"
        )
    return policy


def validate_dispatch_role(profile: dict) -> dict[str, str]:
    """Reject a profile unless its role, model, and effort are one fixed pair."""
    role = profile.get("dispatch_role")
    if role not in DISPATCH_ROLE_SETTINGS:
        raise SystemExit("dispatch_role must be qa or dev")
    settings = DISPATCH_ROLE_SETTINGS[role]
    if profile.get("model") != settings["model"]:
        raise SystemExit(
            f"{role} dispatch_role requires model {settings['model']}"
        )
    if profile.get("effort") != settings["effort"]:
        raise SystemExit(
            f"{role} dispatch_role requires effort {settings['effort']}"
        )
    return settings


def validate_task_identity(profile: dict) -> str:
    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    policy = task_session_policy(profile)
    issue = str(task.get("issue", "")).strip()
    run_id = str(task.get("run_id", "")).strip()
    if policy == "ticketed":
        if not issue:
            raise SystemExit(
                "ticketed task requires task_contract.issue"
            )
        if run_id:
            raise SystemExit(
                "ticketed task must not set task_contract.run_id"
            )
        expected = issue
    else:
        if issue:
            raise SystemExit(
                "one-shot task must not set task_contract.issue"
            )
        if not run_id:
            raise SystemExit(
                "one-shot task requires task_contract.run_id"
            )
        intent = task.get("intent")
        if not isinstance(intent, str) or not intent.strip():
            raise SystemExit(
                "one-shot task requires non-empty task_contract.intent"
            )
        expected = run_id
    if not TASK_KEY_PATTERN.fullmatch(expected):
        raise SystemExit(
            "task identity must match [A-Za-z0-9][A-Za-z0-9._-]{0,127}"
        )
    return expected


def validate_task_key(profile: dict, task_key: str) -> None:
    expected = validate_task_identity(profile)
    if expected != str(task_key):
        raise SystemExit(
            f"task identity {expected} does not match requested key={task_key}"
        )


@contextmanager
def task_operation_lock(profile: dict, task_key: str, operation: str):
    """Serialize snapshot, launch, and verification for one task conversation."""
    validate_task_key(profile, task_key)
    lock_path = Path(profile["state_dir"]) / "runs" / f"{task_key}.operation.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    handle = lock_path.open("a+")
    try:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit(
                f"refusing {operation}: another snapshot, AGY launch, or verify "
                f"operation owns task {task_key}"
            )
        handle.seek(0)
        handle.truncate()
        handle.write(
            json.dumps(
                {
                    "task_key": task_key,
                    "operation": operation,
                    "started_at": datetime.now(timezone.utc).isoformat(),
                }
            )
            + "\n"
        )
        handle.flush()
        yield
    finally:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        finally:
            handle.close()


def required_external_payload_classes(profile: dict) -> set[str]:
    """Return the exact external data classes exposed by this task profile."""
    required = {"task_contract", "oracle", "repository_read_context"}
    task = profile.get("task_contract")
    if isinstance(task, dict) and task.get("design_inputs"):
        required.add("design_inputs")
    if profile.get("inject_prompt_file"):
        required.add("injected_prompt")
    if profile.get("mode") == "bounded-write":
        required.add("repository_write_diff")
    return required


def validate_approved_payload_classes(
    profile: dict,
    payloads: object,
    *,
    field: str,
) -> list[str]:
    if not isinstance(payloads, list) or any(
        not isinstance(value, str) for value in payloads
    ):
        raise SystemExit(f"{field} must be a list of strings")
    approved = set(payloads)
    missing = sorted(required_external_payload_classes(profile) - approved)
    if missing:
        raise SystemExit(
            f"{field} is missing payload classes: " + ", ".join(missing)
        )
    return sorted(approved)


def standing_consent_reference(consent: object) -> str | None:
    """Return the requested local standing-consent id, if any.

    An omitted consent deliberately opts into the user's local standing record.
    A machine without that record still fails closed before preflight.
    """
    if consent is None:
        return DEFAULT_STANDING_CONSENT_ID
    if not isinstance(consent, dict) or consent.get("mode") != STANDING_CONSENT_MODE:
        return None
    if set(consent) - {"mode", "consent_id"}:
        raise SystemExit(
            "standing external_payload_consent may contain only mode and consent_id"
        )
    consent_id = consent.get("consent_id")
    if not isinstance(consent_id, str) or not consent_id.strip():
        raise SystemExit("standing external_payload_consent.consent_id is required")
    return consent_id.strip()


def load_standing_external_payload_consent(profile: dict, consent_id: str) -> dict:
    """Resolve an explicit, revocable user-local all-bounded-WI approval."""
    try:
        stored = json.loads(STANDING_CONSENT.read_text())
    except FileNotFoundError:
        raise SystemExit(
            "no profile-local external_payload_consent and no standing consent "
            f"registry at {STANDING_CONSENT}; obtain explicit user approval or "
            "configure a valid local standing registry"
        ) from None
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(
            f"standing consent registry is unreadable: {STANDING_CONSENT}: {error}"
        ) from error

    if not isinstance(stored, dict):
        raise SystemExit("standing consent registry must be a JSON object")
    if stored.get("version") != 1:
        raise SystemExit("standing consent registry version must be 1")
    if stored.get("consent_id") != consent_id:
        raise SystemExit(
            "standing consent registry consent_id does not match profile reference"
        )
    if stored.get("scope") != "all_bounded_work_items":
        raise SystemExit(
            "standing consent registry scope must be all_bounded_work_items"
        )
    if "revoked" in stored and not isinstance(stored["revoked"], bool):
        raise SystemExit("standing consent registry revoked must be boolean")
    if stored.get("revoked") is True:
        raise SystemExit("standing consent registry is revoked")
    if stored.get("destination") != EXTERNAL_SERVICE:
        raise SystemExit(
            f"standing consent registry destination must be {EXTERNAL_SERVICE}"
        )
    if stored.get("approved") is not True:
        raise SystemExit("standing consent registry approved must be true")
    if stored.get("approval_source") != STANDING_CONSENT_SOURCE:
        raise SystemExit(
            "standing consent registry approval_source must be "
            f"{STANDING_CONSENT_SOURCE}"
        )
    approval_record = stored.get("approval_record")
    if not isinstance(approval_record, str) or not approval_record.strip():
        raise SystemExit(
            "standing consent registry approval_record must preserve explicit "
            "user approval text"
        )
    approved_payload_classes = validate_approved_payload_classes(
        profile,
        stored.get("approved_payload_classes"),
        field="standing consent registry approved_payload_classes",
    )
    return {
        "mode": STANDING_CONSENT_MODE,
        "consent_id": consent_id,
        "registry_path": str(STANDING_CONSENT),
        "registry_digest": json_digest(stored),
        "destination": EXTERNAL_SERVICE,
        "approved": True,
        "approval_source": STANDING_CONSENT_SOURCE,
        "approval_record": approval_record.strip(),
        "approved_payload_classes": approved_payload_classes,
    }


def validate_external_payload_consent(profile: dict) -> dict:
    """Require current explicit per-task or standing user approval."""
    consent = profile.get("external_payload_consent")
    consent_id = standing_consent_reference(consent)
    if consent_id is not None:
        return load_standing_external_payload_consent(profile, consent_id)
    if not isinstance(consent, dict):
        raise SystemExit("external_payload_consent must be a JSON object")
    if consent.get("destination") != EXTERNAL_SERVICE:
        raise SystemExit(
            f"external_payload_consent.destination must be {EXTERNAL_SERVICE}"
        )
    if consent.get("approved") is not True:
        raise SystemExit("external_payload_consent.approved must be true")
    if consent.get("approval_source") != "explicit_user_after_risk_disclosure":
        raise SystemExit(
            "external_payload_consent.approval_source must be "
            "explicit_user_after_risk_disclosure; use a local standing registry "
            "instead of fabricating consent"
        )
    approval_record = consent.get("approval_record")
    if not isinstance(approval_record, str) or not approval_record.strip():
        raise SystemExit(
            "external_payload_consent.approval_record must preserve the explicit "
            "user approval text"
        )
    if approval_record.strip() == "REPLACE_WITH_EXACT_USER_APPROVAL_TEXT":
        raise SystemExit(
            "external_payload_consent.approval_record still contains the template "
            "placeholder"
        )
    approved_payload_classes = validate_approved_payload_classes(
        profile,
        consent.get("approved_payload_classes"),
        field="external_payload_consent.approved_payload_classes",
    )
    return {
        "destination": EXTERNAL_SERVICE,
        "approved": True,
        "approval_source": "explicit_user_after_risk_disclosure",
        "approval_record": approval_record.strip(),
        "approved_payload_classes": approved_payload_classes,
    }


def load_profile(path: str, *, validate_design: bool = True) -> dict:
    profile = json.loads(Path(path).read_text())
    required = (
        "root",
        "agy_project_root",
        "repo",
        "state_dir",
        "mode",
        "agy_project_id",
        "backend_resolution",
        "dispatch_role",
        "model",
        "effort",
        "worktree_layout",
        "launch_cwd",
        "global_permissions",
        "project_permissions",
        "task_commands",
        "protected_artifacts",
        "snapshot_paths",
        "allowed_repo_writes",
        "assignment_digest",
    )
    missing = [key for key in required if key not in profile]
    if missing:
        legacy = (
            " Legacy profiles placed the reusable baseline in "
            "`project_permissions`; migrate it to `global_permissions` and "
            "leave only repository-specific exceptions in "
            "`project_permissions`."
            if "permissions" in profile or "global_permissions" not in profile
            else ""
        )
        raise SystemExit(f"profile missing: {', '.join(missing)}.{legacy}")

    root = Path(profile["root"]).resolve()
    if not root.is_dir():
        raise SystemExit(f"root is not a directory: {root}")
    profile["root"] = str(root)

    # An AGY Project is a durable app/repository scope, not a ticket
    # namespace. A task runs from a distinct linked worktree physically inside
    # that scope; readiness checks below prove the complete binding.
    scope_value = profile["agy_project_root"]
    if not isinstance(scope_value, str) or not scope_value.strip():
        raise SystemExit("agy_project_root must be a non-empty directory path")
    agy_scope_root = Path(scope_value).resolve()
    if not agy_scope_root.is_dir():
        raise SystemExit(f"agy_project_root is not a directory: {agy_scope_root}")
    if agy_scope_root != root:
        scope_common = git_common_dir(agy_scope_root)
        root_common = git_common_dir(root)
        if scope_common is None or root_common is None:
            raise SystemExit(
                "root and agy_project_root must be Git worktree roots for "
                "a shared AGY Project dispatch"
            )
        if scope_common != root_common:
            raise SystemExit(
                "root must be a worktree of the same Git repository as "
                "agy_project_root"
            )
    profile["agy_project_root"] = str(agy_scope_root)

    validate_dispatch_role(profile)
    if profile["worktree_layout"] != REQUIRED_WORKTREE_LAYOUT:
        raise SystemExit(
            f"worktree_layout must be {REQUIRED_WORKTREE_LAYOUT}"
        )
    if profile["launch_cwd"] != REQUIRED_LAUNCH_CWD:
        raise SystemExit(f"launch_cwd must be {REQUIRED_LAUNCH_CWD}")

    state_dir = Path(profile["state_dir"]).resolve()
    if not state_dir.is_relative_to(TEMP_ROOT):
        raise SystemExit(
            "state_dir must be under /tmp/execution/agy so controller state "
            "remains transient and shared by Claude and Codex"
        )
    if state_dir == root or state_dir.is_relative_to(root):
        raise SystemExit(
            "state_dir must be outside the repository so controller evidence "
            "does not appear as an AGY repository mutation"
        )
    if state_dir == agy_scope_root or state_dir.is_relative_to(agy_scope_root):
        raise SystemExit(
            "state_dir must be outside agy_project_root so AGY cannot read "
            "controller-only state through its Project scope"
        )
    state_namespace = state_dir.relative_to(TEMP_ROOT)
    project_namespace = agy_project_id(profile)
    task_namespace = validate_task_identity(profile)
    if not TASK_KEY_PATTERN.fullmatch(project_namespace):
        raise SystemExit(
            "agy_project_id must be path-safe for the controller state namespace"
        )
    if state_namespace.parts != (project_namespace, task_namespace):
        raise SystemExit(
            "state_dir must equal /tmp/execution/agy/<agy_project_id>/<task-key>; "
            f"expected {TEMP_ROOT / project_namespace / task_namespace}"
        )
    profile["state_dir"] = str(state_dir)

    assignment_digest = profile["assignment_digest"]
    if not isinstance(assignment_digest, str) or not re.fullmatch(
        r"[0-9a-f]{64}", assignment_digest
    ):
        raise SystemExit("assignment_digest must be a sha256 digest")

    if profile["mode"] not in ("measure-only", "bounded-write"):
        raise SystemExit("mode must be measure-only or bounded-write")
    if profile["mode"] == "measure-only" and profile["allowed_repo_writes"]:
        raise SystemExit("measure-only profile cannot grant repository writes")

    if profile.get("sandbox") is not True:
        raise SystemExit("sandbox must be fixed to true")

    task = profile.get("task_contract")
    if not isinstance(task, dict):
        raise SystemExit("profile missing task_contract")
    task_kind = task.get("kind")
    if profile["mode"] == "bounded-write":
        if task_kind != "implementation":
            raise SystemExit(
                "bounded-write requires task_contract.kind=implementation"
            )
        design_inputs = task.get("design_inputs")
        if not isinstance(design_inputs, list) or not design_inputs:
            raise SystemExit(
                "bounded-write requires at least one frozen design input"
            )
        for entry in design_inputs:
            if (
                not isinstance(entry, dict)
                or not entry.get("path")
                or not entry.get("sha256")
            ):
                raise SystemExit("each design input requires path and sha256")
            design_path = Path(entry["path"])
            if not design_path.is_absolute():
                design_path = root / design_path
            if validate_design:
                if not design_path.is_file():
                    raise SystemExit(f"design input is missing: {design_path}")
                if sha256(design_path) != entry["sha256"]:
                    raise SystemExit(f"design input hash mismatch: {design_path}")
    elif task_kind not in ("measurement", "investigation", "review", "audit"):
        raise SystemExit(
            "measure-only task_contract.kind must be measurement, "
            "investigation, review, or audit"
        )
    validate_task_identity(profile)

    project_id = profile.get("agy_project_id")
    if not isinstance(project_id, str) or not project_id.strip():
        raise SystemExit("agy_project_id must name the manually selected persistent AGY Project")
    profile["agy_project_id"] = project_id.strip()

    backend_resolution = profile.get("backend_resolution")
    required_resolution = {
        "backend",
        "persistent_root",
        "project_id",
        "project_config_digest",
        "workspace_cache_observation_digest",
        "registry_entry_digest",
        "agy_version",
    }
    if not isinstance(backend_resolution, dict) or set(backend_resolution) != required_resolution:
        raise SystemExit("backend_resolution has an invalid shape")
    if backend_resolution.get("backend") != "agy-cli":
        raise SystemExit("backend_resolution.backend must be agy-cli")
    if backend_resolution.get("project_id") != profile["agy_project_id"]:
        raise SystemExit("backend_resolution.project_id must match agy_project_id")
    if Path(str(backend_resolution.get("persistent_root", ""))).resolve() != agy_scope_root:
        raise SystemExit("backend_resolution persistent_root must match agy_project_root")
    for field in (
        "project_config_digest",
        "workspace_cache_observation_digest",
        "registry_entry_digest",
    ):
        if not isinstance(backend_resolution.get(field), str) or not re.fullmatch(
            r"[0-9a-f]{64}", backend_resolution[field]
        ):
            raise SystemExit(f"backend_resolution.{field} must be a sha256 digest")
    if not isinstance(backend_resolution.get("agy_version"), str) or not backend_resolution["agy_version"].strip():
        raise SystemExit("backend_resolution.agy_version must be non-empty")

    global_permissions = profile["global_permissions"]
    if not isinstance(global_permissions, dict):
        raise SystemExit("global_permissions must be an object")
    for kind in PERMISSION_KINDS:
        global_permissions[kind] = validate_rule_list(
            global_permissions.get(kind, []),
            f"global_permissions.{kind}",
        )
    project_permissions = profile["project_permissions"]
    if not isinstance(project_permissions, dict):
        raise SystemExit("project_permissions must be an object")
    for kind in PERMISSION_KINDS:
        project_permissions[kind] = validate_rule_list(
            project_permissions.get(kind, []),
            f"project_permissions.{kind}",
        )

    observation = profile.get("project_policy_observation")
    if observation is not None:
        if not isinstance(observation, dict):
            raise SystemExit("project_policy_observation must be an object")
        if observation.get("source") != "official_cli_permissions":
            raise SystemExit(
                "project_policy_observation.source must be "
                "official_cli_permissions"
            )
        if observation.get("project_id") != profile["agy_project_id"]:
            raise SystemExit(
                "project_policy_observation.project_id must match agy_project_id"
            )
        observed_root = observation.get("project_root")
        if not isinstance(observed_root, str) or not observed_root.strip():
            raise SystemExit("project_policy_observation.project_root is required")
        observation["project_root"] = str(Path(observed_root).resolve())
        observed_at = observation.get("observed_at")
        if not isinstance(observed_at, str) or not observed_at.strip():
            raise SystemExit("project_policy_observation.observed_at is required")
        observed_permissions = observation.get("permissions")
        if not isinstance(observed_permissions, dict):
            raise SystemExit("project_policy_observation.permissions must be an object")
        for kind in PERMISSION_KINDS:
            observed_permissions[kind] = validate_rule_list(
                observed_permissions.get(kind, []),
                f"project_policy_observation.permissions.{kind}",
            )

    task_commands = profile["task_commands"]
    if not isinstance(task_commands, dict):
        raise SystemExit("task_commands must be an object")
    for kind in ("allow", "deny"):
        task_commands[kind] = validate_rule_list(
            task_commands.get(kind, []),
            f"task_commands.{kind}",
        )
    overlap = sorted(
        set(task_commands["allow"]) & set(task_commands["deny"])
    )
    if overlap:
        raise SystemExit(
            "task_commands cannot both allow and deny: " + ", ".join(overlap)
        )
    expected_soft_denied = task.get("expected_soft_denied_commands", [])
    if (
        not isinstance(expected_soft_denied, list)
        or any(
            not isinstance(command, str) or not command
            for command in expected_soft_denied
        )
    ):
        raise SystemExit(
            "task_contract.expected_soft_denied_commands must be exact command strings"
        )
    expected_soft_denied = list(dict.fromkeys(expected_soft_denied))
    if expected_soft_denied and profile["mode"] != "measure-only":
        raise SystemExit(
            "expected_soft_denied_commands require mode=measure-only"
        )
    missing_negative_controls = sorted(
        set(expected_soft_denied) - set(task_commands["allow"])
    )
    if missing_negative_controls:
        raise SystemExit(
            "expected_soft_denied_commands must also be in task_commands.allow: "
            + ", ".join(missing_negative_controls)
        )
    unsupported_negative_controls = sorted(
        set(expected_soft_denied) - {assignment_guard.SAFE_DENIAL_CANARY}
    )
    if unsupported_negative_controls:
        raise SystemExit(
            "expected_soft_denied_commands must use the fixed safe denial "
            f"canary: {assignment_guard.SAFE_DENIAL_CANARY}"
        )
    task["expected_soft_denied_commands"] = expected_soft_denied

    profile["snapshot_paths"] = [
        relative_repo_path(root, value, "snapshot_paths")
        for value in profile["snapshot_paths"]
    ]
    profile["allowed_repo_writes"] = [
        relative_repo_path(root, value, "allowed_repo_writes")
        for value in profile["allowed_repo_writes"]
    ]
    ignored_writes = [
        relative
        for relative in profile["allowed_repo_writes"]
        if repo_relative_path_is_ignored(root, relative)
    ]
    if ignored_writes:
        raise SystemExit(
            "allowed_repo_writes must be Git-visible paths, not ignored paths: "
            + ", ".join(ignored_writes)
        )

    budgets = profile.get("path_change_budgets", {})
    if not isinstance(budgets, dict):
        raise SystemExit("path_change_budgets must be an object")
    unknown_budget_paths = sorted(set(budgets) - set(profile["allowed_repo_writes"]))
    if unknown_budget_paths:
        raise SystemExit(
            "path_change_budgets contains non-writable paths: "
            + ", ".join(unknown_budget_paths)
        )
    normalized_budgets = {}
    for relative, budget in budgets.items():
        if isinstance(budget, bool):
            raise SystemExit(
                f"path_change_budgets.{relative} must be an integer or "
                "{max_added, max_deleted} object"
            )
        if isinstance(budget, int):
            if budget < 0:
                raise SystemExit(
                    f"path_change_budgets.{relative} must not be negative"
                )
            normalized_budgets[relative] = budget
            continue
        if not isinstance(budget, dict) or set(budget) != {
            "max_added",
            "max_deleted",
        }:
            raise SystemExit(
                f"path_change_budgets.{relative} must be an integer or "
                "{max_added, max_deleted} object"
            )
        added = budget["max_added"]
        deleted = budget["max_deleted"]
        if (
            isinstance(added, bool)
            or isinstance(deleted, bool)
            or not isinstance(added, int)
            or not isinstance(deleted, int)
            or added < 0
            or deleted < 0
        ):
            raise SystemExit(
                f"path_change_budgets.{relative} limits must be "
                "non-negative integers"
            )
        normalized_budgets[relative] = {
            "max_added": added,
            "max_deleted": deleted,
        }
    profile["path_change_budgets"] = normalized_budgets

    protected = profile["protected_artifacts"]
    if not isinstance(protected, list):
        raise SystemExit("protected_artifacts must be a list")
    for entry in protected:
        if (
            not isinstance(entry, dict)
            or not entry.get("path")
            or not entry.get("sha256")
        ):
            raise SystemExit("each protected artifact requires path and sha256")
        artifact = Path(entry["path"])
        if not artifact.is_absolute():
            artifact = root / artifact
            entry["path"] = str(artifact)
        if artifact.resolve() == SETTINGS.resolve():
            raise SystemExit(
                "protected_artifacts must not include the CLI settings file; "
                "its sparse serialization is bound through permission_state_digest"
            )
        if validate_design:
            if not artifact.is_file():
                raise SystemExit(f"protected artifact is missing: {artifact}")
            if sha256(artifact) != entry["sha256"]:
                raise SystemExit(f"protected artifact hash mismatch: {artifact}")

    inject_prompt_file = profile.get("inject_prompt_file")
    if inject_prompt_file:
        prompt_path = Path(inject_prompt_file).resolve()
        if not prompt_path.is_file():
            raise SystemExit(f"inject_prompt_file is not a file: {prompt_path}")
        profile["inject_prompt_file"] = str(prompt_path)
        profile["inject_prompt_file_sha256"] = sha256(prompt_path)
    profile["external_payload_consent"] = validate_external_payload_consent(profile)
    return profile


def manifest(root: Path, relative_paths: list[str]) -> dict[str, str]:
    entries: dict[str, str] = {}
    for relative in relative_paths:
        path = root / relative
        if path.is_file():
            entries[relative] = sha256(path)
        elif path.is_dir():
            for child in sorted(item for item in path.rglob("*") if item.is_file()):
                entries[str(child.relative_to(root))] = sha256(child)
        else:
            entries[relative] = "<missing>"
    return entries


def git_common_dir(root: Path) -> Path | None:
    """Return the shared Git dir for a worktree without changing it."""
    result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--git-common-dir"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        return None
    candidate = Path(result.stdout.strip())
    if not candidate.is_absolute():
        candidate = root / candidate
    return candidate.resolve()


def git_worktree_root(root: Path) -> Path | None:
    """Return the exact worktree top level containing root, if any."""
    result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--show-toplevel"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode or not result.stdout.strip():
        return None
    return Path(result.stdout.strip()).resolve()


def git_worktree_admin_dir(root: Path) -> Path | None:
    """Return this worktree's administrative directory, not only the common dir."""
    result = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "--absolute-git-dir"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode or not result.stdout.strip():
        return None
    return Path(result.stdout.strip()).resolve()


def project_ignores_task_worktree(scope: Path, root: Path) -> bool:
    """Keep the nested worktree container out of the persistent root status."""
    try:
        relative = root.relative_to(scope)
    except ValueError:
        return False
    result = subprocess.run(
        [
            "git",
            "-C",
            str(scope),
            "check-ignore",
            "-q",
            "--no-index",
            "--",
            relative.as_posix(),
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    return result.returncode == 0


def repo_relative_path_is_ignored(root: Path, relative: str) -> bool:
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "check-ignore",
            "-q",
            "--no-index",
            "--",
            relative,
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    return result.returncode == 0


def registered_worktree_paths(scope: Path) -> list[Path]:
    """Return Git's authoritative linked-worktree registry for this repo."""
    result = subprocess.run(
        ["git", "-C", str(scope), "worktree", "list", "--porcelain"],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        return []
    return sorted(
        dict.fromkeys(
            Path(line.removeprefix("worktree ")).resolve()
            for line in result.stdout.splitlines()
            if line.startswith("worktree ")
        )
    )


def git_pointer_sha256(root: Path) -> str | None:
    pointer = root / ".git"
    return sha256(pointer) if pointer.is_file() else None


def agy_project_root(profile: dict) -> Path:
    """Return the durable AGY Project scope, never just the task root."""
    return Path(profile.get("agy_project_root", profile["root"])).resolve()


def worktree_scope_report(profile: dict) -> dict:
    """Prove a task uses a distinct in-Project worktree of the same repo."""
    root = Path(profile["root"]).resolve()
    scope = agy_project_root(profile)
    blockers = []
    if profile.get("worktree_layout") != REQUIRED_WORKTREE_LAYOUT:
        blockers.append(
            f"worktree_layout must be {REQUIRED_WORKTREE_LAYOUT}"
        )
    if profile.get("launch_cwd") != REQUIRED_LAUNCH_CWD:
        blockers.append(f"launch_cwd must be {REQUIRED_LAUNCH_CWD}")
    if scope == root:
        blockers.append(
            "root must be a distinct task worktree inside agy_project_root"
        )
    elif not root.is_relative_to(scope):
        blockers.append(
            "root must be physically nested inside agy_project_root"
        )
    elif not project_ignores_task_worktree(scope, root):
        blockers.append(
            "the persistent Project root must ignore the nested task "
            "worktree path"
        )

    scope_top = git_worktree_root(scope)
    root_top = git_worktree_root(root)
    if scope_top != scope:
        blockers.append("agy_project_root must be an exact Git worktree root")
    if root_top != root:
        blockers.append("root must be an exact Git worktree root")

    registered = registered_worktree_paths(scope)
    registered_in_project = [
        path
        for path in registered
        if path == scope or path.is_relative_to(scope)
    ]
    if scope not in registered:
        blockers.append("agy_project_root is absent from git worktree list")
    if root not in registered:
        blockers.append("root is absent from git worktree list")

    scope_common = git_common_dir(scope)
    root_common = git_common_dir(root)
    if scope_common is None or root_common is None:
        blockers.append(
            "agy_project_root and root must both be Git worktree roots"
        )
    elif scope_common != root_common:
        blockers.append(
            "root is not a worktree of the agy_project_root repository"
        )
    return {
        "mode": REQUIRED_WORKTREE_LAYOUT,
        "project_scope_root": str(scope),
        "worktree_root": str(root),
        "project_worktree_top_level": str(scope_top) if scope_top else None,
        "task_worktree_top_level": str(root_top) if root_top else None,
        "git_common_dir": str(scope_common) if scope_common else None,
        "registered_in_project_worktrees": [
            str(path) for path in registered_in_project
        ],
        "project_git_pointer_sha256": git_pointer_sha256(scope),
        "task_git_pointer_sha256": git_pointer_sha256(root),
        "ignored_by_project_root": project_ignores_task_worktree(scope, root),
        "launch_cwd": str(root),
        "dispatch_ready": not blockers,
        "blockers": blockers,
    }


def agy_project_id(profile: dict) -> str:
    project_id = profile.get("agy_project_id")
    if not isinstance(project_id, str) or not project_id.strip():
        raise SystemExit("agy_project_id must name a persistent AGY Project")
    return project_id.strip()


@contextmanager
def project_concurrency_lock(profile: dict, task_key: str, operation: str):
    """Serialize a persistent AGY Project between at most one exclusive
    bounded-write task and any number of concurrent measure-only tasks —
    the scheduling contract for generic executor tasks. Nest this outside
    task_operation_lock at any call site that snapshots or launches AGY."""
    project = agy_project_id(profile)
    lock_path = TEMP_ROOT / project / "project.concurrency.lock"
    lock_path.parent.mkdir(parents=True, exist_ok=True)
    exclusive = profile.get("mode") == "bounded-write"
    flag = fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH
    handle = lock_path.open("a+")
    try:
        try:
            fcntl.flock(handle.fileno(), flag | fcntl.LOCK_NB)
        except BlockingIOError:
            kind = "bounded-write" if exclusive else "measure-only"
            blocking = "another task" if exclusive else "an active bounded-write task"
            raise SystemExit(
                f"refusing {operation}: {kind} task {task_key} cannot start "
                f"in Project {project} while {blocking} holds it"
            )
        if exclusive:
            # Only the exclusive holder may safely own the file's content —
            # concurrent LOCK_SH holders writing here would race each other.
            handle.seek(0)
            handle.truncate()
            handle.write(
                json.dumps(
                    {
                        "task_key": task_key,
                        "operation": operation,
                        "mode": profile.get("mode"),
                        "started_at": datetime.now(timezone.utc).isoformat(),
                    }
                )
                + "\n"
            )
            handle.flush()
        yield
    finally:
        try:
            fcntl.flock(handle.fileno(), fcntl.LOCK_UN)
        finally:
            handle.close()


def read_json_or_empty(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text())


def global_permission_sources() -> dict[str, dict[str, list[str]]]:
    """Read the formally documented CLI Global settings surface only."""
    settings = read_json_or_empty(SETTINGS)
    settings_permissions = settings.get("permissions", {})
    return {
        "agy_cli_global_settings": normalize_permission_surface(settings_permissions),
    }


def allow_non_workspace_access() -> object:
    """Return the effective CLI value, including its documented safe default."""
    value = read_json_or_empty(SETTINGS).get("allowNonWorkspaceAccess", False)
    if not isinstance(value, bool):
        raise SystemExit("allowNonWorkspaceAccess must be a boolean when present")
    return value


def tool_permission_mode() -> str:
    """Return the effective CLI tool mode, including sparse-persistence default."""
    value = read_json_or_empty(SETTINGS).get("toolPermission", "request-review")
    if value not in TOOL_PERMISSION_MODES:
        raise SystemExit("toolPermission is missing an official CLI mode")
    return value


def static_guard_sources() -> dict[str, Path]:
    return {
        "loader": Path(__file__).resolve().with_name("_agy_global_guard_loader.py"),
        "core": Path(assignment_guard.__file__).resolve(),
        "chain": Path(__file__).resolve().with_name("_agy_global_pretool_chain.py"),
    }


def static_guard_installation_state() -> dict[str, object]:
    """Read the controller-owned fixed loader without modifying user files."""
    sources = static_guard_sources()
    targets = {
        "loader": STATIC_GUARD_LOADER,
        "core": STATIC_GUARD_CORE,
        "chain": STATIC_GUARD_CHAIN,
    }
    files: dict[str, dict[str, object]] = {}
    for name, source in sources.items():
        expected = sha256(source)
        target = targets[name]
        installed = (
            target.is_file()
            and not target.is_symlink()
            and target.stat().st_uid == os.getuid()
            and target.stat().st_mode & 0o077 == 0
            and sha256(target) == expected
        )
        files[name] = {
            "path": str(target),
            "source_sha256": expected,
            "installed": installed,
        }
    return {"ready": all(item["installed"] for item in files.values()), "files": files}


def global_assignment_guard_hook_config() -> dict[str, object]:
    """Return the fixed loader hook without an environment-selected program."""
    return {
        "PreToolUse": [
            {
                "matcher": "*",
                "hooks": [static_assignment_guard_loader_hook()],
            }
        ]
    }


def static_assignment_guard_loader_hook() -> dict[str, object]:
    """Return the only controller-approved static assignment-guard command."""
    return {
        "type": "command",
        "command": "exec " + shlex.quote(str(STATIC_GUARD_LOADER)),
        "timeout": 10,
    }


def static_assignment_guard_chain_hook() -> dict[str, object]:
    """Return the fixed cap-plus-assignment guard hook command."""
    return {
        "type": "command",
        "command": "exec " + shlex.quote(str(STATIC_GUARD_CHAIN)),
        "timeout": 10,
    }


def cap_agent_guard_covers_run_command(value: dict[str, object]) -> bool:
    """Require the loader on a more-specific legacy command matcher.

    AGY selects a specific ``run_command`` matcher before a wildcard matcher.
    A reviewed cap-agent guard therefore has to use the fixed chain wrapper.
    The wrapper invokes cap first, then the fixed loader after cap allows. A
    failing cap guard can still short-circuit the fixed negative control;
    raw-stream verification handles only that case.
    """
    cap = value.get("cap-agent-guard")
    if not isinstance(cap, dict) or cap.get("enabled", True) is False:
        return True
    entries = cap.get("PreToolUse")
    if not isinstance(entries, list):
        return True
    required = static_assignment_guard_chain_hook()
    for entry in entries:
        if not isinstance(entry, dict) or entry.get("matcher") != "run_command":
            continue
        hooks = entry.get("hooks")
        if not isinstance(hooks, list) or required not in hooks:
            return False
    return True


def global_hook_observation() -> list[dict[str, object]]:
    """Bind the official user-global hook files that can affect this run."""
    observation: list[dict[str, object]] = []
    for path in GLOBAL_HOOK_PATHS:
        if not path.exists():
            continue
        try:
            raw = path.read_bytes()
            value = json.loads(raw)
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            raise SystemExit(f"cannot read AGY hooks config {path}: {error}") from error
        if not isinstance(value, dict):
            raise SystemExit(f"AGY hooks config must be a JSON object: {path}")
        active_pretool = sorted(
            name
            for name, config in value.items()
            if isinstance(name, str)
            and isinstance(config, dict)
            and config.get("enabled", True) is not False
            and isinstance(config.get("PreToolUse"), list)
            and bool(config["PreToolUse"])
        )
        unsupported = [
            name
            for name in active_pretool
            if name not in {"cap-agent-guard", GLOBAL_ASSIGNMENT_GUARD_NAME}
        ]
        if unsupported:
            raise SystemExit(
                "unsupported user-global AGY PreToolUse hooks are active: "
                + ", ".join(unsupported)
            )
        if GLOBAL_ASSIGNMENT_GUARD_NAME in active_pretool and value.get(
            GLOBAL_ASSIGNMENT_GUARD_NAME
        ) != global_assignment_guard_hook_config():
            raise SystemExit(
                "global execution-assignment-guard hook differs from the "
                "frozen opt-in loader contract"
            )
        observation.append(
            {
                "path": str(path.resolve()),
                "sha256": hashlib.sha256(raw).hexdigest(),
                "names": sorted(value),
                "active_pretool": active_pretool,
                "cap_agent_guard_covers_run_command": (
                    cap_agent_guard_covers_run_command(value)
                ),
            }
        )
    return observation


def global_assignment_guard_hook_state() -> dict[str, object]:
    """Require exactly one active, static global loader before dispatch."""
    matches = [
        entry
        for entry in global_hook_observation()
        if GLOBAL_ASSIGNMENT_GUARD_NAME in entry["active_pretool"]
    ]
    if len(matches) > 1:
        raise SystemExit(
            "execution-assignment-guard must be active in exactly one AGY "
            "global hooks file"
        )
    if not matches:
        return {
            "installed": False,
            "path": None,
            "command_matcher_covered": False,
        }
    return {
        "installed": True,
        "path": matches[0]["path"],
        "command_matcher_covered": all(
            entry["cap_agent_guard_covers_run_command"]
            for entry in global_hook_observation()
        ),
    }


def global_permission_surface() -> dict[str, list[str]]:
    sources = global_permission_sources()
    return normalize_permission_surface(
        {
            kind: [
                rule
                for source in sources.values()
                for rule in source[kind]
            ]
            for kind in PERMISSION_KINDS
        }
    )


def split_rule_tokens(pattern: str) -> list[str]:
    tokens: list[str] = []
    current: list[str] = []
    escaped = False
    for character in pattern.strip():
        if character.isspace() and not escaped:
            if current:
                tokens.append("".join(current))
                current = []
            continue
        current.append(character)
        if escaped:
            escaped = False
        elif character == "\\":
            escaped = True
    if current:
        tokens.append("".join(current))
    return tokens


def command_rule_matches(rule: str, command: str) -> bool:
    match = re.fullmatch(r"command\((.*)\)", rule)
    if not match:
        return False
    pattern = match.group(1)
    if pattern == "*":
        return True
    try:
        command_tokens = shlex.split(command)
    except ValueError:
        return False
    rule_tokens = split_rule_tokens(pattern)
    if not rule_tokens or len(command_tokens) < len(rule_tokens):
        return False
    for rule_token, command_token in zip(rule_tokens, command_tokens):
        try:
            if re.fullmatch(rule_token, command_token) is None:
                return False
        except re.error:
            if rule_token != command_token:
                return False
    return True


def permission_decision(
    global_surface: dict[str, list[str]],
    project_surface: dict[str, list[str]],
    command: str,
) -> tuple[str, str | None, str | None]:
    """Return AGY's documented cross-scope union decision and its source.

    Antigravity documents that Projects inherit and augment Global rules, and
    that Deny > Ask > Allow. This function applies that documented ordering to
    the two observed surfaces. It deliberately does not read private Project
    registry files to manufacture an observation.
    """
    for kind in ("deny", "ask", "allow"):
        for source, surface in (("project", project_surface), ("global", global_surface)):
            for rule in surface.get(kind, []):
                if command_rule_matches(rule, command):
                    return kind, rule, source
    return "ask", None, "default"


def task_command_decision(profile: dict, command: str) -> tuple[str, str, str]:
    """Enforce the controller contract before evaluating the broader AGY policy."""
    commands = profile["task_commands"]
    if command in commands["deny"]:
        return "deny", "task_commands.deny", "task_contract"
    if command not in commands["allow"]:
        return "deny", "task_commands.allow (not listed)", "task_contract"
    return "allow", "task_commands.allow", "task_contract"


def expected_project_surface(profile: dict) -> dict[str, list[str]]:
    return normalize_permission_surface(profile["project_permissions"])


def permission_state(profile: dict) -> dict:
    global_sources = global_permission_sources()
    global_surface = normalize_permission_surface(
        {
            kind: [
                rule
                for source in global_sources.values()
                for rule in source[kind]
            ]
            for kind in PERMISSION_KINDS
        }
    )
    observation = profile.get("project_policy_observation")
    observed_permissions = (
        normalize_permission_surface(observation["permissions"])
        if isinstance(observation, dict)
        else None
    )
    return {
        "project_id": agy_project_id(profile),
        "project_scope_root": str(agy_project_root(profile)),
        "project": observed_permissions,
        "project_observation": observation,
        "global": global_surface,
        "global_sources": global_sources,
        "allow_non_workspace_access": allow_non_workspace_access(),
        "tool_permission_mode": tool_permission_mode(),
        "global_hooks": global_hook_observation(),
        "assignment_guard_hook": global_assignment_guard_hook_state(),
        "assignment_guard_loader": static_guard_installation_state(),
    }


def permission_state_digest(profile: dict) -> str:
    state = permission_state(profile)
    return json_digest(
        {
            "project_id": state["project_id"],
            "project_scope_root": state["project_scope_root"],
            "project": state["project"],
            "project_observation": state["project_observation"],
            "global": state["global"],
            "global_sources": state["global_sources"],
            "allow_non_workspace_access": state["allow_non_workspace_access"],
            "tool_permission_mode": state["tool_permission_mode"],
            "global_hooks": state["global_hooks"],
            "assignment_guard_hook": state["assignment_guard_hook"],
            "assignment_guard_loader": state["assignment_guard_loader"],
        }
    )


def formal_project_capabilities() -> dict[str, object]:
    """Report public CLI controls and the read-only Project identity surfaces."""
    result = subprocess.run(
        ["agy", "--help"],
        text=True,
        capture_output=True,
        check=False,
    )
    surface = result.stdout + result.stderr
    subcommands = {
        line.split()[0]
        for line in surface.splitlines()
        if line.startswith("  ") and line.strip() and not line.lstrip().startswith("--")
    }
    return {
        "agy_help_exit_code": result.returncode,
        "project_selector_flag": "--project" in surface,
        "project_creation_flag": "--new-project" in surface,
        "project_enumeration_cli": "projects" in subcommands,
        "machine_readable_project_policy_cli": "permissions" in subcommands,
        "formal_configuration_paths": [str(SETTINGS)],
        "read_only_project_config_dir": str(
            HOME / ".gemini" / "config" / "projects"
        ),
        "read_only_workspace_cache": str(
            HOME / ".gemini" / "antigravity-cli" / "cache" / "projects.json"
        ),
        "official_manual_surfaces": [
            "AGY /permissions scope picker (Global, Project, Shared)",
            "agy --new-project and agy --project=<id>",
        ],
    }


def project_setup_manual_steps(profile: dict) -> list[str]:
    """Return fail-closed CLI-only controller steps."""
    project_id = agy_project_id(profile)
    project_root = str(agy_project_root(profile))
    return [
        f"From {project_root}, run `agy --new-project` once and record the Project "
        "ID shown by the CLI. The optional read-only cache does not need a mapping.",
        f"Use `agy --project={project_id}` for the persistent Project. Do not create "
        "a Project for the linked task worktree.",
        "Confirm ~/.gemini/config/projects/<project-id>.json has that ID and exactly "
        "one workspace resource for the persistent root.",
        "In that AGY CLI session, run `/permissions`, select Global, and review the "
        "global_permissions baseline exactly once for all Projects.",
        "Run `/permissions`, select Project, and review only the profile's "
        "project_permissions exceptions.",
        "Keep the effective `allowNonWorkspaceAccess` value false. The CLI may omit this "
        "key because false is its documented sparse-persistence default. Record the CLI "
        "Project id, root, and rules in the registry, then rerun doctor and snapshot.",
        "Before snapshot, install the exact inert-by-default "
        "execution-assignment-guard loader in one AGY user-global hooks.json file. "
        "Install its fixed loader and core under ~/.codex/execution. It reads a "
        "short-lived, worktree-bound registration only for active executor tasks. "
        "The executor never writes CLI settings, global hooks, or Project files.",
    ]


def project_policy_report(profile: dict) -> dict:
    state = permission_state(profile)
    expected = expected_project_surface(profile)
    actual = state["project"]
    missing = (
        {
            kind: sorted(set(expected[kind]) - set(actual[kind]))
            for kind in PERMISSION_KINDS
        }
        if actual is not None
        else {kind: [] for kind in PERMISSION_KINDS}
    )
    extra = (
        {
            kind: sorted(set(actual[kind]) - set(expected[kind]))
            for kind in PERMISSION_KINDS
        }
        if actual is not None
        else {kind: [] for kind in PERMISSION_KINDS}
    )
    global_surface = state["global"]
    expected_global = normalize_permission_surface(profile["global_permissions"])
    global_missing = {
        kind: sorted(set(expected_global[kind]) - set(global_surface[kind]))
        for kind in PERMISSION_KINDS
    }
    global_extra = {
        kind: sorted(set(global_surface[kind]) - set(expected_global[kind]))
        for kind in PERMISSION_KINDS
    }
    project_policy_incomplete = actual is None or any(
        missing[kind] or extra[kind] for kind in PERMISSION_KINDS
    )
    global_policy_incomplete = any(
        global_missing[kind] or global_extra[kind] for kind in PERMISSION_KINDS
    )
    command_checks = []
    blockers = []
    expected_soft_denied = set(
        profile["task_contract"].get("expected_soft_denied_commands", [])
    )
    scope = worktree_scope_report(profile)
    guard_hook = state["assignment_guard_hook"]
    guard_loader = state["assignment_guard_loader"]
    observation = state["project_observation"]
    observed_root_matches = (
        isinstance(observation, dict)
        and Path(observation["project_root"]).resolve() == agy_project_root(profile)
    )
    blockers.extend(scope["blockers"])

    if actual is None:
        blockers.append(
            "Project policy has not been observed through /permissions Project scope"
        )
    elif any(missing[kind] or extra[kind] for kind in PERMISSION_KINDS):
        blockers.append(
            "Project-scope permission rules differ from project_permissions"
        )
    if actual is not None and not observed_root_matches:
        blockers.append(
            "Project policy observation root does not match agy_project_root"
        )
    if state["allow_non_workspace_access"] is not False:
        blockers.append("effective allowNonWorkspaceAccess must be false")
    if not guard_hook["installed"]:
        blockers.append(
            "one exact user-global execution-assignment-guard hook must be active"
        )
    elif not guard_hook["command_matcher_covered"]:
        blockers.append(
            "every active cap-agent-guard run_command matcher must use the "
            "fixed execution-assignment-guard chain"
        )
    if not guard_loader["ready"]:
        blockers.append(
            "fixed user-local assignment guard loader and core must match the "
            "controller source"
        )
    if global_policy_incomplete:
        blockers.append(
            "Global permissions differ from the reviewed global_permissions baseline"
        )

    for expected_decision in ("allow", "deny"):
        for command in profile["task_commands"].get(expected_decision, []):
            task_decision, task_rule, task_source = task_command_decision(profile, command)
            if task_decision == "deny":
                decision, rule, source = task_decision, task_rule, task_source
            elif actual is None:
                decision, rule, source = "unknown", None, "project-unobserved"
            else:
                decision, rule, source = permission_decision(
                    global_surface, actual, command
                )
            is_negative_control = (
                expected_decision == "allow" and command in expected_soft_denied
            )
            decision_matches = (
                decision == "deny"
                if is_negative_control
                else decision == expected_decision
            )
            command_checks.append(
                {
                    "command": command,
                    "expected": (
                        "soft-denied" if is_negative_control else expected_decision
                    ),
                    "decision": decision,
                    "matched_rule": rule,
                    "source": source,
                }
            )
            if not decision_matches:
                blockers.append(
                    f"task command expected "
                    f"{'an explicit deny' if is_negative_control else expected_decision} "
                    "but resolves "
                    f"{decision}: {command}"
                )
    if actual is None or project_policy_incomplete or not observed_root_matches:
        provisioning_status = (
            "PROJECT_SETUP_REQUIRED: Project policy requires official CLI observation"
        )
    elif state["allow_non_workspace_access"] is not False:
        provisioning_status = (
            "GLOBAL_SETUP_REQUIRED: allowNonWorkspaceAccess must be false"
        )
    elif not guard_hook["installed"]:
        provisioning_status = (
            "GLOBAL_GUARD_SETUP_REQUIRED: install the exact user-global "
            "execution-assignment-guard loader"
        )
    elif not guard_hook["command_matcher_covered"]:
        provisioning_status = (
            "GLOBAL_GUARD_SETUP_REQUIRED: use the fixed "
            "execution-assignment-guard chain for every active "
            "cap-agent-guard run_command matcher"
        )
    elif not guard_loader["ready"]:
        provisioning_status = (
            "GLOBAL_GUARD_SETUP_REQUIRED: install the fixed user-local "
            "assignment guard loader and core"
        )
    elif global_policy_incomplete:
        provisioning_status = (
            "GLOBAL_SETUP_REQUIRED: Global permissions differ from reviewed baseline"
        )
    elif scope["blockers"]:
        provisioning_status = "WORKTREE_BINDING_REQUIRED: task root is not eligible"
    else:
        provisioning_status = "READY"

    return {
        "project_id": state["project_id"],
        "project_root": state["project_scope_root"],
        "worktree_root": profile["root"],
        "adapter_settings": {
            "dispatch_role": profile["dispatch_role"],
            "model": profile["model"],
            "effort": profile["effort"],
            "worktree_layout": profile["worktree_layout"],
            "launch_cwd": profile["launch_cwd"],
        },
        "worktree_scope": scope,
        "project_permission_digest": json_digest(actual) if actual is not None else None,
        "project_policy_observation": state["project_observation"],
        "project_discovery_status": (
            "authoritative_project_config_resolved"
            if actual is not None
            else "PROJECT_SETUP_REQUIRED"
        ),
        "project_policy_observability": (
            "official_cli_permissions"
            if actual is not None
            else "PROJECT_SETUP_REQUIRED"
        ),
        "formal_project_capabilities": formal_project_capabilities(),
        "manual_setup": project_setup_manual_steps(profile),
        "allow_non_workspace_access": state["allow_non_workspace_access"],
        "assignment_guard_hook": guard_hook,
        "assignment_guard_loader": guard_loader,
        "tool_permission_mode": state["tool_permission_mode"],
        "global_hooks": state["global_hooks"],
        "permission_layer_diagnostics": {
            "global": {
                "decision": "ready" if not global_policy_incomplete else "drift",
                "source": "agy_cli_global_settings",
                "matched_rule": None,
                "settings_path": str(SETTINGS),
            },
            "project": {
                "decision": (
                    "observed" if actual is not None else "PROJECT_SETUP_REQUIRED"
                ),
                "source": (
                    "official_cli_permissions"
                    if actual is not None
                    else "project-unobserved"
                ),
                "matched_rule": None,
            },
            "file_access_policy": {
                "decision": (
                    "deny"
                    if state["allow_non_workspace_access"] is False
                    else "unknown_or_drift"
                ),
                "source": (
                    "agy_cli_global_settings"
                    if actual is not None
                    else "project-unobserved"
                ),
                "matched_rule": "allowNonWorkspaceAccess: false",
            },
            "assignment_guard": {
                "decision": (
                    "ready"
                    if (
                        guard_hook["installed"]
                        and guard_hook["command_matcher_covered"]
                        and guard_loader["ready"]
                    )
                    else "blocked"
                ),
                "source": "agy_cli_global_hooks",
                "matched_rule": guard_hook["path"],
            },
            "task_contract": {
                "decision": "exact_allowlist_required",
                "source": "task_commands",
                "matched_rule": "byte-exact task_commands.allow",
            },
            "controller_host": {
                "decision": "outside_agy_policy",
                "source": "controller_host",
                "matched_rule": None,
                "remediation": (
                    "A host launch/egress denial proves no AGY payload was sent; "
                    "do not change AGY policy or silently change workers."
                ),
            },
        },
        "permission_state_digest": permission_state_digest(profile),
        "project_permissions_status": (
            "ready" if not project_policy_incomplete else "drift_or_unobserved"
        ),
        "global_permissions_status": (
            "ready" if not global_policy_incomplete else "drift"
        ),
        "missing_project_rules": missing,
        "extra_project_rules": extra,
        "missing_global_rules": global_missing,
        "extra_global_rules": global_extra,
        "global_rules": global_surface,
        "global_rule_sources": state["global_sources"],
        "task_command_checks": command_checks,
        "provisioning_status": provisioning_status,
        "dispatch_ready": not blockers,
        "blockers": blockers,
    }


def doctor(profile: dict) -> dict:
    report = project_policy_report(profile)
    report["external_payload_consent"] = {
        "destination": profile["external_payload_consent"]["destination"],
        "approval_source": profile["external_payload_consent"]["approval_source"],
        "approved_payload_classes": profile["external_payload_consent"][
            "approved_payload_classes"
        ],
    }
    print(json.dumps(report, indent=2))
    if not report["dispatch_ready"]:
        raise SystemExit(2)
    return report


def require_project_ready(profile: dict) -> dict:
    report = project_policy_report(profile)
    if not report["dispatch_ready"]:
        raise SystemExit(
            report["provisioning_status"]
            + "; rerun `doctor` after using `agy --new-project`, "
            "`agy --project=<id>`, and `/permissions`; confirm the read-only "
            "Project config. The optional projects.json cache is only a cross-check: "
            + "; ".join(report["blockers"])
        )
    return report


def validate_live_issue(profile: dict, issue: str) -> dict:
    result = subprocess.run(
        [
            "gh",
            "issue",
            "view",
            str(issue),
            "-R",
            profile["repo"],
            "--json",
            "number,state,url,title,body,comments",
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise SystemExit(
            f"cannot verify live issue #{issue}: {result.stderr.strip()}"
        )
    payload = json.loads(result.stdout)
    if str(payload.get("number")) != str(issue):
        raise SystemExit(f"live issue identity mismatch for #{issue}")
    if (
        profile["task_contract"]["kind"] == "implementation"
        and payload.get("state") != "OPEN"
    ):
        raise SystemExit(f"implementation issue #{issue} is not open")
    return payload


def frozen_task_state(profile: dict, task_key: str) -> dict:
    validate_task_key(profile, task_key)
    if task_session_policy(profile) == "ticketed":
        return validate_live_issue(profile, task_key)
    task = profile["task_contract"]
    return {
        "run_id": task_key,
        "state": "ONE_SHOT",
        "kind": task["kind"],
        "intent": task["intent"].strip(),
    }


def task_state_contract(profile: dict, task_state: dict) -> dict:
    """Freeze task-defining fields without binding mutable discussion chatter."""
    if task_session_policy(profile) == "ticketed":
        return {
            key: task_state.get(key)
            for key in ("number", "state", "url", "title", "body")
        }
    return {
        key: task_state.get(key)
        for key in ("run_id", "state", "kind", "intent")
    }


def assert_task_state_unchanged(
    profile: dict,
    task_state: dict,
    snapshot_data: dict,
) -> None:
    expected = snapshot_data.get("task_state_contract_digest")
    current = json_digest(task_state_contract(profile, task_state))
    if not isinstance(expected, str) or current != expected:
        raise SystemExit("VOID: frozen task state changed after snapshot")


def assignment_guard_contract() -> dict[str, str]:
    """Bind the local enforcement program into every frozen dispatch contract."""
    source = Path(assignment_guard.__file__).resolve()
    if not source.is_file():
        raise SystemExit("assignment guard source is missing")
    return {
        "policy_schema": assignment_guard.POLICY_SCHEMA,
        "audit_schema": assignment_guard.AUDIT_SCHEMA,
        "safe_denial_canary": assignment_guard.SAFE_DENIAL_CANARY,
        "program_sha256": sha256(source),
    }


def dispatch_contract(profile: dict) -> dict:
    # The initial snapshot freezes every execution boundary below.  A
    # controller may nevertheless issue a bounded correction in the same
    # ticket conversation after inspecting a candidate.  That round prompt is
    # recorded with its report and must not turn an otherwise isolated
    # correction into an unverifiable dirty-worktree dead end.
    contract = {
        key: profile.get(key)
        for key in (
            "assignment_digest",
            "backend_resolution",
            "root",
            "repo",
            "mode",
            "dispatch_role",
            "model",
            "effort",
            "worktree_layout",
            "launch_cwd",
            "sandbox",
            "timeout",
            "agy_project_root",
            "inject_prompt_file",
            "inject_prompt_file_sha256",
            "task_contract",
            "project_permissions",
            "task_commands",
            "protected_artifacts",
            "snapshot_paths",
            "allowed_repo_writes",
            "path_change_budgets",
            "external_payload_consent",
        )
    }
    contract["assignment_guard"] = assignment_guard_contract()
    return contract


def dispatch_contracts_match(
    current_contract: dict,
    prior_contract: dict,
    *,
    allow_prompt_hash_change: bool = False,
) -> bool:
    """Compare frozen authority, optionally admitting one ticket-resume prompt."""
    prior = dict(prior_contract)
    current = dict(current_contract)
    if allow_prompt_hash_change:
        prior.pop("inject_prompt_file_sha256", None)
        current.pop("inject_prompt_file_sha256", None)
    return prior == current


def snapshot_contract_matches(
    profile: dict,
    snapshot_contract: dict,
    *,
    allow_prompt_hash_change: bool = False,
) -> bool:
    """Compare the current profile with the snapshot-bound authority."""
    return dispatch_contracts_match(
        dispatch_contract(profile),
        snapshot_contract,
        allow_prompt_hash_change=allow_prompt_hash_change,
    )


def assert_injected_prompt_unchanged(profile: dict) -> None:
    path = profile.get("inject_prompt_file")
    expected = profile.get("inject_prompt_file_sha256")
    if not path:
        return
    prompt = Path(path)
    if not prompt.is_file() or not isinstance(expected, str) or sha256(prompt) != expected:
        raise SystemExit("VOID: injected prompt bytes changed after profile load")


def conversation_id_for_task(profile: dict, task_key: str) -> str | None:
    log_dir = Path(profile["state_dir"]) / "runs"
    conversation_path = log_dir / f"{task_key}.conversation"
    if conversation_path.is_file() and conversation_path.read_text().strip():
        return conversation_path.read_text().strip()
    return None


def conversation_database(conversation_id: str) -> Path:
    return CONVERSATION_DIR / f"{conversation_id}.db"


def readonly_conversation_connection(database: Path) -> sqlite3.Connection:
    """Open a completed AGY conversation without creating SQLite sidecars."""
    uri = database.resolve().as_uri() + "?mode=ro&immutable=1"
    return sqlite3.connect(uri, uri=True)


def conversation_step_max(conversation_id: str | None) -> int:
    if not conversation_id:
        return -1
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(
            f"conversation state is missing for {conversation_id}: {database}"
        )
    connection = readonly_conversation_connection(database)
    try:
        row = connection.execute(
            "select min(idx), max(idx), count(*) from steps"
        ).fetchone()
    finally:
        connection.close()
    if not row or not row[2]:
        return -1
    if row[0] is None or int(row[0]) < 0:
        raise SystemExit("VOID: AGY conversation contains a negative step index")
    return int(row[1])


def conversation_steps_digest(
    conversation_id: str,
    *,
    through_step: int,
) -> str:
    """Hash the exact auditable row surface through a verified step ceiling."""
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(
            f"conversation state is missing for {conversation_id}: {database}"
        )
    connection = readonly_conversation_connection(database)
    try:
        schema = connection.execute("pragma table_info(steps)").fetchall()
        column_names = [str(column[1]) for column in schema]
        if "idx" not in column_names:
            raise SystemExit("VOID: AGY conversation steps schema lacks idx")
        negative = connection.execute(
            "select idx from steps where idx < 0 limit 1"
        ).fetchone()
        if negative is not None:
            raise SystemExit("VOID: AGY conversation contains a negative step index")
        rows = connection.execute(
            "select * from steps where idx <= ? order by idx",
            (through_step,),
        ).fetchall()
    finally:
        connection.close()

    def normalized_value(value: object) -> dict:
        if value is None:
            return {"type": "null", "value": None}
        if isinstance(value, bytes):
            return {
                "type": "blob",
                "sha256": hashlib.sha256(value).hexdigest(),
                "length": len(value),
            }
        if isinstance(value, str):
            encoded = value.encode(errors="surrogateescape")
            return {
                "type": "text",
                "sha256": hashlib.sha256(encoded).hexdigest(),
                "length": len(encoded),
            }
        if isinstance(value, bool):
            return {"type": "bool", "value": value}
        if isinstance(value, int):
            return {"type": "integer", "value": value}
        if isinstance(value, float):
            return {"type": "real", "value": repr(value)}
        encoded = repr(value).encode()
        return {
            "type": type(value).__name__,
            "sha256": hashlib.sha256(encoded).hexdigest(),
        }

    normalized_rows = [
        [normalized_value(value) for value in row]
        for row in rows
    ]
    normalized_schema = [
        {
            "cid": int(column[0]),
            "name": str(column[1]),
            "type": str(column[2]),
            "notnull": int(column[3]),
            "default": column[4],
            "pk": int(column[5]),
        }
        for column in schema
    ]
    return json_digest(
        {
            "schema": normalized_schema,
            "columns": column_names,
            "rows": normalized_rows,
        }
    )


def git_ignored_paths(root: Path) -> list[str]:
    """Return ignored paths that ordinary porcelain deliberately suppresses."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "status",
            "--porcelain=v1",
            "--ignored=matching",
            "--untracked-files=all",
        ],
        text=True,
        capture_output=True,
        check=True,
    )
    return sorted(
        dict.fromkeys(
            line[3:]
            for line in result.stdout.splitlines()
            if line.startswith("!! ")
        )
    )


def tracked_manifest(root: Path) -> dict[str, str]:
    """Hash all tracked files so the persistent Project root stays read-only."""
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z"],
        text=False,
        capture_output=True,
        check=True,
    )
    entries: dict[str, str] = {}
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8", errors="surrogateescape")
        path = root / relative
        entries[relative] = sha256(path) if path.is_file() else "<missing>"
    return entries


def untracked_manifest(root: Path) -> dict[str, str]:
    """Hash non-ignored untracked files so dirty user WIP is preserved."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        text=False,
        capture_output=True,
        check=True,
    )
    entries: dict[str, str] = {}
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8", errors="surrogateescape")
        path = root / relative
        entries[relative] = sha256(path) if path.is_file() else "<missing>"
    return entries


def is_rebuild_cache_descendant(path: str) -> bool:
    """Return whether an ignored file lives in a recognized rebuild cache."""
    return any(
        part in {"target", "__pycache__", ".venv"}
        for part in Path(path).parts
    )


def ignored_noncache_manifest(
    root: Path,
    *,
    excluded_prefixes: tuple[str, ...] = (),
    exempt_rebuild_caches: bool = True,
) -> dict[str, str]:
    """Hash ignored non-cache bytes that a worker must not silently mutate."""
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "ls-files",
            "--others",
            "--ignored",
            "--exclude-standard",
            "-z",
        ],
        text=False,
        capture_output=True,
        check=True,
    )
    entries: dict[str, str] = {}
    normalized_prefixes = tuple(prefix.rstrip("/") for prefix in excluded_prefixes)
    for raw in result.stdout.split(b"\0"):
        if not raw:
            continue
        relative = raw.decode("utf-8", errors="surrogateescape")
        if exempt_rebuild_caches and is_rebuild_cache_descendant(relative):
            continue
        if any(
            relative == prefix or relative.startswith(prefix + "/")
            for prefix in normalized_prefixes
        ):
            continue
        path = root / relative
        entries[relative] = sha256(path) if path.is_file() else "<missing>"
    return entries


def git_index_entries_digest(root: Path) -> str:
    """Hash semantic index entries while ignoring benign stat-cache rewrites."""
    result = subprocess.run(
        ["git", "-C", str(root), "ls-files", "--stage", "-v", "-z"],
        text=False,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        # Git keeps explicitly prunable worktree rows until an owner removes
        # them. Freeze that unavailable state instead of mutating shared Git
        # metadata during an executor snapshot. If the row becomes usable
        # before verify, its real digest differs from this sentinel.
        return "<unavailable>"
    return hashlib.sha256(result.stdout).hexdigest()


def registered_worktree_index_digests(root: Path) -> dict[str, str]:
    """Freeze semantic index entries/flags for every shared-repository worktree."""
    values: dict[str, str] = {}
    for worktree in registered_worktree_paths(root):
        if not worktree.is_dir():
            values[str(worktree)] = "<missing>"
            continue
        values[str(worktree)] = git_index_entries_digest(worktree)
    return values


def git_admin_manifest(
    root: Path,
    *,
    protect_raw_index: bool,
) -> dict[str, str]:
    """Freeze stable Git control bytes reachable from one Project worktree."""
    common = git_common_dir(root)
    admin = git_worktree_admin_dir(root)
    if common is None or admin is None:
        raise SystemExit(f"cannot resolve Git admin identity for {root}")

    entries: dict[str, str] = {}

    def record(label: str, path: Path) -> None:
        if path.is_symlink():
            entries[label] = "symlink:" + str(path.readlink())
        elif path.is_file():
            entries[label] = sha256(path)
        else:
            entries[label] = "<missing>"

    pointer = root / ".git"
    if pointer.is_file() or pointer.is_symlink():
        record("worktree:.git", pointer)
    else:
        entries["worktree:.git"] = "<directory>"

    for name in ("config", "HEAD", "packed-refs"):
        record(f"common:{name}", common / name)
    covered_directories = {"hooks", "info", "refs", "logs", "worktrees", "objects"}
    for child in sorted(common.iterdir()):
        if child.name in covered_directories:
            continue
        if child.name in {"index", "index.lock"}:
            if child.name == "index" and protect_raw_index:
                record("common:index", child)
            continue
        if child.name.startswith("fsmonitor--daemon") or re.fullmatch(
            r"[0-9a-f]{40}", child.name
        ):
            # Git's fsmonitor runtime endpoints/tokens are volatile cache state;
            # semantic index entries and flags are frozen separately.
            continue
        if child.is_file() or child.is_symlink():
            record(f"common:{child.name}", child)
            continue
        if child.is_dir():
            for path in sorted(
                item
                for item in child.rglob("*")
                if item.is_file() or item.is_symlink()
            ):
                record(
                    f"common:{path.relative_to(common).as_posix()}",
                    path,
                )
    for directory_name in ("hooks", "info", "refs", "logs"):
        directory = common / directory_name
        if not directory.is_dir():
            entries[f"common:{directory_name}/"] = "<missing>"
            continue
        for path in sorted(
            item
            for item in directory.rglob("*")
            if item.is_file() or item.is_symlink()
        ):
            relative_to_common = path.relative_to(common).as_posix()
            if relative_to_common.startswith("refs/codex/turn-diffs/"):
                # Codex Desktop rotates controller-local turn snapshots while
                # the executor is running. They are not repository refs and
                # cannot grant the worker Git authority.
                continue
            record(
                f"common:{relative_to_common}",
                path,
            )
    worktree_admins = common / "worktrees"
    if not worktree_admins.is_dir():
        entries["common:worktrees/"] = "<missing>"
    else:
        for path in sorted(
            item
            for item in worktree_admins.rglob("*")
            if item.is_file() or item.is_symlink()
        ):
            relative = path.relative_to(worktree_admins)
            if (
                relative.name in {"index", "index.lock"}
            ):
                continue
            record(
                f"common:worktrees/{relative.as_posix()}",
                path,
            )

    if admin == common:
        pass
    else:
        for path in sorted(
            item
            for item in admin.rglob("*")
            if item.is_file() or item.is_symlink()
        ):
            relative = path.relative_to(admin).as_posix()
            if not protect_raw_index and (
                relative == "index"
                or relative == "index.lock"
            ):
                continue
            record(f"admin:{relative}", path)
    return entries


def git_common_objects_digest(root: Path) -> str:
    """Hash every shared Git object byte once per snapshot/verification phase."""
    common = git_common_dir(root)
    if common is None:
        raise SystemExit(f"cannot resolve Git object store for {root}")
    objects = common / "objects"
    if not objects.is_dir():
        raise SystemExit(f"Git object store is missing: {objects}")
    digest = hashlib.sha256()
    for path in sorted(
        item
        for item in objects.rglob("*")
        if item.is_file() or item.is_symlink()
    ):
        relative = path.relative_to(objects).as_posix()
        digest.update(relative.encode(errors="surrogateescape"))
        digest.update(b"\0")
        if path.is_symlink():
            digest.update(b"symlink:")
            digest.update(str(path.readlink()).encode(errors="surrogateescape"))
        else:
            digest.update(b"file:")
            digest.update(sha256(path).encode())
        digest.update(b"\0")
    return digest.hexdigest()


def repository_worktree_baseline(
    root: Path,
    *,
    excluded_ignored_prefixes: tuple[str, ...] = (),
    exempt_rebuild_caches: bool = True,
    protect_raw_index: bool = True,
) -> dict:
    """Capture exact tracked/untracked bytes and Git identity for one worktree."""
    status = subprocess.run(
        ["git", "-C", str(root), "status", "--porcelain=v1", "--untracked-files=all"],
        text=True,
        capture_output=True,
        check=True,
    )
    head = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "HEAD"],
        text=True,
        capture_output=True,
        check=True,
    )
    return {
        "head": head.stdout.strip(),
        "git_admin_manifest": git_admin_manifest(
            root,
            protect_raw_index=protect_raw_index,
        ),
        "git_index_entries_digest": git_index_entries_digest(root),
        "git_status": status.stdout,
        "ignored_paths": git_ignored_paths(root),
        "ignored_noncache_manifest": ignored_noncache_manifest(
            root,
            excluded_prefixes=excluded_ignored_prefixes,
            exempt_rebuild_caches=exempt_rebuild_caches,
        ),
        "tracked_manifest": tracked_manifest(root),
        "untracked_manifest": untracked_manifest(root),
    }


def in_project_sibling_worktrees(profile: dict) -> list[Path]:
    """List other linked worktrees physically visible inside the AGY Project."""
    root = Path(profile["root"]).resolve()
    scope = agy_project_root(profile)
    siblings = []
    for candidate in registered_worktree_paths(scope):
        if (
            candidate not in (scope, root)
            and candidate.is_relative_to(scope)
            and candidate.is_dir()
        ):
            siblings.append(candidate)
    return sorted(dict.fromkeys(siblings))


def sibling_worktree_baselines(profile: dict) -> dict[str, dict]:
    """Freeze every sibling worktree the shared Project could mutate."""
    return {
        str(root): repository_worktree_baseline(
            root,
            exempt_rebuild_caches=False,
        )
        for root in in_project_sibling_worktrees(profile)
    }


def project_scope_baseline(profile: dict) -> dict | None:
    """Capture a read-only baseline for a shared persistent Project root."""
    root = Path(profile["root"]).resolve()
    scope = agy_project_root(profile)
    if scope == root:
        return None
    excluded = tuple(
        str(path.relative_to(scope))
        for path in registered_worktree_paths(scope)
        if path != scope and path.is_relative_to(scope)
    )
    return repository_worktree_baseline(
        scope,
        excluded_ignored_prefixes=excluded,
        exempt_rebuild_caches=False,
    )


def stable_git_admin_manifest(manifest: dict) -> dict:
    """Drop controller-local Codex turn snapshots from baseline comparisons."""
    return {
        key: value
        for key, value in manifest.items()
        if not key.startswith("common:refs/codex/turn-diffs/")
    }


def stable_worktree_baseline(baseline: dict) -> dict:
    """Normalize an old or new worktree baseline for stable comparison."""
    normalized = dict(baseline)
    admin = normalized.get("git_admin_manifest")
    if isinstance(admin, dict):
        normalized["git_admin_manifest"] = stable_git_admin_manifest(admin)
    return normalized


def assert_project_scope_unchanged(profile: dict, snapshot_data: dict) -> None:
    baseline = snapshot_data.get("project_scope_baseline")
    if baseline is None:
        return
    current = project_scope_baseline(profile)
    if stable_worktree_baseline(current) != stable_worktree_baseline(baseline):
        raise SystemExit(
            "VOID: persistent AGY Project worktree changed after snapshot; "
            "task worktrees may be added for dispatch but the Project root is read-only"
        )


def assert_sibling_worktrees_unchanged(profile: dict, snapshot_data: dict) -> None:
    baseline = snapshot_data.get("sibling_worktree_baselines")
    if not isinstance(baseline, dict):
        raise SystemExit(
            "VOID: snapshot lacks sibling-worktree baselines; create a fresh snapshot"
        )
    current = sibling_worktree_baselines(profile)
    stable_current = {
        path: stable_worktree_baseline(value)
        for path, value in current.items()
    }
    stable_baseline = {
        path: stable_worktree_baseline(value)
        for path, value in baseline.items()
    }
    if stable_current != stable_baseline:
        raise SystemExit(
            "VOID: an in-Project sibling worktree or its bytes changed after snapshot"
        )


def assert_task_ignored_noncache_unchanged(
    profile: dict,
    snapshot_data: dict,
) -> None:
    baseline = snapshot_data.get("task_worktree_baseline")
    expected = (
        baseline.get("ignored_noncache_manifest")
        if isinstance(baseline, dict)
        else None
    )
    if not isinstance(expected, dict):
        raise SystemExit(
            "VOID: snapshot lacks ignored non-cache byte hashes; create a fresh snapshot"
        )
    current = ignored_noncache_manifest(Path(profile["root"]).resolve())
    if current != expected:
        raise SystemExit("VOID: ignored non-cache task-worktree bytes changed")


def assert_task_git_admin_unchanged(profile: dict, snapshot_data: dict) -> None:
    """Protect task/common Git control state without binding index stat cache."""
    baseline = snapshot_data.get("task_worktree_baseline")
    expected_admin = (
        baseline.get("git_admin_manifest")
        if isinstance(baseline, dict)
        else None
    )
    expected_index = (
        baseline.get("git_index_entries_digest")
        if isinstance(baseline, dict)
        else None
    )
    if not isinstance(expected_admin, dict) or not isinstance(expected_index, str):
        raise SystemExit(
            "VOID: snapshot lacks task Git-admin baselines; create a fresh snapshot"
        )
    root = Path(profile["root"]).resolve()
    current_admin = git_admin_manifest(root, protect_raw_index=False)
    current_index = git_index_entries_digest(root)
    if (
        stable_git_admin_manifest(current_admin)
        != stable_git_admin_manifest(expected_admin)
        or current_index != expected_index
    ):
        raise SystemExit("VOID: task or shared Git administrative state changed")


def snapshot_worktree_heads(snapshot_data: dict) -> list[str]:
    """Return every frozen worktree HEAD that must remain readable."""
    baselines = [
        snapshot_data.get("task_worktree_baseline"),
        snapshot_data.get("project_scope_baseline"),
        *(
            snapshot_data.get("sibling_worktree_baselines", {}).values()
            if isinstance(snapshot_data.get("sibling_worktree_baselines"), dict)
            else []
        ),
    ]
    heads = sorted(
        {
            baseline.get("head")
            for baseline in baselines
            if isinstance(baseline, dict) and isinstance(baseline.get("head"), str)
        }
    )
    if not heads or any(
        re.fullmatch(r"(?:[0-9a-f]{40}|[0-9a-f]{64})", head) is None
        for head in heads
    ):
        raise SystemExit(
            "VOID: snapshot lacks valid frozen worktree HEAD objects"
        )
    return heads


def assert_git_common_objects_intact(profile: dict, snapshot_data: dict) -> None:
    """Allow append-only shared objects while preserving frozen HEAD graphs."""
    root = Path(profile["root"]).resolve()
    heads = snapshot_worktree_heads(snapshot_data)
    result = subprocess.run(
        [
            "git",
            "-C",
            str(root),
            "fsck",
            "--connectivity-only",
            "--no-dangling",
            "--no-reflogs",
            *heads,
        ],
        text=True,
        capture_output=True,
        check=False,
    )
    if result.returncode:
        raise SystemExit(
            "VOID: a frozen worktree HEAD object graph is missing or corrupt"
        )


def assert_registered_worktree_indexes_unchanged(
    profile: dict,
    snapshot_data: dict,
) -> None:
    expected = snapshot_data.get("registered_worktree_index_digests")
    if not isinstance(expected, dict):
        raise SystemExit(
            "VOID: snapshot lacks registered-worktree index digests; "
            "create a fresh snapshot"
        )
    current = registered_worktree_index_digests(Path(profile["root"]).resolve())
    if current != expected:
        raise SystemExit("VOID: a registered worktree Git index changed")


def is_rebuild_cache_path(path: str) -> bool:
    """Return whether an ignored path is a compiler/interpreter rebuild cache.

    These caches are produced by controller-owned acceptance gates as well as
    by an allowed worker build. They carry no candidate source and are already
    excluded from the repository status/manifest contract. Keep the exception
    deliberately narrow: task-local UV virtual environments are included so a
    controller EC gate can create its declared Python environment between
    same-ticket AGY revisions; every other ignored path still proves drift.
    """
    normalized = path.rstrip("/")
    return (
        normalized == "target"
        or normalized.endswith("/__pycache__")
        or normalized == ".venv"
        or normalized.endswith("/.venv")
    )


def assert_ignored_paths_unchanged(root: Path, snapshot_data: dict) -> None:
    """Fail closed on ignored drift except narrow reproducible build caches."""
    baseline = snapshot_data.get("ignored_paths")
    if not isinstance(baseline, list) or not all(
        isinstance(path, str) and path for path in baseline
    ):
        raise SystemExit(
            "VOID: snapshot lacks an ignored-path baseline; create a fresh snapshot"
        )
    expected = set(baseline)
    actual = set(git_ignored_paths(root))
    added = sorted(
        path for path in actual - expected if not is_rebuild_cache_path(path)
    )
    removed = sorted(
        path for path in expected - actual if not is_rebuild_cache_path(path)
    )
    if added or removed:
        details = []
        if added:
            details.append("added " + ", ".join(added))
        if removed:
            details.append("removed " + ", ".join(removed))
        raise SystemExit("VOID: ignored repository path drift: " + "; ".join(details))


def snapshot(profile: dict, task_key: str) -> Path:
    with project_concurrency_lock(profile, task_key, "snapshot"):
        with task_operation_lock(profile, task_key, "snapshot"):
            return snapshot_under_lock(profile, task_key)


def snapshot_under_lock(profile: dict, task_key: str) -> Path:
    readiness = require_project_ready(profile)
    task_state = frozen_task_state(profile, task_key)
    root = Path(profile["root"])
    state = Path(profile["state_dir"])
    oracle = state / "oracles" / f"{task_key}.md"
    if not oracle.is_file():
        raise SystemExit("refusing snapshot: create the oracle first")
    (state / "snapshots").mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        ["git", "-C", str(root), "status", "--porcelain=v1", "--untracked-files=all"],
        text=True,
        capture_output=True,
        check=True,
    )
    if result.stdout:
        raise SystemExit(
            "refusing snapshot from a dirty worktree; create a clean isolated root"
        )
    artifact_hashes = {
        entry["path"]: sha256(Path(entry["path"]))
        for entry in profile["protected_artifacts"]
    }
    protected_contents = {
        entry["path"]: base64.b64encode(Path(entry["path"]).read_bytes()).decode()
        for entry in profile["protected_artifacts"]
    }
    writable_contents = {}
    for relative in profile["allowed_repo_writes"]:
        path = root / relative
        writable_contents[relative] = (
            path.read_text(errors="surrogateescape") if path.is_file() else None
        )
    conversation_id = conversation_id_for_task(profile, task_key)
    verified_predecessor = None
    if conversation_id:
        verified_predecessor = assert_verified_predecessor(
            profile,
            task_key,
            conversation_id,
        )
    payload = {
        "task_key": task_key,
        "session_policy": task_session_policy(profile),
        "issue": profile["task_contract"].get("issue"),
        "at": datetime.now(timezone.utc).isoformat(),
        "oracle_sha256": sha256(oracle),
        "git_status": result.stdout,
        "ignored_paths": git_ignored_paths(root),
        "task_worktree_baseline": repository_worktree_baseline(
            root,
            protect_raw_index=False,
        ),
        "manifest": manifest(root, profile["snapshot_paths"]),
        "protected_artifacts": artifact_hashes,
        "protected_contents_base64": protected_contents,
        "writable_contents": writable_contents,
        "dispatch_contract": dispatch_contract(profile),
        "backend_resolution": profile["backend_resolution"],
        "agy_project_id": agy_project_id(profile),
        "agy_project_root": str(agy_project_root(profile)),
        "worktree_scope": worktree_scope_report(profile),
        "registered_worktree_index_digests": (
            registered_worktree_index_digests(root)
        ),
        "project_scope_baseline": project_scope_baseline(profile),
        "sibling_worktree_baselines": sibling_worktree_baselines(profile),
        "permission_state_digest": readiness["permission_state_digest"],
        "conversation_id": conversation_id,
        "conversation_step_floor": (
            verified_predecessor["conversation_step_max"]
            if verified_predecessor is not None
            else -1
        ),
        "conversation_predecessor_digest": (
            verified_predecessor["conversation_steps_digest"]
            if verified_predecessor is not None
            else None
        ),
        "attempt_predecessor_ordinal": (
            verified_predecessor["attempt_ordinal_max"]
            if verified_predecessor is not None
            else None
        ),
        "attempt_predecessor_digest": (
            verified_predecessor["attempt_lineage_digest"]
            if verified_predecessor is not None
            else None
        ),
        "task_state": task_state_contract(profile, task_state),
        "task_state_contract_digest": json_digest(
            task_state_contract(profile, task_state)
        ),
        "live_issue": (
            task_state_contract(profile, task_state)
            if task_session_policy(profile) == "ticketed"
            else None
        ),
    }
    payload["snapshot_id"] = json_digest(payload)
    encoded = json.dumps(payload, indent=2) + "\n"
    history = state / "snapshots" / "history" / task_key
    history.mkdir(parents=True, exist_ok=True)
    immutable = history / f"{payload['snapshot_id']}.json"
    if immutable.exists() and immutable.read_text() != encoded:
        raise SystemExit("snapshot history collision; refusing overwrite")
    if not immutable.exists():
        immutable.write_text(encoded)
    output = state / "snapshots" / f"{task_key}.json"
    output.write_text(encoded)
    print(output)
    return output


def render_prompt(
    profile: dict,
    task_key: str,
    oracle_text: str,
    task_state: dict,
    *,
    continuation: bool = False,
) -> str:
    writes = profile["allowed_repo_writes"] or ["none"]
    allowed = profile["task_commands"].get("allow", [])
    denied = profile["task_commands"].get("deny", [])
    expected_soft_denied = profile["task_contract"].get(
        "expected_soft_denied_commands", []
    )
    negative_control_block = (
        "\n".join(f"- {command}" for command in expected_soft_denied)
        or "- none"
    )
    injections: list[str] = []
    task_instructions = str(
        profile["task_contract"].get("instructions", "")
    ).strip()
    if task_instructions:
        injections.append(
            "Task-local controller instruction (frozen by profile):\n"
            + task_instructions
        )
    inject_prompt_file = profile.get("inject_prompt_file")
    if inject_prompt_file:
        prompt_file_instruction = Path(inject_prompt_file).read_text().strip()
        if prompt_file_instruction:
            injections.append(prompt_file_instruction)
    injection = "\n\n".join(injections)
    policy = task_session_policy(profile)
    if continuation:
        phase = (
            "Continue the same bounded ticket conversation. Inspect the current "
            "working diff and complete only unfinished injected criteria."
        )
    elif policy == "ticketed":
        phase = "Execute exactly one bounded ticket."
    else:
        phase = (
            "Execute exactly one bounded one-shot task. This session will not "
            "be resumed."
        )
    identity = (
        f"Ticket: #{task_key}."
        if policy == "ticketed"
        else f"One-shot run id: {task_key}."
    )
    state_label = (
        "Controller-validated live ticket snapshot"
        if policy == "ticketed"
        else "Controller-frozen one-shot task"
    )
    design_inputs = profile["task_contract"].get("design_inputs", [])
    design_block = (
        "\n".join(
            f"- {entry['path']} (sha256 {entry['sha256']})"
            for entry in design_inputs
        )
        or "- none; this is a read-only evidence task"
    )
    return f"""{phase} {identity}
Repository root: {profile['root']}
Required shell working directory: {agy_launch_cwd(profile)}
GitHub repo: {profile['repo']}
AGY Project: {agy_project_id(profile)}
AGY Project scope: {agy_project_root(profile)}
Task kind: {profile['task_contract']['kind']}
Session policy: {policy}
Persistent AGY permission-state digest: {permission_state_digest(profile)}

{state_label}:

```json
{json.dumps(task_state, indent=2)}
```

Frozen design inputs:
{design_block}

You are a bounded executor, not the project owner. The controller validated
the frozen task and the persistent AGY Project policy immediately before this
run. The Project may grant broader reusable tool access than this task: that
does not widen this task. Obey the task-local command and write allowlists
below; the controller will void any out-of-scope repository mutation.

Exact repository write allowlist:
{chr(10).join(f"- {path}" for path in writes)}

Exact shell command lines authorized for this task:
{chr(10).join(f"- {command}" for command in allowed) or "- none"}

Shell command lines explicitly forbidden for this task:
{chr(10).join(f"- {command}" for command in denied) or "- none"}

Expected soft-deny negative controls:
{negative_control_block}

Run normal authorized commands first. Then attempt each listed negative-control
command exactly once. Do not replace or retry a denied command. A denial is the
expected transport result, not permission to change the command.

Do not change branches, create worktrees, commit, push, mutate a tracker, or
write any repository path outside the exact allowlist. Read-only git commands
are permitted only when explicitly listed above. Every shell tool call must
run with its recorded Cwd exactly equal to the required shell working
directory above; do not `cd` to the persistent Project root or another
worktree. Use absolute paths. If a
command is unavailable, a path is missing, or the task conflicts with the
injected contract, stop and report FAIL instead of improvising.

Every Bash tool call must copy one authorized command line byte-for-byte. A
narrower `sed` range, reordered flag, changed quote or escape form, appended
pipeline, or semantically equivalent helper is a different command. Use the
built-in read-file tool for additional source inspection; do not synthesize a
new shell command.

## Controller oracle (injected, immutable)

{oracle_text.strip()}

## Round-specific controller injection

{injection or "(none)"}

Your terminal answer must end with one final line-anchored `## EXEC REPORT`
block containing PASS or FAIL per criterion, exact changed paths, runnable
selectors, witnesses, and unfinished steps. Progress chatter may precede that
block; the dispatcher normalizes the last report marker. Do not claim a command
you were denied or did not run. A PASS is provisional until controller
verification. Any tracker comment is optional unverified input; the local report
is mandatory. The controller independently verifies every claim and alone
decides acceptance, publication, and any closure.
"""


def conversation_log_mentions(path: Path) -> tuple[list[str], list[str], list[str]]:
    """Return all, creation-event, and standalone AGY conversation ids."""
    if not path.exists():
        return [], [], []
    text = path.read_text(errors="replace")
    uuid = r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
    all_mentions = re.findall(
        rf"(?:Created conversation\s+|conversation=)({uuid})",
        text,
    )
    created = re.findall(
        rf"server\.go:\d+\]\s+Created conversation\s+({uuid})\s*$",
        text,
        flags=re.MULTILINE,
    )
    standalone = re.findall(
        rf"^conversation=({uuid})\s*$",
        text,
        flags=re.MULTILINE,
    )
    return all_mentions, created, standalone


def conversation_id_mentioned_in_log(path: Path) -> str | None:
    """Bind a resume to the one consistent conversation id in its AGY log."""
    all_mentions, _, _ = conversation_log_mentions(path)
    if len(set(all_mentions)) > 1:
        raise SystemExit("VOID: AGY log contains conflicting conversation ids")
    return all_mentions[0] if all_mentions else None


def conversation_id_from_log(path: Path) -> str | None:
    all_mentions, created, standalone = conversation_log_mentions(path)
    if len(set(all_mentions)) > 1:
        raise SystemExit("VOID: AGY log contains conflicting conversation ids")
    if len(set(created)) > 1:
        raise SystemExit("VOID: AGY log contains conflicting creation events")
    if created:
        if all_mentions and any(value != created[0] for value in all_mentions):
            raise SystemExit("VOID: AGY log conversation lineage is inconsistent")
        return created[0]
    if len(set(standalone)) == 1 and standalone:
        if all_mentions and any(value != standalone[0] for value in all_mentions):
            raise SystemExit("VOID: AGY log conversation lineage is inconsistent")
        return standalone[0]
    return None


def extract_exec_report(raw: str) -> str | None:
    stripped = raw.lstrip()
    markers = list(re.finditer(r"^## EXEC REPORT", stripped, flags=re.MULTILINE))
    if not markers:
        return None
    return stripped[markers[-1].start() :].lstrip()


def extract_run_command_requests(payload: bytes) -> list[dict[str, str | None]]:
    text = payload.decode(errors="replace")
    decoder = json.JSONDecoder()
    requests = []
    cursor = 0

    def collect(value: object) -> None:
        if isinstance(value, dict):
            command = value.get("CommandLine")
            if isinstance(command, str):
                cwd = value.get("Cwd")
                requests.append(
                    {
                        "command": command,
                        "cwd": cwd if isinstance(cwd, str) else None,
                    }
                )
            for child in value.values():
                collect(child)
        elif isinstance(value, list):
            for child in value:
                collect(child)

    while cursor < len(text):
        start = text.find("{", cursor)
        if start < 0:
            break
        try:
            value, consumed = decoder.raw_decode(text[start:])
        except json.JSONDecodeError:
            cursor = start + 1
            continue
        cursor = start + consumed
        collect(value)
    return requests


def extract_run_command_lines(payload: bytes) -> list[str]:
    """Compatibility view used by the denied-command reporter."""
    return [item["command"] for item in extract_run_command_requests(payload)]


def run_command_marker_count(payload: bytes) -> int:
    """Count canonical or JSON-escaped CommandLine keys, including bad objects."""
    count = 0
    for match in re.finditer(rb'"(?:\\.|[^"\\])*"\s*:', payload):
        key_token = match.group(0).rsplit(b":", 1)[0].strip()
        try:
            key = json.loads(key_token.decode())
        except (UnicodeDecodeError, json.JSONDecodeError):
            continue
        if key == "CommandLine":
            count += 1
    return count


def requested_run_commands(
    conversation_id: str,
    *,
    after_step: int = -1,
) -> list[dict]:
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(
            f"conversation state is missing for {conversation_id}: {database}"
        )
    connection = readonly_conversation_connection(database)
    try:
        schema = connection.execute("pragma table_info(steps)").fetchall()
        column_names = [str(column[1]) for column in schema]
        required_columns = {"idx", "step_type", "status"}
        if not required_columns.issubset(column_names):
            raise SystemExit("VOID: AGY conversation steps schema is unsupported")
        negative = connection.execute(
            "select idx from steps where idx < 0 limit 1"
        ).fetchone()
        if negative is not None:
            raise SystemExit("VOID: AGY conversation contains a negative step index")
        rows = connection.execute("select * from steps order by idx").fetchall()
    finally:
        connection.close()
    commands = []
    # AGY records the authoritative shell request in step type 15, then may
    # replay the same request in later lifecycle/result rows (for example
    # types 21 and 132).  Audit each authoritative request once while refusing
    # any command identity that appears only on a non-request row.
    authoritative_requests: set[tuple[str, str | None]] = set()
    idx_position = column_names.index("idx")
    type_position = column_names.index("step_type")
    status_position = column_names.index("status")
    subtrajectory_position = (
        column_names.index("has_subtrajectory")
        if "has_subtrajectory" in column_names
        else None
    )
    for row in rows:
        idx = int(row[idx_position])
        step_type = int(row[type_position])
        status = int(row[status_position])
        if subtrajectory_position is not None:
            has_subtrajectory = row[subtrajectory_position]
            false_values = (None, False, 0, "", "0", b"", b"0", b"\x00")
            if has_subtrajectory not in false_values:
                raise SystemExit(
                    "VOID: AGY conversation step references an unaudited "
                    f"subtrajectory at step {idx}"
                )
        is_post_snapshot = idx > after_step
        if not is_post_snapshot and step_type != 15:
            continue
        request_surfaces: list[tuple[str, list[dict[str, str | None]]]] = []
        command_like_surface = False
        for column_name, value in zip(column_names, row):
            if not isinstance(value, (bytes, str)):
                continue
            raw_value = value.encode() if isinstance(value, str) else value
            markers = run_command_marker_count(raw_value)
            requests = extract_run_command_requests(raw_value)
            if not markers and not requests:
                continue
            command_like_surface = True
            if len(requests) != markers:
                raise SystemExit(
                    "VOID: cannot completely parse every AGY shell command and "
                    f"Cwd from conversation step {idx} column {column_name}: "
                    f"{len(requests)} parsed for {markers} command marker(s)"
                )
            request_surfaces.append((column_name, requests))
        if not request_surfaces:
            # Step type 15 is a general AGY tool/request row.  A row with no
            # CommandLine key is not a shell request; any command-like marker
            # was already required above to parse completely and would not
            # reach this branch.
            if command_like_surface:
                raise SystemExit(
                    "VOID: cannot completely parse every AGY shell command and "
                    f"Cwd from conversation step {idx}"
                )
            continue
        # Payload and metadata can repeat the same request a different number
        # of times.  Their unique command/Cwd identities must still agree.
        surface_identities = [
            {
                (request["command"], request["cwd"])
                for request in requests
            }
            for _, requests in request_surfaces
        ]
        identities = surface_identities[0]
        mismatched_surfaces = [
            column_name
            for (column_name, _), candidate in zip(
                request_surfaces[1:],
                surface_identities[1:],
            )
            if candidate != identities
        ]
        if mismatched_surfaces:
            raise SystemExit(
                "VOID: AGY shell command records disagree across conversation "
                f"step {idx} surfaces: " + ", ".join(mismatched_surfaces)
            )
        if int(step_type) != 15:
            unexpected = identities - authoritative_requests
            if unexpected:
                rendered = ", ".join(
                    f"{command!r} cwd={cwd!r}"
                    for command, cwd in sorted(
                        unexpected,
                        key=lambda item: (item[0], item[1] or ""),
                    )
                )
                raise SystemExit(
                    "VOID: AGY shell command appeared only in unsupported "
                    f"conversation step type {step_type} at step {idx}: "
                    + rendered
                )
            continue
        if not identities:
            raise SystemExit(
                "VOID: AGY request step contains no auditable shell command at "
                f"step {idx}"
            )
        authoritative_requests.update(identities)
        if not is_post_snapshot:
            continue
        commands.extend(
            {
                "step": int(idx),
                "status": int(status),
                "command": command,
                "cwd": cwd,
            }
            for command, cwd in sorted(
                identities,
                key=lambda item: (item[0], item[1] or ""),
            )
        )
    return commands


def assert_snapshot_conversation_predecessor_unchanged(
    conversation_id: str | None,
    snapshot_data: dict,
) -> None:
    """Protect the exact pre-floor rows already covered by prior verification."""
    snapshot_conversation = snapshot_data.get("conversation_id")
    if not snapshot_conversation:
        return
    if conversation_id != snapshot_conversation:
        raise SystemExit("VOID: snapshot conversation lineage changed")
    floor = snapshot_data.get("conversation_step_floor")
    expected = snapshot_data.get("conversation_predecessor_digest")
    if not isinstance(floor, int) or not isinstance(expected, str):
        raise SystemExit(
            "VOID: snapshot lacks a verified predecessor row digest; "
            "create a fresh snapshot only after verification"
        )
    current = conversation_steps_digest(
        snapshot_conversation,
        through_step=floor,
    )
    if current != expected:
        raise SystemExit("VOID: verified pre-snapshot conversation rows changed")


def audit_task_commands(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> list[dict]:
    current_id = conversation_id_for_task(profile, task_key)
    snapshot_id = snapshot_data.get("conversation_id")
    if snapshot_id and current_id != snapshot_id:
        raise SystemExit(
            "VOID: task conversation changed after snapshot: "
            f"{snapshot_id} -> {current_id or '<missing>'}"
        )
    assert_snapshot_conversation_predecessor_unchanged(
        current_id,
        snapshot_data,
    )
    if not current_id:
        return []
    commands = requested_run_commands(
        current_id,
        after_step=int(snapshot_data.get("conversation_step_floor", -1)),
    )
    allowed = set(profile["task_commands"]["allow"])
    denied_commands = set(profile["task_commands"]["deny"])
    forbidden = [item for item in commands if item["command"] in denied_commands]
    unlisted = [
        item
        for item in commands
        if item["command"] not in allowed
    ]
    required_cwd = agy_launch_cwd(profile)
    wrong_cwd = [
        item
        for item in commands
        if not item["cwd"]
        or item["cwd"] != str(required_cwd)
        or not Path(item["cwd"]).is_absolute()
        or Path(item["cwd"]).resolve() != required_cwd
    ]
    if forbidden or unlisted or wrong_cwd:
        details = {
            "forbidden": forbidden,
            "unlisted": unlisted,
            "wrong_cwd": wrong_cwd,
            "required_cwd": str(required_cwd),
        }
        raise SystemExit(
            "VOID: AGY requested a shell command outside the task-local "
            "exact allowlist or task-worktree cwd: "
            + json.dumps(details, sort_keys=True)
        )
    return commands


def expected_soft_denial_failures(
    profile: dict,
    audited_commands: list[dict],
    *,
    inspected: dict | None = None,
    guard_events: list[dict] | None = None,
) -> list[dict]:
    """Require each frozen negative control to be denied in raw transport.

    AGY's local conversation database records the command request, but its
    status value does not reliably distinguish a pre-tool hook denial from an
    executed command. The stream ERROR and the frozen guard audit are the
    authoritative execution witnesses when they are available.
    """
    failures: list[dict] = []
    expected = profile["task_contract"].get(
        "expected_soft_denied_commands",
        [],
    )
    for command in expected:
        matches = [
            item for item in audited_commands if item.get("command") == command
        ]
        if len(matches) != 1:
            failures.append(
                {
                    "command": command,
                    "reason": "attempt-count",
                    "actual_count": len(matches),
                }
            )
            continue
        if inspected is None or guard_events is None:
            if matches[0].get("status") == AGY_STEP_STATUS_DENIED:
                continue
            failures.append(
                {
                    "command": command,
                    "reason": "executed-instead-of-denied",
                    "status": matches[0].get("status"),
                    "step": matches[0].get("step"),
                }
            )
            continue
        streamed = [
            call
            for call in inspected.get("tool_calls", [])
            if isinstance(call, dict) and call.get("command") == command
        ]
        if len(streamed) != 1:
            failures.append(
                {
                    "command": command,
                    "reason": "raw-stream-attempt-count",
                    "actual_count": len(streamed),
                }
            )
            continue
        step = streamed[0].get("step_index")
        all_calls = inspected.get("tool_calls")
        all_steps = (
            [item.get("step_index") for item in all_calls if isinstance(item, dict)]
            if isinstance(all_calls, list)
            else []
        )
        if not isinstance(step, int) or not all_steps or step != max(all_steps):
            failures.append(
                {
                    "command": command,
                    "reason": "raw-stream-not-last",
                    "step": step,
                }
            )
            continue
        denied_steps = inspected.get("denied_tool_steps", [])
        if not isinstance(step, int) or step not in denied_steps:
            failures.append(
                {
                    "command": command,
                    "reason": "raw-stream-not-denied",
                    "step": step,
                }
            )
            continue
        matching_denials = [
            event
            for event in guard_events
            if event.get("step_index") == step
            and event.get("command") == command
            and event.get("decision") == "deny"
        ]
        if not matching_denials:
            failures.append(
                {
                    "command": command,
                    "reason": "assignment-guard-not-denied",
                    "step": step,
                }
            )
    return failures


def assert_no_sandbox_file_access_denial(profile: dict, task_key: str) -> None:
    """Reject a candidate when AGY's terminal sandbox could not read its task root.

    An allowed command that is denied by the macOS terminal sandbox has not
    produced the required task evidence. It must not be misclassified as a
    missing Project permission or silently accepted because the conversation
    database did not persist the attempted command.
    """
    log_dir = Path(profile["state_dir"]) / "runs"
    for log in log_dir.glob(f"{task_key}*.agy.log"):
        text = log.read_text(errors="replace")
        if (
            "SANDBOX_COMMAND_BLOCKED" in text
            and "Operation not permitted" in text
        ):
            raise SystemExit(
                "VOID: AGY terminal sandbox denied task-root file access; "
                "keep allowNonWorkspaceAccess=false and the reviewed Global "
                "baseline, then use a fresh clean-worktree snapshot after fixing "
                "the Project/worktree binding"
            )


def denied(profile: dict, task_key: str) -> None:
    validate_task_key(profile, task_key)
    log_dir = Path(profile["state_dir"]) / "runs"
    conversation_path = log_dir / f"{task_key}.conversation"
    conversation_id = (
        conversation_path.read_text().strip() if conversation_path.exists() else None
    )
    if not conversation_id:
        raise SystemExit(
            f"cannot inspect {task_key}: no conversation id in state"
        )
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(f"cannot inspect {task_key}: missing {database}")
    connection = readonly_conversation_connection(database)
    try:
        rows = connection.execute(
            "select idx, step_payload from steps "
            "where status = 7 order by idx"
        ).fetchall()
    finally:
        connection.close()
    commands = [
        (idx, command)
        for idx, payload in rows
        for command in extract_run_command_lines(payload or b"")
    ]
    if not commands:
        raise SystemExit(
            f"no denied run_command payload found for {task_key}; "
            "inspect the AGY log"
        )
    for idx, command in commands:
        print(f"step {idx}: {command}")


def decode_snapshot(encoded: str, *, expected_id: str | None = None) -> dict:
    snapshot_data = json.loads(encoded)
    snapshot_id = snapshot_data.get("snapshot_id")
    unsigned = dict(snapshot_data)
    unsigned.pop("snapshot_id", None)
    if not isinstance(snapshot_id, str) or json_digest(unsigned) != snapshot_id:
        raise SystemExit("VOID: snapshot id/digest mismatch")
    if expected_id is not None and snapshot_id != expected_id:
        raise SystemExit("VOID: immutable snapshot identity mismatch")
    return snapshot_data


def immutable_snapshot_path(profile: dict, task_key: str, snapshot_id: str) -> Path:
    return (
        Path(profile["state_dir"])
        / "snapshots"
        / "history"
        / task_key
        / f"{snapshot_id}.json"
    )


def load_immutable_snapshot(
    profile: dict,
    task_key: str,
    snapshot_id: str,
) -> dict:
    immutable = immutable_snapshot_path(profile, task_key, snapshot_id)
    if not immutable.is_file():
        raise SystemExit("VOID: immutable snapshot history is missing")
    return decode_snapshot(immutable.read_text(), expected_id=snapshot_id)


def load_snapshot(profile: dict, task_key: str) -> dict:
    path = Path(profile["state_dir"]) / "snapshots" / f"{task_key}.json"
    if not path.is_file():
        raise SystemExit("missing pre-dispatch snapshot")
    encoded = path.read_text()
    snapshot_data = decode_snapshot(encoded)
    snapshot_id = snapshot_data["snapshot_id"]
    immutable = (
        immutable_snapshot_path(profile, task_key, snapshot_id)
    )
    if not immutable.is_file() or immutable.read_text() != encoded:
        raise SystemExit("VOID: immutable snapshot history is missing or changed")
    return snapshot_data


def assert_permission_state_unchanged(profile: dict, snapshot_data: dict) -> None:
    current = permission_state_digest(profile)
    expected = snapshot_data.get("permission_state_digest")
    if current != expected:
        raise SystemExit(
            "VOID: AGY Project/global permission state changed after snapshot; "
            "rerun doctor and create a fresh snapshot"
        )


def assert_snapshot_identity(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> None:
    snapshot_key = str(
        snapshot_data.get("task_key")
        or snapshot_data.get("issue")
        or ""
    )
    if snapshot_key != task_key:
        raise SystemExit(
            f"VOID: snapshot task identity changed: {snapshot_key} -> {task_key}"
        )
    snapshot_policy = snapshot_data.get("session_policy", "ticketed")
    current_policy = task_session_policy(profile)
    if snapshot_policy != current_policy:
        raise SystemExit(
            "VOID: snapshot session policy changed: "
            f"{snapshot_policy} -> {current_policy}"
        )


def assert_oracle_unchanged(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> Path:
    oracle = Path(profile["state_dir"]) / "oracles" / f"{task_key}.md"
    expected = snapshot_data.get("oracle_sha256")
    if not oracle.is_file() or not isinstance(expected, str):
        raise SystemExit(
            "VOID: oracle or its snapshot digest is missing; create a fresh snapshot"
        )
    if sha256(oracle) != expected:
        raise SystemExit("VOID: oracle changed after snapshot")
    return oracle


def assert_dispatch_contract_unchanged(
    profile: dict,
    snapshot_data: dict,
    *,
    allow_prompt_hash_change: bool = False,
) -> None:
    snapshot_contract = snapshot_data.get("dispatch_contract")
    if not isinstance(snapshot_contract, dict) or not snapshot_contract_matches(
        profile,
        snapshot_contract,
        allow_prompt_hash_change=allow_prompt_hash_change,
    ):
        raise SystemExit("VOID: dispatch contract changed after snapshot")
    if snapshot_data.get("agy_project_id") != agy_project_id(profile):
        raise SystemExit("VOID: AGY project changed after snapshot")
    if snapshot_data.get("agy_project_root") != str(agy_project_root(profile)):
        raise SystemExit("VOID: AGY Project scope changed after snapshot")
    if snapshot_data.get("backend_resolution") != profile["backend_resolution"]:
        raise SystemExit("VOID: backend resolution changed after snapshot")
    assert_worktree_scope_unchanged(profile, snapshot_data)


def assert_worktree_scope_unchanged(profile: dict, snapshot_data: dict) -> None:
    if snapshot_data.get("worktree_scope") != worktree_scope_report(profile):
        raise SystemExit("VOID: task worktree binding changed after snapshot")


def canonical_attempt_suffix(ordinal: int) -> str:
    if ordinal == 0:
        return ""
    if ordinal == 1:
        return ".resume"
    if ordinal >= 2:
        return f".resume.{ordinal}"
    raise ValueError("attempt ordinal must be non-negative")


def attempt_ordinal_from_suffix(suffix: str) -> int:
    if suffix == "":
        return 0
    if suffix == ".resume":
        return 1
    match = re.fullmatch(r"\.resume\.([0-9]+)", suffix)
    if match is None:
        raise SystemExit(f"VOID: noncanonical run-attempt suffix: {suffix!r}")
    number = match.group(1)
    ordinal = int(number)
    if ordinal < 2 or str(ordinal) != number:
        raise SystemExit(f"VOID: noncanonical run-attempt suffix: {suffix!r}")
    return ordinal


def canonical_attempt_files(
    profile: dict,
    task_key: str,
    ending: str,
) -> dict[int, Path]:
    """Resolve attempt artifacts by immutable ordinal, never mutable mtime."""
    log_dir = Path(profile["state_dir"]) / "runs"
    pattern = re.compile(
        rf"^{re.escape(task_key)}"
        rf"(?P<suffix>\.resume(?:\.(?P<number>[0-9]+))?)?"
        + re.escape(ending)
        + r"$"
    )
    values: dict[int, Path] = {}
    if not log_dir.is_dir():
        return values
    for path in log_dir.iterdir():
        name = path.name
        is_candidate = (
            name == f"{task_key}{ending}"
            or (
                name.startswith(f"{task_key}.resume")
                and name.endswith(ending)
            )
        )
        if not is_candidate:
            continue
        match = pattern.fullmatch(name)
        if match is None:
            raise SystemExit(f"VOID: noncanonical attempt artifact name: {path}")
        suffix = match.group("suffix")
        number = match.group("number")
        if suffix is None:
            ordinal = 0
        elif number is None:
            ordinal = 1
        else:
            ordinal = int(number)
            if ordinal < 2 or str(ordinal) != number:
                raise SystemExit(f"VOID: noncanonical attempt ordinal: {path}")
        if ordinal in values:
            raise SystemExit(f"VOID: duplicate attempt ordinal {ordinal}: {path}")
        values[ordinal] = path.resolve()
    return values


def assert_complete_attempt_lineage(profile: dict, task_key: str) -> list[int]:
    required_endings = (
        ".prompt.md",
        ".contract.json",
        ".agy.log",
        ".stream.jsonl",
        ".stderr.log",
        ".response.md",
        ".guard.py",
        ".guard-policy.json",
        ".guard-audit.jsonl",
        ".hooks.json",
        ".evidence.json",
    )
    inventories = {
        ending: canonical_attempt_files(profile, task_key, ending)
        for ending in required_endings
    }
    contracts = inventories[".contract.json"]
    evidence = inventories[".evidence.json"]
    if not contracts:
        raise SystemExit("VOID: no complete AGY run-attempt lineage exists")
    expected = set(contracts)
    mismatched = [
        ending
        for ending, values in inventories.items()
        if set(values) != expected
    ]
    if mismatched:
        raise SystemExit(
            "VOID: run attempts lack one-to-one canonical artifacts: "
            + ", ".join(mismatched)
        )
    ordinals = sorted(contracts)
    if ordinals != list(range(ordinals[-1] + 1)):
        raise SystemExit("VOID: run attempt ordinals are not contiguous")
    reports = canonical_attempt_files(profile, task_key, ".report.md")
    if not set(reports).issubset(expected):
        raise SystemExit("VOID: normalized report has no matching run attempt")
    for ordinal in ordinals:
        evidence_path = evidence[ordinal]
        try:
            value = json.loads(evidence_path.read_text())
        except (OSError, json.JSONDecodeError) as error:
            raise SystemExit(f"VOID: malformed run evidence: {evidence_path}: {error}")
        if not isinstance(value, dict):
            raise SystemExit(f"VOID: malformed run evidence: {evidence_path}")
        expected_suffix = canonical_attempt_suffix(ordinal)
        if (
            value.get("version") != RUN_EVIDENCE_VERSION
            or value.get("audit_contract_version") != AUDIT_CONTRACT_VERSION
            or value.get("suffix") != expected_suffix
            or value.get("attempt_ordinal") != ordinal
        ):
            raise SystemExit(
                f"VOID: run evidence attempt identity mismatch: {evidence_path}"
            )
        reported = value.get("delivery_status") == "reported"
        if (ordinal in reports) != reported:
            raise SystemExit(
                "VOID: normalized report inventory disagrees with delivery "
                f"status for attempt {ordinal}"
            )
    return ordinals


def attempt_lineage_digest(
    profile: dict,
    task_key: str,
    *,
    through_ordinal: int | None = None,
) -> str:
    """Hash every canonical artifact in ordinal order, independent of mtime."""
    ordinals = assert_complete_attempt_lineage(profile, task_key)
    if through_ordinal is not None:
        if through_ordinal not in ordinals:
            raise SystemExit(
                "VOID: attempt-lineage ceiling is absent from run artifacts"
            )
        ordinals = [
            ordinal for ordinal in ordinals if ordinal <= through_ordinal
        ]
    endings = (
        ".prompt.md",
        ".contract.json",
        ".agy.log",
        ".stream.jsonl",
        ".stderr.log",
        ".response.md",
        ".guard.py",
        ".guard-policy.json",
        ".guard-audit.jsonl",
        ".hooks.json",
        ".evidence.json",
        ".report.md",
    )
    inventories = {
        ending: canonical_attempt_files(profile, task_key, ending)
        for ending in endings
    }
    payload = []
    for ordinal in ordinals:
        files = []
        for ending in endings:
            path = inventories[ending].get(ordinal)
            if path is not None:
                files.append(
                    {
                        "name": path.name,
                        "sha256": sha256(path),
                    }
                )
        payload.append({"ordinal": ordinal, "files": files})
    return json_digest(payload)


def assert_snapshot_attempt_predecessor_unchanged(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> None:
    """Keep previously verified attempt artifacts immutable across a new round."""
    if not snapshot_data.get("conversation_id"):
        return
    ordinal = snapshot_data.get("attempt_predecessor_ordinal")
    expected = snapshot_data.get("attempt_predecessor_digest")
    if not isinstance(ordinal, int) or not isinstance(expected, str):
        raise SystemExit(
            "VOID: snapshot lacks verified attempt-artifact predecessor lineage"
        )
    current = attempt_lineage_digest(
        profile,
        task_key,
        through_ordinal=ordinal,
    )
    if current != expected:
        raise SystemExit(
            "VOID: verified predecessor attempt artifacts changed after snapshot"
        )


def latest_round_contract(
    profile: dict,
    task_key: str,
) -> tuple[Path, dict] | None:
    candidates = canonical_attempt_files(profile, task_key, ".contract.json")
    if not candidates:
        return None
    path = candidates[max(candidates)]
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SystemExit(f"VOID: malformed round contract: {path}")
    return path, value


def assert_executed_round_contract(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> None:
    assert_complete_attempt_lineage(profile, task_key)
    latest = latest_round_contract(profile, task_key)
    if latest is None:
        raise SystemExit("VOID: executed round contract is missing")
    path, round_contract = latest
    snapshot_contract = snapshot_data.get("dispatch_contract")
    if not isinstance(snapshot_contract, dict):
        raise SystemExit("VOID: snapshot dispatch contract is missing")
    is_ticketed_resume = (
        ".resume" in path.name
        and task_session_policy(profile) == "ticketed"
    )
    if not dispatch_contracts_match(
        round_contract,
        snapshot_contract,
        allow_prompt_hash_change=is_ticketed_resume,
    ):
        raise SystemExit("VOID: executed round exceeded the snapshot contract")
    if round_contract != dispatch_contract(profile):
        raise SystemExit("VOID: profile changed after the executed round")


def permission_denial_text(value: str) -> bool:
    normalized = value.casefold()
    return any(
        marker in normalized
        for marker in (
            "permission_denied",
            "permission denied",
            "denied by policy",
            "denied by sandbox",
            "operation not permitted",
            "sandbox_command_blocked",
            "soft-deny",
            "soft denying",
            "soft-denying",
            "auto-denied",
            "denied by hook",
            "denied by pre-tool hook",
            "hook denied",
            "tool use denied",
        )
    )


def structured_tool_permission_denial(event: dict) -> bool:
    """Inspect only structured tool error values, never agent response prose."""
    if event.get("event") not in {
        "step_update",
        "tool",
        "tool_result",
        "tool_error",
    }:
        return False

    errors: list[object] = []

    def collect(value: object) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if key.casefold() in {"error", "tool_error", "permission_error"}:
                    errors.append(child)
                else:
                    collect(child)
        elif isinstance(value, list):
            for child in value:
                collect(child)

    collect(event)
    return any(
        permission_denial_text(
            json.dumps(error, sort_keys=True)
            if not isinstance(error, str)
            else error
        )
        for error in errors
    )


def structured_tool_permission_denial_steps(events: list[dict]) -> set[int]:
    """Return tool steps denied before a later PreToolUse hook can run."""
    denied: set[int] = set()
    for event in events:
        if not structured_tool_permission_denial(event):
            continue
        update = event.get("step_update")
        step = update.get("step_index") if isinstance(update, dict) else None
        if isinstance(step, int) and step >= 0:
            denied.add(step)
    return denied


def stream_tool_calls(events: list[dict]) -> tuple[list[dict], str | None]:
    """Return one canonical tool request per streamed step index."""
    by_step: dict[int, dict] = {}
    for event in events:
        if event.get("event") != "step_update":
            continue
        update = event.get("step_update")
        if not isinstance(update, dict):
            return [], "step_update payload must be an object"
        info = update.get("tool_info")
        info = info if isinstance(info, dict) else {}
        tool_name = update.get("tool_name") or info.get("name")
        if update.get("step_type") != "tool" and not isinstance(tool_name, str):
            continue
        step = update.get("step_index")
        if not isinstance(step, int) or step < 0 or not isinstance(tool_name, str):
            return [], "tool step index or name is invalid"
        args = info.get("parameters")
        if not isinstance(args, dict):
            args = info.get("args")
        if not isinstance(args, dict):
            args = update.get("parameters")
        if not isinstance(args, dict):
            if step in by_step:
                continue
            return [], f"tool step {step} has no request parameters"
        target = None
        for field in (
            *assignment_guard.READ_PATH_FIELDS.values(),
            *assignment_guard.WRITE_PATH_FIELDS.values(),
        ):
            if field in args:
                target = args[field]
                break
        canonical = {
            "step_index": step,
            "conversation_id": update.get("conversation_id"),
            "tool_name": tool_name,
            "tool_args": args,
            "tool_args_digest": assignment_guard.canonical_digest(args),
            "command": args.get("CommandLine"),
            "cwd": args.get("Cwd"),
            "target_path": target,
        }
        prior = by_step.get(step)
        if prior is not None and prior != canonical:
            return [], f"tool step {step} changed identity in the stream"
        by_step[step] = canonical
    return [by_step[step] for step in sorted(by_step)], None


def bind_implicit_run_command_cwd(tool_calls: list[dict], init_cwd: str) -> None:
    """Bind an omitted stream command Cwd to the already-verified init Cwd."""
    for call in tool_calls:
        if call.get("tool_name") != "run_command":
            continue
        args = call.get("tool_args")
        if not isinstance(args, dict) or "Cwd" in args:
            continue
        effective_args = {**args, "Cwd": init_cwd}
        call["tool_args"] = effective_args
        call["tool_args_digest"] = assignment_guard.canonical_digest(effective_args)
        call["cwd"] = init_cwd


def inspect_headless_stream(
    stream_path: Path,
    stderr_path: Path,
    *,
    expected_cwd: Path,
    requested_conversation_id: str | None,
    expected_permission_mode: str | None = None,
    exit_code: int = 0,
) -> dict:
    """Reparse raw headless transport and classify it without trusting evidence."""
    events: list[dict] = []
    parse_error = None
    try:
        lines = stream_path.read_text(errors="strict").splitlines()
    except (OSError, UnicodeError) as error:
        lines = []
        parse_error = str(error)
    for number, line in enumerate(lines, start=1):
        if not line.strip():
            parse_error = f"blank NDJSON record at line {number}"
            break
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            parse_error = f"invalid NDJSON at line {number}: {error}"
            break
        if not isinstance(value, dict) or not isinstance(value.get("event"), str):
            parse_error = f"invalid event envelope at line {number}"
            break
        events.append(value)

    response = ""
    init_id = None
    result_id = None
    result_status = None
    init_cwd = None
    init_permission_mode = None
    tool_calls: list[dict] = []
    denied_tool_steps: set[int] = set()
    status = "invalid-stream"
    if parse_error is None:
        cwd_mismatch = False
        permission_mode_mismatch = False
        if events and events[0].get("event") == "init":
            first_init = events[0].get("init")
            init_id = events[0].get("conversation_id")
            raw_init_cwd = (
                first_init.get("cwd") if isinstance(first_init, dict) else None
            )
            if isinstance(raw_init_cwd, str) and Path(raw_init_cwd).is_absolute():
                init_cwd = str(Path(raw_init_cwd).resolve())
                if Path(init_cwd) != expected_cwd.resolve():
                    cwd_mismatch = True
            init_permission_mode = (
                first_init.get("permission_mode")
                if isinstance(first_init, dict)
                else None
            )
            permission_mode_mismatch = (
                expected_permission_mode is not None
                and init_permission_mode != expected_permission_mode
            )
        init_positions = [
            index for index, event in enumerate(events) if event.get("event") == "init"
        ]
        result_positions = [
            index for index, event in enumerate(events) if event.get("event") == "result"
        ]
        invalid_shape = (
            not events
            or init_positions != [0]
            or len(result_positions) != 1
            or result_positions[0] != len(events) - 1
        )
        if invalid_shape:
            if (
                cwd_mismatch
                and len(events) == 1
                and init_positions == [0]
                and not result_positions
                and isinstance(init_id, str)
                and init_id
            ):
                status = "cwd-mismatch"
            elif (
                permission_mode_mismatch
                and len(events) == 1
                and init_positions == [0]
                and not result_positions
                and isinstance(init_id, str)
                and init_id
            ):
                status = "permission-mode-mismatch"
            else:
                parse_error = "stream must start with one init and end with one result"
        else:
            init_event = events[0]
            result_event = events[-1]
            init = init_event.get("init")
            result = result_event.get("result")
            init_id = init_event.get("conversation_id")
            if (
                not isinstance(init, dict)
                or not isinstance(init_id, str)
                or not init_id
                or not isinstance(init.get("cwd"), str)
                or not Path(init["cwd"]).is_absolute()
                or not isinstance(result, dict)
                or not isinstance(result.get("conversation_id"), str)
                or not isinstance(result.get("status"), str)
                or not isinstance(result.get("response"), str)
            ):
                parse_error = "init or result event has an invalid shape"
            else:
                init_cwd = str(Path(init["cwd"]).resolve())
                init_permission_mode = init.get("permission_mode")
                result_id = result["conversation_id"]
                result_status = result["status"]
                response = result["response"]
                tool_calls, tool_error = stream_tool_calls(events)
                denied_tool_steps = structured_tool_permission_denial_steps(events)
                if tool_error is not None:
                    parse_error = tool_error
                    status = "invalid-stream"
                elif init_cwd is None:
                    parse_error = "init cwd is unavailable for command binding"
                    status = "invalid-stream"
                else:
                    bind_implicit_run_command_cwd(tool_calls, init_cwd)
                if tool_error is None and parse_error is None and cwd_mismatch:
                    status = "cwd-mismatch"
                elif tool_error is None and parse_error is None and permission_mode_mismatch:
                    status = "permission-mode-mismatch"
                elif tool_error is None and parse_error is None and (
                    result_id != init_id
                    or (
                        requested_conversation_id is not None
                        and (
                            init_id != requested_conversation_id
                            or result_id != requested_conversation_id
                        )
                    )
                    or any(
                        call.get("conversation_id") not in {None, init_id}
                        for call in tool_calls
                    )
                ):
                    status = "conversation-mismatch"
                elif tool_error is None and parse_error is None and (
                    permission_denial_text(
                        stderr_path.read_text(errors="replace")
                        if stderr_path.is_file()
                        else ""
                    )
                    or any(
                        structured_tool_permission_denial(event) for event in events
                    )
                ):
                    status = "soft-denied"
                elif tool_error is None and parse_error is None and (result_status != "SUCCESS" or exit_code != 0):
                    status = "result-error"
                elif tool_error is None and parse_error is None and extract_exec_report(response) is None:
                    status = "invalid-report"
                elif tool_error is None and parse_error is None:
                    status = "reported"

    trusted_conversation_id = init_id
    if status in {
        "invalid-stream",
        "cwd-mismatch",
        "permission-mode-mismatch",
        "conversation-mismatch",
    }:
        trusted_conversation_id = None
    return {
        "delivery_status": status,
        "conversation_id": trusted_conversation_id,
        "init_conversation_id": init_id,
        "result_conversation_id": result_id,
        "init_cwd": init_cwd,
        "init_permission_mode": init_permission_mode,
        "result_status": result_status,
        "tool_calls": tool_calls,
        "denied_tool_steps": sorted(denied_tool_steps),
        "response": response,
        "exec_report": extract_exec_report(response),
        "parse_error": parse_error,
    }


def assert_assignment_guard_audit(
    profile: dict,
    task_key: str,
    inspected: dict,
    guard_paths: dict[str, Path],
) -> list[dict]:
    """Verify the hook policy and every streamed tool request from raw files."""
    try:
        policy, policy_digest = assignment_guard.load_policy(
            guard_paths["guard_policy"]
        )
    except assignment_guard.PolicyError as error:
        raise SystemExit(f"VOID: invalid assignment guard policy: {error}") from error
    if policy != assignment_guard_policy(profile, task_key):
        raise SystemExit("VOID: assignment guard policy differs from the profile")
    if sha256(guard_paths["guard_program"]) != assignment_guard_contract()[
        "program_sha256"
    ]:
        raise SystemExit("VOID: assignment guard program differs from frozen source")
    try:
        hooks = json.loads(guard_paths["guard_hooks"].read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"VOID: invalid assignment hooks artifact: {error}") from error
    if hooks != assignment_hooks_config():
        raise SystemExit("VOID: assignment hooks artifact changed")

    events: list[dict] = []
    try:
        lines = guard_paths["guard_audit"].read_text(errors="strict").splitlines()
    except (OSError, UnicodeError) as error:
        raise SystemExit(f"VOID: cannot read assignment guard audit: {error}") from error
    required_keys = {
        "schema",
        "at",
        "assignment_digest",
        "task_key",
        "policy_sha256",
        "conversation_id",
        "step_index",
        "workspace_paths",
        "model_name",
        "tool_name",
        "tool_args_digest",
        "command",
        "cwd",
        "target_path",
        "decision",
        "reason",
    }
    for number, line in enumerate(lines, start=1):
        if not line.strip():
            raise SystemExit(
                f"VOID: blank assignment guard audit record at line {number}"
            )
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise SystemExit(
                f"VOID: invalid assignment guard audit at line {number}: {error}"
            ) from error
        if not isinstance(value, dict) or set(value) != required_keys:
            raise SystemExit(
                f"VOID: malformed assignment guard audit record at line {number}"
            )
        if (
            value.get("schema") != assignment_guard.AUDIT_SCHEMA
            or value.get("assignment_digest") != profile["assignment_digest"]
            or value.get("task_key") != task_key
            or value.get("policy_sha256") != policy_digest
            or value.get("model_name") != profile["model"]
            or not isinstance(value.get("at"), str)
            or not value["at"]
            or not isinstance(value.get("reason"), str)
            or not value["reason"]
            or not isinstance(value.get("tool_args_digest"), str)
            or not re.fullmatch(r"[0-9a-f]{64}", value["tool_args_digest"])
        ):
            raise SystemExit(
                f"VOID: assignment guard audit identity mismatch at line {number}"
            )
        workspace_paths = value.get("workspace_paths")
        if (
            not isinstance(workspace_paths, list)
            or len(workspace_paths) != 1
            or not isinstance(workspace_paths[0], str)
            or not Path(workspace_paths[0]).is_absolute()
            or str(Path(workspace_paths[0]).resolve()) not in policy["workspace_roots"]
        ):
            raise SystemExit(
                f"VOID: assignment guard workspace identity mismatch at line {number}"
            )
        events.append(value)

    tool_calls = inspected.get("tool_calls")
    if not isinstance(tool_calls, list):
        raise SystemExit("VOID: raw stream has no canonical tool-call inventory")
    denied_steps = inspected.get("denied_tool_steps")
    if not isinstance(denied_steps, list) or any(
        not isinstance(step, int) or step < 0 for step in denied_steps
    ):
        raise SystemExit("VOID: raw stream has an invalid denied-tool inventory")
    stream_steps = {call["step_index"] for call in tool_calls}
    by_step: dict[int, list[dict]] = {}
    for event in events:
        step = event.get("step_index")
        if not isinstance(step, int) or step < 0 or step not in stream_steps:
            raise SystemExit("VOID: assignment guard audit step identity is invalid")
        by_step.setdefault(step, []).append(event)

    root = agy_launch_cwd(profile)
    conversation_id = inspected.get("init_conversation_id")
    for call in tool_calls:
        step = call["step_index"]
        audited_events = by_step.get(step, [])
        if not audited_events:
            if (
                step in denied_steps
                and call.get("command") in policy["expected_denied_commands"]
            ):
                expected_output, _expected_event = assignment_guard.decide(
                    policy,
                    policy_digest,
                    {
                        "conversationId": conversation_id,
                        "stepIdx": step,
                        "workspacePaths": [str(root)],
                        "modelName": profile["model"],
                        "toolCall": {
                            "name": call["tool_name"],
                            "args": call["tool_args"],
                        },
                    },
                )
                if expected_output["decision"] == "deny":
                    continue
            raise SystemExit(
                f"VOID: assignment guard audit is missing streamed tool step {step}"
            )
        target = call.get("target_path")
        expected_target = None
        if isinstance(target, str) and target:
            candidate = Path(target)
            if not candidate.is_absolute():
                candidate = root / candidate
            expected_target = str(candidate.resolve())
        for event in audited_events:
            expected_output, expected_event = assignment_guard.decide(
                policy,
                policy_digest,
                {
                    "conversationId": conversation_id,
                    "stepIdx": step,
                    "workspacePaths": event["workspace_paths"],
                    "modelName": profile["model"],
                    "toolCall": {
                        "name": call["tool_name"],
                        "args": call["tool_args"],
                    },
                },
            )
            if (
                event.get("conversation_id") != conversation_id
                or event.get("tool_name") != call.get("tool_name")
                or (
                    call.get("tool_name") != "run_command"
                    and event.get("tool_args_digest") != call.get("tool_args_digest")
                )
                or event.get("command") != call.get("command")
                or event.get("cwd") != call.get("cwd")
                or event.get("target_path") != expected_target
            ):
                raise SystemExit(
                    f"VOID: assignment guard audit differs from streamed tool step {step}"
                )
            if (
                event.get("decision") != expected_output["decision"]
                or event.get("reason") != expected_event["reason"]
            ):
                raise SystemExit(
                    "VOID: assignment guard decision differs from the frozen contract "
                    f"at streamed tool step {step}"
                )
            if (
                expected_output["decision"] != "allow"
                and call.get("command") not in policy["expected_denied_commands"]
            ):
                raise SystemExit(
                    "VOID: AGY requested a tool outside the frozen assignment at "
                    f"streamed step {step}"
                )
    return events


def guarded_delivery_status(inspected: dict, guard_events: list[dict]) -> str:
    """Treat an audited hook denial as a soft delivery failure."""
    status = inspected["delivery_status"]
    protocol_failures = {
        "invalid-stream",
        "cwd-mismatch",
        "permission-mode-mismatch",
        "conversation-mismatch",
    }
    if status not in protocol_failures and any(
        event.get("decision") == "deny" for event in guard_events
    ):
        return "soft-denied"
    return status


def write_run_evidence(
    *,
    profile: dict,
    task_key: str,
    suffix: str,
    conversation_id: str | None,
    prompt_path: Path,
    round_contract_path: Path,
    agy_log_path: Path,
    stream_path: Path,
    stderr_path: Path,
    response_path: Path,
    guard_artifacts: dict[str, Path],
    normalized_report_path: Path | None,
    snapshot_id: str,
    exit_code: int,
    delivery_status: str,
    init_permission_mode: str | None,
) -> Path:
    ordinal = attempt_ordinal_from_suffix(suffix)
    expected_names = {
        "prompt": f"{task_key}{suffix}.prompt.md",
        "round_contract": f"{task_key}{suffix}.contract.json",
        "agy_log": f"{task_key}{suffix}.agy.log",
        "stream": f"{task_key}{suffix}.stream.jsonl",
        "stderr": f"{task_key}{suffix}.stderr.log",
        "response": f"{task_key}{suffix}.response.md",
        "guard_program": f"{task_key}{suffix}.guard.py",
        "guard_policy": f"{task_key}{suffix}.guard-policy.json",
        "guard_audit": f"{task_key}{suffix}.guard-audit.jsonl",
        "guard_hooks": f"{task_key}{suffix}.hooks.json",
        "normalized_report": f"{task_key}{suffix}.report.md",
    }
    files = {
        "prompt": prompt_path,
        "round_contract": round_contract_path,
        "agy_log": agy_log_path,
        "stream": stream_path,
        "stderr": stderr_path,
        "response": response_path,
        **guard_artifacts,
    }
    if normalized_report_path is not None:
        files["normalized_report"] = normalized_report_path
    missing = [label for label, path in files.items() if not path.is_file()]
    if missing:
        raise SystemExit(
            "dispatch delivery evidence is incomplete: " + ", ".join(missing)
        )
    mismatched_names = [
        label
        for label, path in files.items()
        if path.name != expected_names[label]
    ]
    if mismatched_names:
        raise SystemExit(
            "dispatch delivery evidence has noncanonical artifact names: "
            + ", ".join(mismatched_names)
        )
    payload = {
        "version": RUN_EVIDENCE_VERSION,
        "audit_contract_version": AUDIT_CONTRACT_VERSION,
        "task_key": task_key,
        "suffix": suffix,
        "attempt_ordinal": ordinal,
        "conversation_id": conversation_id,
        "snapshot_id": snapshot_id,
        "assignment_digest": profile["assignment_digest"],
        "backend_resolution": profile["backend_resolution"],
        "exit_code": exit_code,
        "delivery_status": delivery_status,
        "dispatch_role": profile["dispatch_role"],
        "model": profile["model"],
        "effort": profile["effort"],
        "launch_cwd": str(agy_launch_cwd(profile)),
        "init_permission_mode": init_permission_mode,
        "files": {
            label: {"name": path.name, "sha256": sha256(path)}
            for label, path in files.items()
        },
    }
    path = stream_path.parent / f"{task_key}{suffix}.evidence.json"
    path.write_text(json.dumps(payload, indent=2) + "\n")
    return path


def latest_run_evidence(profile: dict, task_key: str) -> tuple[Path, dict] | None:
    candidates = canonical_attempt_files(profile, task_key, ".evidence.json")
    if not candidates:
        return None
    path = candidates[max(candidates)]
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise SystemExit(f"VOID: malformed run evidence: {path}")
    return path, value


def assert_run_evidence(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> dict:
    ordinals = assert_complete_attempt_lineage(profile, task_key)
    latest = latest_run_evidence(profile, task_key)
    if latest is None:
        raise SystemExit("VOID: no AGY run-attempt evidence exists")
    evidence_path, evidence = latest
    if (
        evidence.get("version") != RUN_EVIDENCE_VERSION
        or evidence.get("audit_contract_version") != AUDIT_CONTRACT_VERSION
        or evidence.get("task_key") != task_key
    ):
        raise SystemExit(f"VOID: run evidence identity mismatch: {evidence_path}")
    latest_ordinal = ordinals[-1]
    expected_suffix = canonical_attempt_suffix(latest_ordinal)
    if evidence.get("suffix") != expected_suffix:
        raise SystemExit(f"VOID: run evidence attempt mismatch: {evidence_path}")
    if evidence.get("attempt_ordinal") != latest_ordinal:
        raise SystemExit(f"VOID: run evidence ordinal mismatch: {evidence_path}")
    if evidence.get("snapshot_id") != snapshot_data.get("snapshot_id"):
        raise SystemExit("VOID: run evidence belongs to a different snapshot")
    if evidence.get("assignment_digest") != profile["assignment_digest"]:
        raise SystemExit("VOID: run evidence assignment digest mismatch")
    if evidence.get("backend_resolution") != profile["backend_resolution"]:
        raise SystemExit("VOID: run evidence backend resolution mismatch")
    if evidence.get("model") != profile["model"]:
        raise SystemExit("VOID: run evidence model mismatch")
    if evidence.get("dispatch_role") != profile["dispatch_role"]:
        raise SystemExit("VOID: run evidence dispatch role mismatch")
    if evidence.get("effort") != profile["effort"]:
        raise SystemExit("VOID: run evidence effort mismatch")
    if evidence.get("launch_cwd") != str(agy_launch_cwd(profile)):
        raise SystemExit("VOID: run evidence launch cwd mismatch")
    if "init_permission_mode" not in evidence or not isinstance(
        evidence.get("init_permission_mode"),
        (str, type(None)),
    ):
        raise SystemExit("VOID: run evidence permission mode is malformed")
    exit_code = evidence.get("exit_code")
    delivery_status = evidence.get("delivery_status")
    if not isinstance(exit_code, int) or delivery_status not in {
        "reported",
        "result-error",
        "soft-denied",
        "cwd-mismatch",
        "permission-mode-mismatch",
        "conversation-mismatch",
        "invalid-stream",
        "invalid-report",
    }:
        raise SystemExit("VOID: run evidence delivery classification is invalid")

    log_dir = evidence_path.parent.resolve()
    files = evidence.get("files")
    required = {
        "prompt",
        "round_contract",
        "agy_log",
        "stream",
        "stderr",
        "response",
        "guard_program",
        "guard_policy",
        "guard_audit",
        "guard_hooks",
    }
    if delivery_status == "reported":
        required.add("normalized_report")
    if not isinstance(files, dict) or set(files) != required:
        raise SystemExit("VOID: run evidence file inventory is incomplete")
    resolved: dict[str, Path] = {}
    expected_names = {
        "prompt": f"{task_key}{expected_suffix}.prompt.md",
        "round_contract": f"{task_key}{expected_suffix}.contract.json",
        "agy_log": f"{task_key}{expected_suffix}.agy.log",
        "stream": f"{task_key}{expected_suffix}.stream.jsonl",
        "stderr": f"{task_key}{expected_suffix}.stderr.log",
        "response": f"{task_key}{expected_suffix}.response.md",
        "guard_program": f"{task_key}{expected_suffix}.guard.py",
        "guard_policy": f"{task_key}{expected_suffix}.guard-policy.json",
        "guard_audit": f"{task_key}{expected_suffix}.guard-audit.jsonl",
        "guard_hooks": f"{task_key}{expected_suffix}.hooks.json",
        "normalized_report": f"{task_key}{expected_suffix}.report.md",
    }
    for label in sorted(required):
        entry = files[label]
        if not isinstance(entry, dict):
            raise SystemExit(f"VOID: malformed run evidence entry: {label}")
        name = entry.get("name")
        expected = entry.get("sha256")
        if not isinstance(name, str) or Path(name).name != name:
            raise SystemExit(f"VOID: unsafe run evidence path: {label}")
        if name != expected_names[label]:
            raise SystemExit(
                f"VOID: run evidence artifact name mismatch: {label}"
            )
        path = (log_dir / name).resolve()
        if path.parent != log_dir or not path.is_file():
            raise SystemExit(f"VOID: run evidence file is missing: {label}")
        if not isinstance(expected, str) or sha256(path) != expected:
            raise SystemExit(f"VOID: run evidence digest mismatch: {label}")
        resolved[label] = path

    conversation_id = evidence.get("conversation_id")
    inspected = inspect_headless_stream(
        resolved["stream"],
        resolved["stderr"],
        expected_cwd=agy_launch_cwd(profile),
        requested_conversation_id=(
            conversation_id if latest_ordinal > 0 and isinstance(conversation_id, str) else None
        ),
        expected_permission_mode=tool_permission_mode(),
        exit_code=exit_code,
    )
    if inspected["conversation_id"] != conversation_id:
        raise SystemExit("VOID: stream conversation does not match run evidence")
    if inspected["init_permission_mode"] != evidence.get("init_permission_mode"):
        raise SystemExit("VOID: raw stream permission mode differs from run evidence")
    guard_events = assert_assignment_guard_audit(
        profile,
        task_key,
        inspected,
        {
            label: resolved[label]
            for label in (
                "guard_program",
                "guard_policy",
                "guard_audit",
                "guard_hooks",
            )
        },
    )
    if guarded_delivery_status(inspected, guard_events) != delivery_status:
        raise SystemExit("VOID: run evidence delivery classification mismatch")
    current_conversation_id = conversation_id_for_task(profile, task_key)
    if (
        not isinstance(conversation_id, str)
        or not conversation_id
        or conversation_id != current_conversation_id
    ):
        raise SystemExit("VOID: run evidence has no auditable conversation lineage")
    response = resolved["response"].read_text(errors="replace")
    if response != inspected["response"]:
        raise SystemExit("VOID: final response differs from raw stream result")
    if delivery_status == "reported":
        normalized = resolved["normalized_report"].read_text(errors="replace")
        if inspected["exec_report"] != normalized:
            raise SystemExit("VOID: normalized EXEC REPORT identity mismatch")
    latest_contract = latest_round_contract(profile, task_key)
    if latest_contract is None or latest_contract[0] != resolved["round_contract"]:
        raise SystemExit("VOID: run evidence does not bind the latest round contract")
    return evidence


def latest_run_transport(
    profile: dict,
    task_key: str,
    evidence: dict,
) -> tuple[dict, list[dict]]:
    """Reparse the latest verified evidence files for isolation controls."""
    files = evidence.get("files")
    if not isinstance(files, dict):
        raise SystemExit("VOID: run evidence cannot locate transport artifacts")
    required = (
        "stream",
        "stderr",
        "guard_program",
        "guard_policy",
        "guard_audit",
        "guard_hooks",
    )
    names: dict[str, str] = {}
    for label in required:
        entry = files.get(label)
        name = entry.get("name") if isinstance(entry, dict) else None
        if not isinstance(name, str) or Path(name).name != name:
            raise SystemExit("VOID: run evidence transport artifact is malformed")
        names[label] = name
    log_dir = Path(profile["state_dir"]) / "runs"
    inspected = inspect_headless_stream(
        log_dir / names["stream"],
        log_dir / names["stderr"],
        expected_cwd=agy_launch_cwd(profile),
        requested_conversation_id=(
            evidence.get("conversation_id")
            if isinstance(evidence.get("conversation_id"), str)
            and evidence.get("attempt_ordinal", 0) > 0
            else None
        ),
        expected_permission_mode=tool_permission_mode(),
        exit_code=evidence.get("exit_code", -1),
    )
    guard_events = assert_assignment_guard_audit(
        profile,
        task_key,
        inspected,
        {
            "guard_program": log_dir / names["guard_program"],
            "guard_policy": log_dir / names["guard_policy"],
            "guard_audit": log_dir / names["guard_audit"],
            "guard_hooks": log_dir / names["guard_hooks"],
        },
    )
    if guarded_delivery_status(inspected, guard_events) != evidence.get(
        "delivery_status"
    ):
        raise SystemExit("VOID: latest transport delivery classification drifted")
    return inspected, guard_events


def verified_marker_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "runs" / f"{task_key}.verified.json"


def write_verified_marker(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
    evidence_path: Path,
    evidence: dict,
    *,
    conversation_step_ceiling: int | None = None,
    audited_steps_digest: str | None = None,
) -> Path:
    """Record that the controller audited the latest attempt through its last step."""
    conversation_id = evidence.get("conversation_id")
    if not isinstance(conversation_id, str) or not conversation_id:
        raise SystemExit("VOID: cannot verify a predecessor without conversation lineage")
    latest = latest_run_evidence(profile, task_key)
    if latest is None or latest[0] != evidence_path.resolve():
        raise SystemExit("VOID: verified marker does not bind the latest run attempt")
    current_step_max = conversation_step_max(conversation_id)
    if (
        conversation_step_ceiling is not None
        and current_step_max != conversation_step_ceiling
    ):
        raise SystemExit(
            "VOID: new AGY steps appeared while verification was in progress"
        )
    current_steps_digest = conversation_steps_digest(
        conversation_id,
        through_step=current_step_max,
    )
    if (
        audited_steps_digest is not None
        and current_steps_digest != audited_steps_digest
    ):
        raise SystemExit(
            "VOID: AGY conversation rows changed while verification was in progress"
        )
    attempt_ordinals = assert_complete_attempt_lineage(profile, task_key)
    payload = {
        "version": VERIFIED_MARKER_VERSION,
        "audit_contract_version": AUDIT_CONTRACT_VERSION,
        "task_key": task_key,
        "snapshot_id": snapshot_data.get("snapshot_id"),
        "assignment_digest": profile["assignment_digest"],
        "backend_resolution": profile["backend_resolution"],
        "evidence_name": evidence_path.name,
        "evidence_sha256": sha256(evidence_path),
        "conversation_id": conversation_id,
        "conversation_step_max": current_step_max,
        "conversation_steps_digest": current_steps_digest,
        "attempt_ordinal_max": attempt_ordinals[-1],
        "attempt_lineage_digest": attempt_lineage_digest(profile, task_key),
        "delivery_status": evidence.get("delivery_status"),
        "dispatch_role": evidence.get("dispatch_role"),
        "model": evidence.get("model"),
        "effort": evidence.get("effort"),
        "init_permission_mode": evidence.get("init_permission_mode"),
        "verified_at": datetime.now(timezone.utc).isoformat(),
    }
    path = verified_marker_path(profile, task_key)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n")
    return path


def assert_verified_predecessor(
    profile: dict,
    task_key: str,
    conversation_id: str,
) -> dict:
    """Refuse resume/rebaseline when the latest conversation events are unaudited."""
    path = verified_marker_path(profile, task_key)
    if not path.is_file():
        raise SystemExit(
            "refusing conversation reuse: verify the prior AGY attempt first"
        )
    try:
        marker = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"VOID: malformed verified-predecessor marker: {error}")
    latest = latest_run_evidence(profile, task_key)
    if latest is None:
        raise SystemExit("VOID: verified predecessor has no run evidence")
    evidence_path, evidence = latest
    evidence_snapshot_id = evidence.get("snapshot_id")
    if not isinstance(evidence_snapshot_id, str):
        raise SystemExit("VOID: verified predecessor lacks a snapshot identity")
    historical_snapshot = load_immutable_snapshot(
        profile,
        task_key,
        evidence_snapshot_id,
    )
    assert_run_evidence(profile, task_key, historical_snapshot)
    attempt_ordinals = assert_complete_attempt_lineage(profile, task_key)
    if (
        marker.get("version") != VERIFIED_MARKER_VERSION
        or marker.get("audit_contract_version") != AUDIT_CONTRACT_VERSION
        or marker.get("task_key") != task_key
        or marker.get("conversation_id") != conversation_id
        or marker.get("snapshot_id") != evidence.get("snapshot_id")
        or marker.get("assignment_digest") != evidence.get("assignment_digest")
        or marker.get("backend_resolution") != evidence.get("backend_resolution")
        or marker.get("evidence_name") != evidence_path.name
        or marker.get("evidence_sha256") != sha256(evidence_path)
        or marker.get("delivery_status") != evidence.get("delivery_status")
        or marker.get("dispatch_role") != evidence.get("dispatch_role")
        or marker.get("model") != evidence.get("model")
        or marker.get("effort") != evidence.get("effort")
        or marker.get("init_permission_mode")
        != evidence.get("init_permission_mode")
        or marker.get("attempt_ordinal_max") != attempt_ordinals[-1]
        or marker.get("attempt_lineage_digest")
        != attempt_lineage_digest(profile, task_key)
    ):
        raise SystemExit("VOID: verified-predecessor lineage changed")
    expected_step = marker.get("conversation_step_max")
    if not isinstance(expected_step, int):
        raise SystemExit("VOID: verified predecessor lacks a conversation step ceiling")
    if conversation_step_max(conversation_id) != expected_step:
        raise SystemExit(
            "refusing conversation reuse: new AGY steps appeared after verification"
        )
    expected_digest = marker.get("conversation_steps_digest")
    if (
        not isinstance(expected_digest, str)
        or conversation_steps_digest(
            conversation_id,
            through_step=expected_step,
        )
        != expected_digest
    ):
        raise SystemExit("VOID: verified conversation rows changed after verification")
    return marker


def assert_initial_task_worktree_unchanged(
    profile: dict,
    snapshot_data: dict,
) -> None:
    baseline = snapshot_data.get("task_worktree_baseline")
    if not isinstance(baseline, dict):
        raise SystemExit(
            "VOID: snapshot lacks the task-worktree baseline; create a fresh snapshot"
        )
    current = repository_worktree_baseline(
        Path(profile["root"]).resolve(),
        protect_raw_index=False,
    )
    if stable_worktree_baseline(current) != stable_worktree_baseline(baseline):
        raise SystemExit(
            "VOID: task worktree changed between snapshot and initial dispatch"
        )


def validate_conversation_action(
    profile: dict,
    task_key: str,
    *,
    resume: bool,
) -> str | None:
    validate_task_key(profile, task_key)
    policy = task_session_policy(profile)
    conversation_id = conversation_id_for_task(profile, task_key)
    if resume:
        if policy == "one-shot":
            raise SystemExit(
                f"cannot resume one-shot run {task_key}; create a new run id"
            )
        if not conversation_id:
            raise SystemExit(
                f"cannot resume ticket #{task_key}: no conversation id in state"
            )
    elif conversation_id:
        remediation = (
            f"use resume for ticket #{task_key}"
            if policy == "ticketed"
            else "create a new one-shot run id"
        )
        raise SystemExit(
            f"task {task_key} already has conversation {conversation_id}; "
            + remediation
        )
    else:
        log_dir = Path(profile["state_dir"]) / "runs"
        prior_attempt = any(
            (log_dir / f"{task_key}{suffix}").exists()
            for suffix in (
                ".prompt.md",
                ".contract.json",
                ".agy.log",
                ".stream.jsonl",
                ".stderr.log",
                ".response.md",
                ".evidence.json",
            )
        )
        if prior_attempt:
            remediation = (
                "create a new one-shot run id"
                if policy == "one-shot"
                else "treat the missing conversation lineage as VOID"
            )
            raise SystemExit(
                f"task {task_key} already has an initial run attempt without "
                f"auditable conversation lineage; {remediation}"
            )
    return conversation_id


def agy_command(profile: dict, conversation_id: str | None) -> list[str]:
    """Build the bounded AGY invocation from the frozen profile."""
    if profile.get("sandbox") is not True:
        raise SystemExit("sandbox must remain fixed to true")
    command = [
        "agy",
        "--project",
        agy_project_id(profile),
        "--sandbox",
        "--output-format",
        "stream-json",
    ]
    if conversation_id:
        command.extend(["--conversation", conversation_id])
    return command


def agy_launch_cwd(profile: dict) -> Path:
    """Return the snapshot-bound task worktree used as AGY's process cwd."""
    if profile.get("launch_cwd") != REQUIRED_LAUNCH_CWD:
        raise SystemExit(f"launch_cwd must be {REQUIRED_LAUNCH_CWD}")
    return Path(profile["root"]).resolve()


def assignment_guard_policy(profile: dict, task_key: str) -> dict:
    """Create the exact per-attempt tool policy consumed by the AGY hook."""
    validate_task_key(profile, task_key)
    return {
        "schema": assignment_guard.POLICY_SCHEMA,
        "assignment_digest": profile["assignment_digest"],
        "task_key": task_key,
        "worktree_root": str(agy_launch_cwd(profile)),
        "workspace_roots": [
            str(agy_launch_cwd(profile)),
            str(agy_project_root(profile)),
        ],
        "executor_role": profile["dispatch_role"],
        "model": profile["model"],
        "task_commands": {
            "allow": list(profile["task_commands"]["allow"]),
            "deny": list(profile["task_commands"]["deny"]),
        },
        "allowed_repo_writes": list(profile["allowed_repo_writes"]),
        "expected_denied_commands": list(
            profile["task_contract"].get("expected_soft_denied_commands", [])
        ),
    }


def assignment_hooks_config() -> dict[str, object]:
    """Record the immutable global-hook loader required for this run."""
    return {GLOBAL_ASSIGNMENT_GUARD_NAME: global_assignment_guard_hook_config()}


def materialize_assignment_guard(
    profile: dict,
    task_key: str,
    suffix: str,
    log_dir: Path,
) -> dict[str, Path]:
    """Write immutable run-local hook inputs before AGY starts."""
    log_dir = log_dir.resolve()
    paths = {
        "guard_program": log_dir / f"{task_key}{suffix}.guard.py",
        "guard_policy": log_dir / f"{task_key}{suffix}.guard-policy.json",
        "guard_audit": log_dir / f"{task_key}{suffix}.guard-audit.jsonl",
        "guard_hooks": log_dir / f"{task_key}{suffix}.hooks.json",
    }
    for path in paths.values():
        if path.exists():
            raise SystemExit(f"refusing to overwrite assignment guard artifact: {path}")

    guard_source = Path(assignment_guard.__file__).resolve()
    if sha256(guard_source) != assignment_guard_contract()["program_sha256"]:
        raise SystemExit("assignment guard changed while materializing the run")
    shutil.copyfile(guard_source, paths["guard_program"])
    paths["guard_program"].chmod(0o700)
    paths["guard_policy"].write_text(
        json.dumps(assignment_guard_policy(profile, task_key), indent=2) + "\n"
    )
    paths["guard_policy"].chmod(0o600)
    try:
        assignment_guard.load_policy(paths["guard_policy"])
    except assignment_guard.PolicyError as error:
        raise SystemExit(f"cannot materialize assignment guard policy: {error}") from error
    paths["guard_audit"].touch(exist_ok=False)
    paths["guard_audit"].chmod(0o600)
    hooks = assignment_hooks_config()
    paths["guard_hooks"].write_text(json.dumps(hooks, indent=2) + "\n")
    paths["guard_hooks"].chmod(0o600)
    return paths


@contextmanager
def installed_active_guard_registration(
    profile: dict,
    task_key: str,
    suffix: str,
    guard_paths: dict[str, Path],
) -> object:
    """Publish one private worktree-bound registration for the fixed loader."""
    log_dir = guard_paths["guard_policy"].parent.resolve()
    registration = log_dir / f"{task_key}{suffix}.active-guard.json"
    if registration.exists() or registration.is_symlink():
        raise SystemExit(
            "refusing to overwrite active assignment guard registration: "
            f"{registration}"
        )
    payload = {
        "schema": ACTIVE_GUARD_REGISTRATION_SCHEMA,
        "worktree_root": str(agy_launch_cwd(profile)),
        "workspace_roots": assignment_guard_policy(profile, task_key)[
            "workspace_roots"
        ],
        "policy_path": str(guard_paths["guard_policy"].resolve()),
        "audit_path": str(guard_paths["guard_audit"].resolve()),
        "core_sha256": assignment_guard_contract()["program_sha256"],
    }
    expected = (json.dumps(payload, sort_keys=True) + "\n").encode()
    descriptor = os.open(
        registration,
        os.O_WRONLY | os.O_CREAT | os.O_EXCL,
        0o600,
    )
    try:
        os.write(descriptor, expected)
    finally:
        os.close(descriptor)
    drift = None
    try:
        yield registration
    finally:
        try:
            detail = registration.lstat()
            if (
                not stat.S_ISREG(detail.st_mode)
                or registration.is_symlink()
                or detail.st_uid != os.getuid()
                or detail.st_mode & 0o077
                or registration.read_bytes() != expected
            ):
                drift = "active assignment guard registration changed during AGY execution"
        except OSError as error:
            drift = f"cannot re-read active assignment guard registration: {error}"
        try:
            if registration.is_symlink() or registration.is_file():
                registration.unlink()
            elif registration.exists():
                drift = drift or "active assignment guard registration became a directory"
        finally:
            if drift is not None:
                raise SystemExit(f"VOID: {drift}")


def terminate_headless_process(process: subprocess.Popen) -> None:
    """Stop a bad headless launch, then escalate only after a short grace."""
    if process.poll() is not None:
        return
    process.terminate()
    try:
        process.wait(timeout=TERMINATE_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        process.kill()
        process.wait()


def capture_headless_stream(
    command: list[str],
    *,
    cwd: Path,
    stream_path: Path,
    stderr_path: Path,
    expected_permission_mode: str | None = None,
    env: dict[str, str] | None = None,
) -> int:
    """Capture raw NDJSON and stop before work when init is absent or unsafe."""
    deadline = time.monotonic() + INIT_TIMEOUT_SECONDS
    with stream_path.open("wb") as stream, stderr_path.open("wb") as diagnostics:
        process = subprocess.Popen(
            command,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=diagnostics,
            env=env,
        )
        if process.stdout is None:
            terminate_headless_process(process)
            return process.returncode
        selector = selectors.DefaultSelector()
        selector.register(process.stdout, selectors.EVENT_READ)
        init_seen = False
        pending = b""
        try:
            while True:
                timeout = 0.25
                if not init_seen:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        terminate_headless_process(process)
                        break
                    timeout = min(timeout, remaining)
                ready = selector.select(timeout)
                if not ready:
                    if process.poll() is not None:
                        break
                    continue
                chunk = os.read(process.stdout.fileno(), 64 * 1024)
                if not chunk:
                    break
                stream.write(chunk)
                stream.flush()
                if init_seen:
                    continue
                pending += chunk
                if b"\n" not in pending:
                    continue
                line, _ = pending.split(b"\n", 1)
                try:
                    event = json.loads(line.decode("utf-8"))
                except (UnicodeDecodeError, json.JSONDecodeError):
                    terminate_headless_process(process)
                    break
                if not isinstance(event, dict) or event.get("event") != "init":
                    terminate_headless_process(process)
                    break
                init = event.get("init")
                init_id = event.get("conversation_id")
                raw_cwd = init.get("cwd") if isinstance(init, dict) else None
                permission_mode = (
                    init.get("permission_mode") if isinstance(init, dict) else None
                )
                if (
                    not isinstance(init_id, str)
                    or not init_id
                    or not isinstance(raw_cwd, str)
                    or not Path(raw_cwd).is_absolute()
                ):
                    terminate_headless_process(process)
                    break
                init_seen = True
                if (
                    Path(raw_cwd).resolve() != cwd.resolve()
                    or (
                        expected_permission_mode is not None
                        and permission_mode != expected_permission_mode
                    )
                ):
                    terminate_headless_process(process)
                    break
        finally:
            selector.close()
            process.stdout.close()
        if process.poll() is None:
            process.wait()
        return process.returncode


def run_agent(profile: dict, task_key: str, *, resume: bool) -> None:
    operation = "resume" if resume else "dispatch"
    with project_concurrency_lock(profile, task_key, operation):
        with task_operation_lock(profile, task_key, operation):
            run_agent_under_lock(profile, task_key, resume=resume)


def run_agent_under_lock(profile: dict, task_key: str, *, resume: bool) -> None:
    require_project_ready(profile)
    state = Path(profile["state_dir"])
    snapshot_data = load_snapshot(profile, task_key)
    assert_snapshot_identity(profile, task_key, snapshot_data)
    assert_snapshot_attempt_predecessor_unchanged(
        profile,
        task_key,
        snapshot_data,
    )
    assert_dispatch_contract_unchanged(
        profile,
        snapshot_data,
        allow_prompt_hash_change=(resume and task_session_policy(profile) == "ticketed"),
    )
    assert_injected_prompt_unchanged(profile)
    oracle = assert_oracle_unchanged(profile, task_key, snapshot_data)
    if not resume:
        assert_initial_task_worktree_unchanged(profile, snapshot_data)
    assert_permission_state_unchanged(profile, snapshot_data)
    assert_project_scope_unchanged(profile, snapshot_data)
    assert_sibling_worktrees_unchanged(profile, snapshot_data)
    assert_ignored_paths_unchanged(Path(profile["root"]), snapshot_data)
    assert_task_ignored_noncache_unchanged(profile, snapshot_data)
    assert_task_git_admin_unchanged(profile, snapshot_data)
    assert_git_common_objects_intact(profile, snapshot_data)
    assert_registered_worktree_indexes_unchanged(profile, snapshot_data)
    audited_commands = audit_task_commands(profile, task_key, snapshot_data)
    task_state = frozen_task_state(profile, task_key)
    assert_task_state_unchanged(profile, task_state, snapshot_data)
    conversation_id = validate_conversation_action(
        profile,
        task_key,
        resume=resume,
    )
    next_attempt_ordinal = 0
    if resume:
        assert conversation_id is not None
        assert_verified_predecessor(profile, task_key, conversation_id)
        ordinals = assert_complete_attempt_lineage(profile, task_key)
        next_attempt_ordinal = ordinals[-1] + 1
    log_dir = state / "runs"
    log_dir.mkdir(parents=True, exist_ok=True)
    suffix = canonical_attempt_suffix(next_attempt_ordinal)
    prompt = render_prompt(
        profile,
        task_key,
        oracle.read_text(),
        task_state_contract(profile, task_state),
        continuation=resume,
    )
    prompt_path = log_dir / f"{task_key}{suffix}.prompt.md"
    prompt_path.write_text(prompt)
    round_contract_path = log_dir / f"{task_key}{suffix}.contract.json"
    round_contract_path.write_text(
        json.dumps(dispatch_contract(profile), indent=2) + "\n"
    )
    conversation_path = log_dir / f"{task_key}.conversation"
    if resume:
        assert conversation_id is not None
        conversation_path.write_text(conversation_id + "\n")
    command = agy_command(profile, conversation_id if resume else None)
    agy_log_path = log_dir / f"{task_key}{suffix}.agy.log"
    agy_log_path.touch(exist_ok=True)
    command.extend(
        [
            "-p",
            prompt,
            "--model",
            profile["model"],
            "--effort",
            profile["effort"],
            "--print-timeout",
            profile.get("timeout", "30m"),
            "--log-file",
            str(agy_log_path),
        ]
    )
    stream_path = log_dir / f"{task_key}{suffix}.stream.jsonl"
    stderr_path = log_dir / f"{task_key}{suffix}.stderr.log"
    guard_artifacts = materialize_assignment_guard(
        profile,
        task_key,
        suffix,
        log_dir,
    )
    expected_permission_mode = tool_permission_mode()
    with installed_active_guard_registration(
        profile,
        task_key,
        suffix,
        guard_artifacts,
    ):
        exit_code = capture_headless_stream(
            command,
            cwd=agy_launch_cwd(profile),
            stream_path=stream_path,
            stderr_path=stderr_path,
            expected_permission_mode=expected_permission_mode,
        )
    inspected = inspect_headless_stream(
        stream_path,
        stderr_path,
        expected_cwd=agy_launch_cwd(profile),
        requested_conversation_id=(conversation_id if resume else None),
        expected_permission_mode=expected_permission_mode,
        exit_code=exit_code,
    )
    guard_events = assert_assignment_guard_audit(
        profile,
        task_key,
        inspected,
        guard_artifacts,
    )
    evidence_conversation_id = inspected["conversation_id"]
    response_path = log_dir / f"{task_key}{suffix}.response.md"
    response_path.write_text(inspected["response"])
    delivery_status = guarded_delivery_status(inspected, guard_events)
    normalized_path = None
    if delivery_status == "reported":
        normalized_report = inspected["exec_report"]
        assert normalized_report is not None
        normalized_path = log_dir / f"{task_key}{suffix}.report.md"
        normalized_path.write_text(normalized_report)
    if evidence_conversation_id:
        conversation_path.write_text(evidence_conversation_id + "\n")
    write_run_evidence(
        profile=profile,
        task_key=task_key,
        suffix=suffix,
        conversation_id=evidence_conversation_id,
        prompt_path=prompt_path,
        round_contract_path=round_contract_path,
        agy_log_path=agy_log_path,
        stream_path=stream_path,
        stderr_path=stderr_path,
        response_path=response_path,
        guard_artifacts=guard_artifacts,
        normalized_report_path=normalized_path,
        snapshot_id=snapshot_data["snapshot_id"],
        exit_code=exit_code,
        delivery_status=delivery_status,
        init_permission_mode=inspected["init_permission_mode"],
    )
    print(
        f"prompt sha256={sha256(prompt_path)}; "
        f"oracle sha256={sha256(oracle)}; exit={exit_code}"
    )
    if delivery_status != "reported":
        raise SystemExit(
            f"{'resume' if resume else 'dispatch'} failed for {task_key}: "
            f"delivery_status={delivery_status}; inspect the raw stream and stderr"
        )
    assert delivery_status == "reported"
    print(
        f"reported {task_key}; run status, then independently verify before "
        "acceptance"
    )


def dispatch(profile: dict, task_key: str) -> None:
    run_agent(profile, task_key, resume=False)


def resume(profile: dict, task_key: str) -> None:
    run_agent(profile, task_key, resume=True)


def recovery_path(state: Path, source_path: str) -> Path:
    identity = hashlib.sha256(source_path.encode()).hexdigest()[:16]
    return state / "snapshots" / "recovery" / f"{identity}.pre-round"


def park_original(
    state: Path,
    source_path: str,
    encoded: str | None,
) -> str:
    if encoded is None:
        return f"{source_path}: no pre-round content is available"
    parked = recovery_path(state, source_path)
    parked.parent.mkdir(parents=True, exist_ok=True)
    parked.write_bytes(base64.b64decode(encoded))
    return (
        f"{source_path}: pre-round bytes parked at {parked}; restore with "
        f"`cp {shlex.quote(str(parked))} {shlex.quote(source_path)}`"
    )


def verify(profile: dict, task_key: str) -> None:
    # A bounded writer keeps the Project lock until its evidence has been
    # checked.  This prevents a second task from changing the shared Project
    # before the first task's verification can prove what happened.
    with project_concurrency_lock(profile, task_key, "verify"):
        with task_operation_lock(profile, task_key, "verify"):
            verify_under_lock(profile, task_key)


def verify_under_lock(profile: dict, task_key: str) -> None:
    require_project_ready(profile)
    validate_task_key(profile, task_key)
    snapshot_data = load_snapshot(profile, task_key)
    assert_snapshot_identity(profile, task_key, snapshot_data)
    assert_snapshot_attempt_predecessor_unchanged(
        profile,
        task_key,
        snapshot_data,
    )
    assert_injected_prompt_unchanged(profile)
    assert_executed_round_contract(profile, task_key, snapshot_data)
    delivery_error = None
    run_evidence = None
    run_evidence_path = None
    inspected = None
    guard_events = None
    try:
        run_evidence = assert_run_evidence(profile, task_key, snapshot_data)
        latest_evidence = latest_run_evidence(profile, task_key)
        assert latest_evidence is not None
        run_evidence_path = latest_evidence[0]
        inspected, guard_events = latest_run_transport(
            profile,
            task_key,
            run_evidence,
        )
    except SystemExit as error:
        delivery_error = str(error)
    assert_worktree_scope_unchanged(profile, snapshot_data)
    assert_oracle_unchanged(profile, task_key, snapshot_data)
    assert_permission_state_unchanged(profile, snapshot_data)
    assert_project_scope_unchanged(profile, snapshot_data)
    assert_sibling_worktrees_unchanged(profile, snapshot_data)
    conversation_id = conversation_id_for_task(profile, task_key)
    conversation_step_ceiling = conversation_step_max(conversation_id)
    audited_steps_digest = (
        conversation_steps_digest(
            conversation_id,
            through_step=conversation_step_ceiling,
        )
        if conversation_id
        else json_digest([])
    )
    audited_commands = audit_task_commands(profile, task_key, snapshot_data)
    soft_denial_failures = expected_soft_denial_failures(
        profile,
        audited_commands,
        inspected=inspected,
        guard_events=guard_events,
    )
    assert_no_sandbox_file_access_denial(profile, task_key)

    state = Path(profile["state_dir"])
    root = Path(profile["root"])
    assert_ignored_paths_unchanged(root, snapshot_data)
    assert_task_ignored_noncache_unchanged(profile, snapshot_data)
    assert_task_git_admin_unchanged(profile, snapshot_data)
    assert_git_common_objects_intact(profile, snapshot_data)
    assert_registered_worktree_indexes_unchanged(profile, snapshot_data)
    result = subprocess.run(
        ["git", "-C", str(root), "status", "--porcelain=v1", "--untracked-files=all"],
        text=True,
        capture_output=True,
        check=True,
    )
    before = snapshot_data["git_status"]
    after = result.stdout
    before_manifest = snapshot_data["manifest"]
    after_manifest = manifest(root, profile["snapshot_paths"])
    all_manifest_paths = {*before_manifest, *after_manifest}
    changed_files = sorted(
        path
        for path in all_manifest_paths
        if before_manifest.get(path) != after_manifest.get(path)
    )
    if profile["mode"] == "measure-only" and (after != before or changed_files):
        raise SystemExit(
            "VOID: repository state changed during a measure-only dispatch: "
            f"{changed_files}"
        )
    if profile["mode"] == "bounded-write" and (after != before or changed_files):
        before_lines = set(before.splitlines())
        status_changes = [
            line[3:] for line in after.splitlines() if line not in before_lines
        ]
        allowed = set(profile["allowed_repo_writes"])
        outside = sorted(
            {
                path
                for path in [*status_changes, *changed_files]
                if path not in allowed
            }
        )
        if outside:
            protected = snapshot_data.get("protected_contents_base64", {})
            hints = [
                park_original(state, path, protected.get(path))
                for path in outside
            ]
            raise SystemExit(
                "VOID: unexpected repository changes: "
                + str(outside)
                + "\n"
                + "\n".join(f"  {hint}" for hint in hints)
            )

    budgets = profile.get("path_change_budgets", {})
    for relative, budget in budgets.items():
        before_text = snapshot_data.get("writable_contents", {}).get(relative)
        path = root / relative
        after_text = (
            path.read_text(errors="surrogateescape") if path.is_file() else None
        )
        before_lines = [] if before_text is None else before_text.splitlines()
        after_lines = [] if after_text is None else after_text.splitlines()
        added = sum(
            1
            for line in difflib.ndiff(before_lines, after_lines)
            if line.startswith("+ ")
        )
        deleted = sum(
            1
            for line in difflib.ndiff(before_lines, after_lines)
            if line.startswith("- ")
        )
        if isinstance(budget, int):
            delta = added + deleted
            if delta > budget:
                raise SystemExit(
                    f"VOID: diff budget exceeded for {relative}: "
                    f"{delta} changed lines > {budget}"
                )
            continue
        if added > budget["max_added"] or deleted > budget["max_deleted"]:
            raise SystemExit(
                f"VOID: diff budget exceeded for {relative}: "
                f"+{added}/-{deleted} exceeds "
                f"+{budget['max_added']}/-{budget['max_deleted']}"
            )

    protected = snapshot_data.get("protected_contents_base64", {})
    for path, expected in snapshot_data["protected_artifacts"].items():
        actual = sha256(Path(path))
        if actual != expected:
            hint = park_original(state, path, protected.get(path))
            raise SystemExit(
                f"VOID: protected artifact changed: {path}\n  {hint}"
            )
    oracle = state / "oracles" / f"{task_key}.md"
    if not oracle.exists():
        raise SystemExit("VOID: oracle disappeared")
    if delivery_error is not None:
        raise SystemExit(
            "VOID: repository isolation checks completed, but AGY delivery "
            f"evidence is invalid: {delivery_error}"
        )
    if soft_denial_failures:
        raise SystemExit(
            "ISOLATION_CONTROL_FAILED: repository isolation checks completed, "
            "but an expected soft-deny command was not denied exactly once: "
            + json.dumps(soft_denial_failures, sort_keys=True)
        )
    assert run_evidence is not None and run_evidence_path is not None
    write_verified_marker(
        profile,
        task_key,
        snapshot_data,
        run_evidence_path,
        run_evidence,
        conversation_step_ceiling=conversation_step_ceiling,
        audited_steps_digest=audited_steps_digest,
    )
    if run_evidence["delivery_status"] != "reported":
        print(
            "DELIVERY_FAILED_ISOLATION_VERIFIED: snapshot, Project, sibling, "
            "repository, protected artifacts, and task-local commands match; "
            f"delivery_status={run_evidence['delivery_status']}; "
            f"oracle sha256={sha256(oracle)}"
        )
        return
    print(
        "snapshot, protected artifacts, static Project policy, and "
        f"{len(audited_commands)} task-local shell command(s) match; "
        f"oracle sha256={sha256(oracle)}"
    )


def status(profile: dict) -> None:
    require_project_ready(profile)
    log_dir = Path(profile["state_dir"]) / "runs"
    for evidence_path in sorted(log_dir.glob("*.evidence.json")):
        attempt_name = evidence_path.name.removesuffix(".evidence.json")
        verdict = None
        try:
            evidence = json.loads(evidence_path.read_text())
        except json.JSONDecodeError:
            verdict = "INVALID EVIDENCE"
        else:
            if (
                evidence.get("version") != RUN_EVIDENCE_VERSION
                or evidence.get("audit_contract_version")
                != AUDIT_CONTRACT_VERSION
            ):
                verdict = "INVALID EVIDENCE"
                delivery_status = None
                task_key = None
            else:
                delivery_status = evidence.get("delivery_status")
                task_key = evidence.get("task_key")
                if (
                    evidence.get("dispatch_role") != profile["dispatch_role"]
                    or evidence.get("model") != profile["model"]
                    or evidence.get("effort") != profile["effort"]
                    or evidence.get("backend_resolution")
                    != profile["backend_resolution"]
                ):
                    verdict = "INVALID EVIDENCE"
            latest_is_current = False
            if verdict is None and isinstance(task_key, str):
                try:
                    latest = latest_run_evidence(profile, task_key)
                    latest_is_current = (
                        latest is not None and latest[0] == evidence_path.resolve()
                    )
                    if latest_is_current:
                        snapshot_data = load_snapshot(profile, task_key)
                        assert_run_evidence(profile, task_key, snapshot_data)
                except (OSError, json.JSONDecodeError, SystemExit):
                    verdict = "INVALID EVIDENCE"

            marker = None
            if verdict is None and isinstance(task_key, str):
                marker_path = verified_marker_path(profile, task_key)
                if marker_path.is_file():
                    try:
                        candidate = json.loads(marker_path.read_text())
                    except json.JSONDecodeError:
                        candidate = None
                    if (
                        isinstance(candidate, dict)
                        and candidate.get("version") == VERIFIED_MARKER_VERSION
                        and candidate.get("audit_contract_version")
                        == AUDIT_CONTRACT_VERSION
                        and candidate.get("evidence_name") == evidence_path.name
                        and candidate.get("evidence_sha256") == sha256(evidence_path)
                        and candidate.get("delivery_status") == delivery_status
                        and candidate.get("backend_resolution")
                        == evidence.get("backend_resolution")
                        and candidate.get("assignment_digest")
                        == evidence.get("assignment_digest")
                        and candidate.get("dispatch_role")
                        == evidence.get("dispatch_role")
                        and candidate.get("model") == evidence.get("model")
                        and candidate.get("effort") == evidence.get("effort")
                        and candidate.get("init_permission_mode")
                        == evidence.get("init_permission_mode")
                    ):
                        marker = candidate
            control_failures = []
            if (
                verdict is None
                and delivery_status == "reported"
                and isinstance(task_key, str)
                and latest_is_current
            ):
                try:
                    audited_commands = audit_task_commands(
                        profile,
                        task_key,
                        snapshot_data,
                    )
                    control_failures = expected_soft_denial_failures(
                        profile,
                        audited_commands,
                    )
                except SystemExit:
                    verdict = "INVALID EVIDENCE"
            if verdict is None and control_failures:
                verdict = "ISOLATION CONTROL FAILED"
            elif verdict is None and delivery_status == "reported":
                verdict = "ISOLATION VERIFIED" if marker else "REPORTED"
            elif verdict is None and delivery_status in {
                "result-error",
                "soft-denied",
                "cwd-mismatch",
                "permission-mode-mismatch",
                "conversation-mismatch",
                "invalid-stream",
                "invalid-report",
            }:
                verdict = (
                    "DELIVERY_FAILED_ISOLATION_VERIFIED"
                    if marker
                    else f"DELIVERY FAILED ({delivery_status})"
                )
            elif verdict is None:
                verdict = "INVALID EVIDENCE"
        print(f"{attempt_name}: {verdict}")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="verb", required=True)
    for verb in (
        "doctor",
        "dispatch",
        "resume",
        "snapshot",
        "verify",
        "status",
        "denied",
    ):
        item = sub.add_parser(verb)
        item.add_argument("profile")
        if verb in ("dispatch", "resume", "snapshot", "verify", "denied"):
            item.add_argument(
                "task_key",
                help="ticket issue id or explicit one-shot run id",
            )
    args = parser.parse_args()
    # A bounded-write task may legitimately change an allowed design input.
    # Freeze and validate those inputs before the first dispatch; later resume
    # rounds are instead bound by the existing snapshot, protected artifacts,
    # allowed write scope, and the ticket conversation. Rehashing writable
    # inputs on resume would make every successful first round unresumable.
    profile = load_profile(
        args.profile,
        validate_design=args.verb in ("snapshot", "dispatch"),
    )
    {
        "doctor": lambda: doctor(profile),
        "snapshot": lambda: snapshot(profile, args.task_key),
        "dispatch": lambda: dispatch(profile, args.task_key),
        "resume": lambda: resume(profile, args.task_key),
        "verify": lambda: verify(profile, args.task_key),
        "status": lambda: status(profile),
        "denied": lambda: denied(profile, args.task_key),
    }[args.verb]()


if __name__ == "__main__":
    main()
