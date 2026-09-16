#!/usr/bin/env python3
"""Fail-closed AGY PreToolUse guard for one frozen executor assignment."""

from __future__ import annotations

import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


POLICY_SCHEMA = "execution-agy-hook-policy-v2"
AUDIT_SCHEMA = "execution-agy-hook-audit-v2"
POLICY_ENV = "AXIOM_EXECUTION_GUARD_POLICY"
AUDIT_ENV = "AXIOM_EXECUTION_GUARD_AUDIT"
SAFE_DENIAL_CANARY = "git clean -nd -- ."
TASK_KEY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
ROLE_MODELS = {
    "qa": "gemini-3.8-flash-high",
    "dev": "gemini-3.8-flash-medium",
}

READ_PATH_FIELDS = {
    "view_file": "AbsolutePath",
    "list_dir": "DirectoryPath",
    "grep_search": "SearchPath",
    "find_by_name": "SearchDirectory",
}
WRITE_PATH_FIELDS = {
    "write_to_file": "TargetFile",
    "replace_file_content": "TargetFile",
    "multi_replace_file_content": "TargetFile",
}
PASSIVE_TOOLS = {"finish", "wait", "wait_5_seconds"}


class PolicyError(ValueError):
    """The controller policy or hook payload is not safe to use."""


def canonical_digest(value: Any) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(encoded).hexdigest()


def _string_list(value: object, field: str) -> list[str]:
    if not isinstance(value, list) or any(
        not isinstance(item, str) or not item for item in value
    ):
        raise PolicyError(f"{field} must be a list of non-empty strings")
    if len(value) != len(set(value)):
        raise PolicyError(f"{field} must not contain duplicates")
    return list(value)


def load_policy(path: Path) -> tuple[dict[str, Any], str]:
    if not path.is_absolute() or not path.is_file():
        raise PolicyError("guard policy must be an existing absolute path")
    try:
        raw = path.read_bytes()
        value = json.loads(raw)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise PolicyError(f"cannot read guard policy: {error}") from error
    if not isinstance(value, dict):
        raise PolicyError("guard policy must be a JSON object")
    required = {
        "schema",
        "assignment_digest",
        "task_key",
        "worktree_root",
        "workspace_roots",
        "executor_role",
        "model",
        "task_commands",
        "allowed_repo_writes",
        "expected_denied_commands",
    }
    if set(value) != required or value.get("schema") != POLICY_SCHEMA:
        raise PolicyError("guard policy keys or schema are invalid")
    if value.get("executor_role") not in ROLE_MODELS:
        raise PolicyError("guard executor_role must be qa or dev")
    assignment_digest = value.get("assignment_digest")
    if not isinstance(assignment_digest, str) or re.fullmatch(
        r"[0-9a-f]{64}", assignment_digest
    ) is None:
        raise PolicyError("guard assignment_digest must be a lowercase sha256")
    task_key = value.get("task_key")
    if not isinstance(task_key, str) or TASK_KEY_PATTERN.fullmatch(task_key) is None:
        raise PolicyError("guard task_key is invalid")
    if value.get("model") != ROLE_MODELS[value["executor_role"]]:
        raise PolicyError("guard model does not match executor_role")
    root_text = value.get("worktree_root")
    if not isinstance(root_text, str) or not Path(root_text).is_absolute():
        raise PolicyError("guard worktree_root must be absolute")
    value["worktree_root"] = str(Path(root_text).resolve())
    workspace_roots = _string_list(value.get("workspace_roots"), "workspace_roots")
    normalized_roots: list[str] = []
    for raw_root in workspace_roots:
        path = Path(raw_root)
        if not path.is_absolute():
            raise PolicyError("workspace_roots must contain absolute paths")
        normalized_roots.append(str(path.resolve()))
    if len(normalized_roots) != len(set(normalized_roots)):
        raise PolicyError("workspace_roots must not resolve to duplicates")
    if value["worktree_root"] not in normalized_roots:
        raise PolicyError("workspace_roots must include worktree_root")
    value["workspace_roots"] = normalized_roots
    commands = value.get("task_commands")
    if not isinstance(commands, dict) or set(commands) != {"allow", "deny"}:
        raise PolicyError("guard task_commands must contain allow and deny")
    commands["allow"] = _string_list(commands["allow"], "task_commands.allow")
    commands["deny"] = _string_list(commands["deny"], "task_commands.deny")
    if set(commands["allow"]) & set(commands["deny"]):
        raise PolicyError("guard command allow and deny lists must not overlap")
    value["allowed_repo_writes"] = _string_list(
        value["allowed_repo_writes"], "allowed_repo_writes"
    )
    for raw_path in value["allowed_repo_writes"]:
        path = Path(raw_path)
        if path.is_absolute() or ".." in path.parts or raw_path in {"", "."}:
            raise PolicyError("allowed_repo_writes must be repository-relative paths")
    value["expected_denied_commands"] = _string_list(
        value["expected_denied_commands"], "expected_denied_commands"
    )
    if not set(value["expected_denied_commands"]).issubset(commands["allow"]):
        raise PolicyError("expected denied commands must be controller-authorized attempts")
    if set(value["expected_denied_commands"]) - {SAFE_DENIAL_CANARY}:
        raise PolicyError("expected denied commands must use the fixed safe canary")
    return value, hashlib.sha256(raw).hexdigest()


def _inside(root: Path, raw_path: object) -> Path | None:
    if not isinstance(raw_path, str) or not raw_path:
        return None
    candidate = Path(raw_path)
    if not candidate.is_absolute():
        candidate = root / candidate
    try:
        resolved = candidate.resolve()
    except (OSError, RuntimeError):
        return None
    return resolved if resolved == root or resolved.is_relative_to(root) else None


def _base_event(
    policy: dict[str, Any], policy_digest: str, payload: object
) -> dict[str, Any]:
    value = payload if isinstance(payload, dict) else {}
    call = value.get("toolCall") if isinstance(value.get("toolCall"), dict) else {}
    args = call.get("args") if isinstance(call.get("args"), dict) else {}
    return {
        "schema": AUDIT_SCHEMA,
        "at": datetime.now(timezone.utc).isoformat(),
        "assignment_digest": policy["assignment_digest"],
        "task_key": policy["task_key"],
        "policy_sha256": policy_digest,
        "conversation_id": value.get("conversationId"),
        "step_index": value.get("stepIdx"),
        "workspace_paths": value.get("workspacePaths"),
        "model_name": value.get("modelName"),
        "tool_name": call.get("name"),
        "tool_args_digest": canonical_digest(args),
        "command": args.get("CommandLine"),
        "cwd": args.get("Cwd"),
        "target_path": None,
        "decision": "deny",
        "reason": "malformed or unauthorized tool request",
    }


def decide(
    policy: dict[str, Any], policy_digest: str, payload: object
) -> tuple[dict[str, str], dict[str, Any]]:
    """Return one AGY hook decision and its controller audit record."""
    event = _base_event(policy, policy_digest, payload)
    value = payload if isinstance(payload, dict) else {}
    root = Path(policy["worktree_root"])
    workspace_paths = value.get("workspacePaths")
    normalized_workspaces: list[str] = []
    if isinstance(workspace_paths, list):
        for path in workspace_paths:
            if not isinstance(path, str) or not Path(path).is_absolute():
                break
            normalized_workspaces.append(str(Path(path).resolve()))
    if normalized_workspaces not in ([path] for path in policy["workspace_roots"]):
        event["reason"] = "workspacePaths are not an authorized AGY Project root"
        return {"decision": "deny", "reason": event["reason"]}, event
    if value.get("modelName") != policy["model"]:
        event["reason"] = "modelName does not equal the frozen backend model"
        return {"decision": "deny", "reason": event["reason"]}, event
    if not isinstance(value.get("conversationId"), str) or not value["conversationId"]:
        event["reason"] = "conversationId is missing"
        return {"decision": "deny", "reason": event["reason"]}, event
    if not isinstance(value.get("stepIdx"), int) or value["stepIdx"] < 0:
        event["reason"] = "stepIdx is invalid"
        return {"decision": "deny", "reason": event["reason"]}, event

    call = value.get("toolCall")
    if not isinstance(call, dict) or not isinstance(call.get("name"), str):
        return {"decision": "deny", "reason": event["reason"]}, event
    tool_name = call["name"]
    args = call.get("args")
    if not isinstance(args, dict):
        event["reason"] = "tool args are invalid"
        return {"decision": "deny", "reason": event["reason"]}, event

    if tool_name == "run_command":
        command = args.get("CommandLine")
        cwd = args.get("Cwd")
        if (
            not isinstance(cwd, str)
            or not Path(cwd).is_absolute()
            or Path(cwd).resolve() != root
        ):
            event["reason"] = "run_command cwd does not equal the assigned worktree"
        elif args.get("RunPersistent") is True:
            event["reason"] = "persistent commands are not allowed"
        elif command in policy["expected_denied_commands"]:
            event["reason"] = "expected assignment isolation denial"
        elif command in policy["task_commands"]["deny"]:
            event["reason"] = "command is explicitly denied by the assignment"
        elif command not in policy["task_commands"]["allow"]:
            event["reason"] = "command is outside the exact assignment allowlist"
        else:
            event["decision"] = "allow"
            event["reason"] = "exact assignment command and cwd"
    elif tool_name in READ_PATH_FIELDS:
        field = READ_PATH_FIELDS[tool_name]
        target = _inside(root, args.get(field))
        event["target_path"] = str(target) if target is not None else None
        if target is None:
            event["reason"] = "read tool path is outside the assigned worktree"
        else:
            event["decision"] = "allow"
            event["reason"] = "read tool path is inside the assigned worktree"
    elif tool_name in WRITE_PATH_FIELDS:
        field = WRITE_PATH_FIELDS[tool_name]
        target = _inside(root, args.get(field))
        event["target_path"] = str(target) if target is not None else None
        allowed_targets = {
            str((root / relative).resolve())
            for relative in policy["allowed_repo_writes"]
        }
        if policy["executor_role"] != "dev":
            event["reason"] = "QA assignments cannot use file-write tools"
        elif target is None or str(target) not in allowed_targets:
            event["reason"] = "write tool path is outside the exact write allowlist"
        else:
            event["decision"] = "allow"
            event["reason"] = "write tool path equals an allowed repository path"
    elif tool_name in PASSIVE_TOOLS:
        event["decision"] = "allow"
        event["reason"] = "passive completion tool"
    else:
        event["reason"] = "tool is not available to a bounded executor"

    return {
        "decision": event["decision"],
        "reason": event["reason"],
    }, event


def append_audit(path: Path, event: dict[str, Any]) -> None:
    if not path.is_absolute():
        raise PolicyError("guard audit path must be absolute")
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
    try:
        os.write(
            descriptor,
            (json.dumps(event, sort_keys=True, separators=(",", ":")) + "\n").encode(),
        )
    finally:
        os.close(descriptor)
    os.chmod(path, 0o600)


def main() -> None:
    policy_text = os.environ.get(POLICY_ENV)
    audit_text = os.environ.get(AUDIT_ENV)
    if not policy_text or not audit_text:
        print(
            json.dumps(
                {
                    "decision": "deny",
                    "reason": "no frozen executor assignment is active",
                }
            )
        )
        return
    try:
        policy, policy_digest = load_policy(Path(policy_text))
        payload = json.loads(sys.stdin.read())
        decision, event = decide(policy, policy_digest, payload)
        append_audit(Path(audit_text), event)
    except Exception as error:  # A guard failure must never become an allow.
        decision = {
            "decision": "deny",
            "reason": f"executor assignment guard failed closed: {error}",
        }
    print(json.dumps(decision, sort_keys=True, separators=(",", ":")))


if __name__ == "__main__":
    main()
