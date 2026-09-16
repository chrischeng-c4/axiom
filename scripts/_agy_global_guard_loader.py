#!/usr/bin/env python3
"""Static AGY global-hook loader for active frozen executor assignments."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import stat
import sys
from pathlib import Path
from typing import Any


TEMP_ROOT = Path("/tmp/execution/agy").resolve()
REGISTRATION_SCHEMA = "execution-agy-active-guard-v2"
CORE_FILENAME = "agy-assignment-guard-core.py"


def emit(decision: str, reason: str) -> None:
    print(json.dumps({"decision": decision, "reason": reason}, sort_keys=True))


def regular_private_file(path: Path) -> bool:
    try:
        detail = path.lstat()
    except OSError:
        return False
    return (
        stat.S_ISREG(detail.st_mode)
        and not path.is_symlink()
        and detail.st_uid == os.getuid()
        and detail.st_mode & 0o077 == 0
    )


def workspace_root(payload: object) -> Path | None:
    value = payload if isinstance(payload, dict) else {}
    paths = value.get("workspacePaths")
    if not isinstance(paths, list) or len(paths) != 1:
        return None
    root = paths[0]
    if not isinstance(root, str) or not Path(root).is_absolute():
        return None
    try:
        return Path(root).resolve()
    except (OSError, RuntimeError):
        return None


def registration_candidates() -> list[Path]:
    candidates: list[Path] = []
    for candidate in TEMP_ROOT.glob("*/*/runs/*.active-guard.json"):
        try:
            resolved = candidate.resolve()
        except (OSError, RuntimeError):
            continue
        if resolved.parent.parent.parent.parent != TEMP_ROOT:
            continue
        if regular_private_file(resolved):
            candidates.append(resolved)
    return sorted(candidates)


def read_registration(path: Path, root: Path) -> dict[str, str] | None:
    try:
        value = json.loads(path.read_text())
    except (OSError, UnicodeError, json.JSONDecodeError):
        return None
    required = {
        "schema",
        "worktree_root",
        "workspace_roots",
        "policy_path",
        "audit_path",
        "core_sha256",
    }
    if not isinstance(value, dict) or set(value) != required:
        return None
    if value.get("schema") != REGISTRATION_SCHEMA:
        return None
    string_keys = required - {"schema", "workspace_roots"}
    if any(not isinstance(value.get(key), str) or not value[key] for key in string_keys):
        return None
    try:
        registered_root = Path(value["worktree_root"]).resolve()
        workspace_roots = value["workspace_roots"]
        if not isinstance(workspace_roots, list) or not workspace_roots:
            return None
        registered_roots = [
            Path(candidate).resolve()
            for candidate in workspace_roots
            if isinstance(candidate, str) and Path(candidate).is_absolute()
        ]
        policy = Path(value["policy_path"]).resolve()
        audit = Path(value["audit_path"]).resolve()
    except (OSError, RuntimeError):
        return None
    if (
        len(registered_roots) != len(workspace_roots)
        or len(registered_roots) != len(set(registered_roots))
        or registered_root not in registered_roots
        or root not in registered_roots
    ):
        return None
    if policy.parent != path.parent or audit.parent != path.parent:
        return None
    if not regular_private_file(policy) or not regular_private_file(audit):
        return None
    if len(value["core_sha256"]) != 64 or any(
        character not in "0123456789abcdef" for character in value["core_sha256"]
    ):
        return None
    return {
        "policy_path": str(policy),
        "audit_path": str(audit),
        "core_sha256": value["core_sha256"],
    }


def active_registration(root: Path) -> dict[str, str] | None:
    matches = [
        registration
        for candidate in registration_candidates()
        if (registration := read_registration(candidate, root)) is not None
    ]
    if len(matches) > 1:
        raise RuntimeError("multiple active guard registrations match this workspace")
    return matches[0] if matches else None


def load_core(expected_sha256: str) -> Any:
    path = Path(__file__).resolve().with_name(CORE_FILENAME)
    if not regular_private_file(path):
        raise RuntimeError("installed assignment guard core is missing or unsafe")
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != expected_sha256:
        raise RuntimeError("installed assignment guard core digest does not match registration")
    spec = importlib.util.spec_from_file_location("agy_assignment_guard_core", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load installed assignment guard core")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main() -> None:
    try:
        payload = json.loads(sys.stdin.read())
    except json.JSONDecodeError:
        emit("deny", "global assignment guard received malformed hook payload")
        return
    root = workspace_root(payload)
    if root is None:
        emit("deny", "global assignment guard received invalid workspacePaths")
        return
    try:
        registration = active_registration(root)
        if registration is None:
            emit("allow", "no frozen executor assignment is active")
            return
        core = load_core(registration["core_sha256"])
        policy, policy_digest = core.load_policy(Path(registration["policy_path"]))
        decision, event = core.decide(policy, policy_digest, payload)
        core.append_audit(Path(registration["audit_path"]), event)
    except Exception as error:  # A loader error must never become an allow.
        emit("deny", f"global assignment guard failed closed: {error}")
        return
    emit(decision["decision"], decision["reason"])


if __name__ == "__main__":
    main()
