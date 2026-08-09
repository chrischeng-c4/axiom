#!/usr/bin/env python3
"""Read-only project-policy controller for bounded headless AGY dispatches."""
from __future__ import annotations

import argparse
import base64
import difflib
import hashlib
import json
import re
import shlex
import sqlite3
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path
from urllib.parse import unquote, urlparse


HOME = Path.home()
SETTINGS = HOME / ".gemini" / "antigravity-cli" / "settings.json"
GLOBAL = HOME / ".gemini" / "config" / "config.json"
PROJECT_DIR = HOME / ".gemini" / "config" / "projects"
CONVERSATION_DIR = HOME / ".gemini" / "antigravity-cli" / "conversations"
TEMP_ROOT = Path("/tmp").resolve()
PERMISSION_KINDS = ("allow", "deny", "ask")
# The two kinds whose loss widens the worker. Dropping an `allow` narrows it,
# which is the safe direction and is already refused elsewhere if it goes too
# far; dropping a `deny` or an `ask` removes a guard, and nothing noticed.
PROTECTIVE_KINDS = ("deny", "ask")
# `allow` is one exact command line; `allow_prefix` is an open family, matched
# the way the permission layer itself matches. A controller means one or the
# other, and until both were sayable only the first was: a read-only verb such
# as `git log` went into `allow`, where no real invocation ever equals it and
# every real invocation is admitted by the prefix rule beside it.
TASK_COMMAND_KINDS = ("allow", "allow_prefix", "deny")
TASK_KEY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")

# Every git call below reads or writes state directly and none of them needs
# the fsmonitor daemon's cache. That daemon is a liability here: on a large
# working tree it stops answering, and then any git command that touches the
# index blocks in `fsmonitor_ipc__send_query` -> `read()` indefinitely -- 0%
# CPU, no output, no timeout. A controller stuck there looks exactly like a
# worker that is still thinking, which is the one failure this tool must never
# imitate. Bypassing the client costs one index refresh and cannot change what
# git reports, so it is always the right trade for a round's control plane.
GIT = ("git", "-c", "core.fsmonitor=false")

# Derived-worktree dispatch. The worker gets its own checkout on its own
# branch, so the controller keeps working while a round is in flight and a
# scope overrun is a diff to read rather than a destroyed round. The AGY
# Project stays one per work area and is rebound to the round's worktree.
DERIVED_BRANCH_PREFIX = "agy/"
# Where a dispatch that produced nothing is parked. It sits below `runs/` so
# the id lookup, which globs that directory for the task key, stops finding the
# dead conversation once its logs move here.
ABANDONED_RUNS = "abandoned"
EXIT_VOID = 1
EXIT_FINDINGS = 2

# AGY's conversation store records a shell command as two rows: the request and
# what became of it. The request always carries status 3, so the outcome row is
# the only place that says whether the command ran.
RUN_COMMAND_REQUEST_STEP = 15
RUN_COMMAND_OUTCOME_STEP = 21
RUN_COMMAND_DENIED_STATUS = 7


def resolved_under(path: Path, root: Path) -> bool:
    """Containment on real paths. `/tmp` is a symlink on macOS, so comparing the
    literal strings answers a different question than the one being asked."""
    try:
        return path.resolve().is_relative_to(root.resolve())
    except OSError:
        return False


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


def load_profile(
    path: str,
    *,
    validate_design: bool = True,
    require_injection: bool | None = None,
    task_key: str | None = None,
) -> dict:
    if require_injection is None:
        require_injection = validate_design
    profile = json.loads(Path(path).read_text())
    required = (
        "root",
        "repo",
        "state_dir",
        "mode",
        "project_permissions",
        "task_commands",
        "protected_artifacts",
        "snapshot_paths",
        "allowed_repo_writes",
    )
    missing = [key for key in required if key not in profile]
    if missing:
        legacy = (
            " The legacy `permissions` profile rewrote AGY project files per "
            "ticket and is no longer supported; split it into persistent "
            "`project_permissions` and ticket-local `task_commands`."
            if "permissions" in profile
            else ""
        )
        raise SystemExit(f"profile missing: {', '.join(missing)}.{legacy}")

    root = Path(profile["root"]).resolve()
    if not root.is_dir():
        raise SystemExit(f"root is not a directory: {root}")
    profile["root"] = str(root)

    state_dir = Path(profile["state_dir"]).resolve()
    if state_dir != TEMP_ROOT and not state_dir.is_relative_to(TEMP_ROOT):
        raise SystemExit(
            "state_dir must be under /tmp/agy-dispatch so controller state "
            "remains transient and shared by Claude and Codex"
        )
    if state_dir == root or state_dir.is_relative_to(root):
        raise SystemExit(
            "state_dir must be outside the repository so controller evidence "
            "does not appear as an AGY repository mutation"
        )
    profile["state_dir"] = str(state_dir)

    if profile["mode"] not in ("measure-only", "bounded-write"):
        raise SystemExit("mode must be measure-only or bounded-write")
    if profile["mode"] == "measure-only" and profile["allowed_repo_writes"]:
        raise SystemExit("measure-only profile cannot grant repository writes")

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
        if not task.get("gate_command"):
            raise SystemExit(
                "bounded-write requires task_contract.gate_command: the one "
                "command whose red/green decides the round. `accept` refuses "
                "until that gate is shown to fail without the change."
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

    project_permissions = profile["project_permissions"]
    if not isinstance(project_permissions, dict):
        raise SystemExit("project_permissions must be an object")
    for kind in PERMISSION_KINDS:
        project_permissions[kind] = validate_rule_list(
            project_permissions.get(kind, []),
            f"project_permissions.{kind}",
        )
    # The invariant that matters is that the worker's effective surface is no
    # wider than the profile declares. `require_empty_global` was a proxy for
    # it: cheap to state, but false whenever a harmless inherited deny rule
    # exists, which pushes the controller toward flipping the flag rather than
    # answering the question. `global_broadening_rules` measures the real thing,
    # so the flag survives only as opt-in extra strictness.
    require_empty = project_permissions.get("require_empty_global", False)
    if not isinstance(require_empty, bool):
        raise SystemExit("project_permissions.require_empty_global must be boolean")
    project_permissions["require_empty_global"] = require_empty

    # A protective rule the Project holds and this profile omits is dropped by
    # `grant`, which installs the declared surface verbatim. Dropping one can be
    # right, so the escape hatch is not a flag: each rule given up is named,
    # in the `<kind> <rule>` form the refusal prints, so it can be copied across
    # and so a rule added to the Project later shows up as a new, unnamed loss
    # rather than disappearing under a blanket permission.
    revocations = validate_rule_list(
        project_permissions.get("intentional_revocations", []),
        "project_permissions.intentional_revocations",
    )
    for entry in revocations:
        kind = entry.split(" ", 1)[0]
        if kind not in PROTECTIVE_KINDS or " " not in entry:
            raise SystemExit(
                "project_permissions.intentional_revocations entries are "
                f"`<{'|'.join(PROTECTIVE_KINDS)}> <rule>`, not: {entry}"
            )
    project_permissions["intentional_revocations"] = revocations

    task_commands = profile["task_commands"]
    if not isinstance(task_commands, dict):
        raise SystemExit("task_commands must be an object")
    for kind in TASK_COMMAND_KINDS:
        task_commands[kind] = validate_rule_list(
            task_commands.get(kind, []),
            f"task_commands.{kind}",
        )
    overlap = sorted(
        (set(task_commands["allow"]) | set(task_commands["allow_prefix"]))
        & set(task_commands["deny"])
    )
    if overlap:
        raise SystemExit(
            "task_commands cannot both allow and deny: " + ", ".join(overlap)
        )
    both = sorted(set(task_commands["allow"]) & set(task_commands["allow_prefix"]))
    if both:
        raise SystemExit(
            "task_commands entries must be exact or a prefix family, not both: "
            + ", ".join(both)
        )

    profile["snapshot_paths"] = [
        relative_repo_path(root, value, "snapshot_paths")
        for value in profile["snapshot_paths"]
    ]
    profile["allowed_repo_writes"] = [
        relative_repo_path(root, value, "allowed_repo_writes")
        for value in profile["allowed_repo_writes"]
    ]

    budgets = profile.get("path_change_budgets", {})
    if not isinstance(budgets, dict):
        raise SystemExit("path_change_budgets must be an object")
    unknown_budget_paths = sorted(set(budgets) - set(profile["allowed_repo_writes"]))
    if unknown_budget_paths:
        raise SystemExit(
            "path_change_budgets contains non-writable paths: "
            + ", ".join(unknown_budget_paths)
        )

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
        elif not resolved_under(artifact, root):
            # An absolute protected path from another root would make `verify`
            # hash a tree the worker never touched: it would miss a real
            # protected mutation and void on the controller's own edits. Emit
            # repo-relative paths so one profile follows its round's worktree.
            raise SystemExit(
                f"protected artifact is outside the round root: {artifact}\n"
                f"root is {root}. Regenerate the profile with repo-relative "
                "protected paths."
            )
        if validate_design:
            if not artifact.is_file():
                raise SystemExit(f"protected artifact is missing: {artifact}")
            actual = sha256(artifact)
            if actual != entry["sha256"]:
                if task_key and has_valid_adjudication(
                    state_dir, task_key, str(artifact), actual
                ):
                    pass
                else:
                    raise SystemExit(f"protected artifact hash mismatch: {artifact}")

    # Every verb that runs at or before authoring time — `doctor` preflights the
    # round, `scaffold` is what creates the injection, `lint` grades it — sees a
    # missing file as the expected starting state, not a fault. `lint` reports it
    # as a finding and `dispatch` refuses on that finding, so nothing reaches a
    # worker unchecked.
    inject_prompt_file = profile.get("inject_prompt_file")
    if require_injection and inject_prompt_file:
        if not Path(inject_prompt_file).is_file():
            raise SystemExit(
                f"inject_prompt_file is not a file: {inject_prompt_file}"
            )
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


def project_root(project: dict) -> Path | None:
    for resource in project.get("projectResources", {}).get("resources", []):
        value = resource.get("gitFolder", {}).get("folderUri") or resource.get(
            "folderUri"
        )
        if not value:
            continue
        parsed = urlparse(value)
        if parsed.scheme == "file":
            return Path(unquote(parsed.path)).resolve()
    return None


def project_path_by_id(project_id: str) -> Path:
    path = PROJECT_DIR / f"{project_id}.json"
    if not path.is_file():
        raise SystemExit(
            f"AGY project is missing: {path}. Create it once with "
            "`agy --new-project`, then configure Project-scope rules with "
            "`/permissions`."
        )
    return path


def project_ids_for_root(root: Path) -> list[str]:
    matches = []
    for path in sorted(PROJECT_DIR.glob("*.json")):
        project = json.loads(path.read_text())
        if project_root(project) == root:
            matches.append(project.get("id") or path.stem)
    return matches


def agy_project_id(profile: dict) -> str:
    explicit = profile.get("agy_project_id")
    if explicit:
        project = json.loads(project_path_by_id(explicit).read_text())
        resolved = project_root(project)
        requested = Path(profile["root"]).resolve()
        if resolved != requested:
            raise SystemExit(
                f"AGY project {explicit} is registered for {resolved}, "
                f"not profile root {requested}"
            )
        return explicit
    root = Path(profile["root"]).resolve()
    matches = project_ids_for_root(root)
    if len(matches) != 1:
        detail = "none" if not matches else ", ".join(matches)
        raise SystemExit(
            f"expected one AGY project for {root}; found {detail}. "
            "Create/register the project once and set agy_project_id explicitly."
        )
    return matches[0]


def read_json_or_empty(path: Path) -> dict:
    if not path.is_file():
        return {}
    return json.loads(path.read_text())


def project_permission_surface(project: dict) -> dict[str, list[str]]:
    grants = project.get("permissionGrants", {}).get("permissionGrants", {})
    return normalize_permission_surface(grants)


def global_permission_surface() -> dict[str, list[str]]:
    settings = read_json_or_empty(SETTINGS)
    config = read_json_or_empty(GLOBAL)
    settings_permissions = settings.get("permissions", {})
    global_grants = config.get("userSettings", {}).get(
        "globalPermissionGrants", {}
    )
    return normalize_permission_surface(
        {
            kind: [
                *(settings_permissions.get(kind, []) or []),
                *(global_grants.get(kind, []) or []),
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


def rule_matches(rule: str, command: str, prefix: str = "command") -> bool:
    match = re.fullmatch(rf"{prefix}\((.*)\)", rule)
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


def command_rule_matches(rule: str, command: str) -> bool:
    return rule_matches(rule, command, "command")


def unsandboxed_rules(allow: list[str]) -> list[str]:
    return [rule for rule in allow if re.fullmatch(r"unsandboxed\(.*\)", rule)]


def runs_unsandboxed(
    project_surface: dict[str, list[str]],
    global_surface: dict[str, list[str]],
    command: str,
) -> bool:
    return any(
        rule_matches(rule, command, "unsandboxed")
        for surface in (project_surface, global_surface)
        for rule in unsandboxed_rules(surface.get("allow", []))
    )


def inert_unsandboxed_rules(surface: dict[str, list[str]]) -> list[str]:
    """`unsandboxed(P)` rules with no `command(P)` twin, which can never fire.

    Sandbox escape is only ever consulted after a command has already resolved
    to `allow`. An `unsandboxed` rule whose pattern no `command` rule admits is
    therefore inert: the command stops at `ask` and the worker blocks on a
    prompt no one is there to answer, while the profile reads as though the
    round granted it. This is decidable from the two lists alone, unlike the
    opposite direction -- whether a given command *needs* the escape is a
    judgement about network and out-of-tree writes, so `doctor` reports that as
    data rather than guessing at it.
    """
    allow = surface.get("allow", [])
    inert = []
    for rule in unsandboxed_rules(allow):
        pattern = re.fullmatch(r"unsandboxed\((.*)\)", rule).group(1)
        twin = f"command({pattern})"
        if twin not in allow:
            inert.append(rule)
    return sorted(inert)


def permission_decision(
    project_surface: dict[str, list[str]],
    global_surface: dict[str, list[str]],
    command: str,
) -> tuple[str, str | None]:
    effective = {
        kind: [
            *project_surface.get(kind, []),
            *global_surface.get(kind, []),
        ]
        for kind in PERMISSION_KINDS
    }
    for kind in ("deny", "ask", "allow"):
        for rule in effective[kind]:
            if command_rule_matches(rule, command):
                return kind, rule
    return "ask", None


def expected_project_surface(profile: dict) -> dict[str, list[str]]:
    return normalize_permission_surface(profile["project_permissions"])


def permission_state(profile: dict) -> dict:
    project_id = agy_project_id(profile)
    project_path = project_path_by_id(project_id)
    project = json.loads(project_path.read_text())
    return {
        "project_id": project_id,
        "project_path": str(project_path),
        "project": project_permission_surface(project),
        "global": global_permission_surface(),
    }


def permission_state_digest(profile: dict) -> str:
    state = permission_state(profile)
    return json_digest(
        {
            "project_id": state["project_id"],
            "project": state["project"],
            "global": state["global"],
        }
    )


def project_policy_report(profile: dict) -> dict:
    state = permission_state(profile)
    expected = expected_project_surface(profile)
    actual = state["project"]
    missing = {
        kind: sorted(set(expected[kind]) - set(actual[kind]))
        for kind in PERMISSION_KINDS
    }
    extra = {
        kind: sorted(set(actual[kind]) - set(expected[kind]))
        for kind in PERMISSION_KINDS
    }
    global_surface = state["global"]
    global_nonempty = any(global_surface[kind] for kind in PERMISSION_KINDS)
    # Only an inherited `allow` can widen the worker beyond the declared
    # surface. Inherited deny/ask rules can only narrow it, so they are
    # reported but are not blockers.
    global_broadening = sorted(set(global_surface["allow"]) - set(expected["allow"]))
    command_checks = []
    blockers = []

    if any(missing[kind] or extra[kind] for kind in PERMISSION_KINDS):
        blockers.append(
            "Project-scope permission rules differ from project_permissions"
        )
    # The same two omissions `grant` refuses on. Repeated here because a
    # surface can reach the Project by a route `grant` never saw, and every
    # other check in this report iterates a list those omissions leave empty.
    blockers.extend(unauthorized_surface_refusals(profile))
    dropped = unnamed_protective_revocations(profile)
    if dropped:
        blockers.append(
            f"the declared surface drops {len(dropped)} guard(s) the Project "
            "held at the start of this round, unnamed in "
            "`project_permissions.intentional_revocations`: "
            + ", ".join(dropped)
        )
    if global_broadening:
        blockers.append(
            "inherited global rules allow "
            f"{len(global_broadening)} command(s) the profile does not declare: "
            + ", ".join(global_broadening)
        )
    if (
        profile["project_permissions"].get("require_empty_global", False)
        and global_nonempty
    ):
        blockers.append(
            "global permission rules are inherited; move them to the AGY "
            "Project scope or explicitly revise the project policy"
        )

    inert_escapes = inert_unsandboxed_rules(actual)
    if inert_escapes:
        blockers.append(
            f"{len(inert_escapes)} unsandboxed rule(s) have no `command(...)` "
            "twin and can never fire: " + ", ".join(inert_escapes)
        )

    # A path cannot be both the round's write target and frozen against
    # writing. `make_profile.py` cannot emit this, because it freezes the
    # complement of the write set; a profile derived from an earlier round's
    # can, because deriving edits the write set and leaves the complement
    # describing the round it came from. The result is a finding that fires on
    # correct work, and a finding that always fires is one the controller
    # learns to skim past (#3428).
    root = Path(profile["root"])
    write_map = {}
    for write in profile["allowed_repo_writes"]:
        w_path = Path(write)
        if not w_path.is_absolute():
            w_path = root / w_path
        write_map[w_path.resolve()] = write

    frozen_targets_list = []
    for entry in profile["protected_artifacts"]:
        p_path = Path(entry["path"])
        if not p_path.is_absolute():
            p_path = root / p_path
        if p_path.resolve() in write_map:
            frozen_targets_list.append(write_map[p_path.resolve()])
    frozen_targets = sorted(set(frozen_targets_list))
    if frozen_targets:
        blockers.append(
            f"{len(frozen_targets)} declared write target(s) are also frozen "
            "as protected artifacts, so writing them is a finding and not "
            "writing them is a finding: " + ", ".join(frozen_targets)
        )

    # A prefix family is probed as an expected-allow under its own text, which
    # is the shortest line the family admits: a surface that refuses that
    # refuses every member. Leaving `allow_prefix` out of this loop would let a
    # round declare a family the worker's permission surface never grants and
    # still report dispatch_ready.
    task_commands = profile.get("task_commands", {})
    probes = (
        [(command, "allow", "allow") for command in task_commands.get("allow", [])]
        + [
            (command, "allow", "allow_prefix")
            for command in task_commands.get("allow_prefix", [])
        ]
        + [(command, "deny", "deny") for command in task_commands.get("deny", [])]
    )
    for command, expected_decision, kind in probes:
        decision, rule = permission_decision(actual, global_surface, command)
        check = {
            "command": command,
            "kind": kind,
            "expected": expected_decision,
            "decision": decision,
            "matched_rule": rule,
            # Reported, never blocked: whether a command needs to leave
            # the sandbox depends on whether it touches the network or
            # writes outside the worktree, which the profile cannot
            # state. A sandboxed `cargo` is the failure that reads like
            # a product defect, so the controller sees this per command.
            "unsandboxed": runs_unsandboxed(actual, global_surface, command),
        }
        if kind == "allow":
            # The question `doctor` could not previously ask: not whether the
            # declared command is allowed, which was never in doubt, but whether
            # the surface also admits argument forms past it. Probing one token
            # past the entry answers it. The answer is normally yes -- every
            # permission rule is a token prefix, so nothing on that layer can
            # express "this line and no longer one" -- and that is precisely the
            # fact an author needs at authoring time: an exact entry states the
            # controller's intent, and the surface will not enforce it. Reported,
            # never blocked, because blocking on it would block every round.
            probe = f"{command} agy-dispatch-argument-probe"
            check["argument_forms_admitted"] = (
                permission_decision(actual, global_surface, probe)[0] == "allow"
                and not task_allowlist_admits(profile, probe)
            )
        command_checks.append(check)
        if decision != expected_decision:
            blockers.append(
                f"task command expected {expected_decision} but resolves "
                f"{decision}: {command}"
            )

    return {
        "project_id": state["project_id"],
        "project_path": state["project_path"],
        "project_root": profile["root"],
        "project_permission_digest": json_digest(actual),
        "permission_state_digest": permission_state_digest(profile),
        "project_permissions_status": (
            "ready"
            if not any(missing[kind] or extra[kind] for kind in PERMISSION_KINDS)
            else "drift"
        ),
        "global_permissions_status": (
            "empty"
            if not global_nonempty
            else "inherited-broadening"
            if global_broadening
            else "inherited-narrowing-only"
        ),
        "missing_project_rules": missing,
        "extra_project_rules": extra,
        "global_rules": global_surface,
        "global_broadening_rules": global_broadening,
        "unnamed_protective_revocations": dropped,
        "task_command_checks": command_checks,
        "dispatch_ready": not blockers,
        "blockers": blockers,
    }


def doctor(profile: dict) -> dict:
    report = project_policy_report(profile)
    print(json.dumps(report, indent=2))
    if not report["dispatch_ready"]:
        raise SystemExit(2)
    return report


def require_project_ready(profile: dict) -> dict:
    report = project_policy_report(profile)
    if not report["dispatch_ready"]:
        raise SystemExit(
            "AGY Project policy is not ready; run `doctor` and configure the "
            "persistent Project scope with `/permissions`: "
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


def dispatch_contract(profile: dict) -> dict:
    return {
        key: profile.get(key)
        for key in (
            "mode",
            "task_contract",
            "project_permissions",
            "task_commands",
            "protected_artifacts",
            "snapshot_paths",
            "allowed_repo_writes",
            "path_change_budgets",
        )
    }


def conversation_id_for_task(profile: dict, task_key: str) -> str | None:
    log_dir = Path(profile["state_dir"]) / "runs"
    conversation_path = log_dir / f"{task_key}.conversation"
    if conversation_path.is_file() and conversation_path.read_text().strip():
        return conversation_path.read_text().strip()
    return conversation_id_from_log(log_dir / f"{task_key}.agy.log")


def conversation_database(conversation_id: str) -> Path:
    return CONVERSATION_DIR / f"{conversation_id}.db"


def connect_conversation(database: Path) -> sqlite3.Connection:
    """Open AGY's conversation store read-only, WAL sidecar or not.

    A WAL-mode database needs its `-shm` sidecar to be readable, and SQLite
    creates that sidecar on open -- which `mode=ro` cannot do once AGY has
    exited and checkpointed the sidecars away. Every reader here then dies on
    `unable to open database file`, and the one verb that exists to explain a
    failed dispatch fails with it.

    `immutable=1` skips the sidecar, at the cost of promising the file is not
    being written. That promise is exactly what the first open failing already
    told us, so it is the fallback and never the first attempt: a live AGY
    keeps its sidecars present, so the plain read-only open succeeds and the
    unsafe path is not reached.
    """
    # `sqlite3.connect` defers the real open to the first statement, so the
    # fallback has to be driven by a query, not by construction.
    for uri in (f"file:{database}?mode=ro", f"file:{database}?mode=ro&immutable=1"):
        connection = sqlite3.connect(uri, uri=True)
        try:
            connection.execute("select 1 from sqlite_master limit 1").fetchone()
            return connection
        except sqlite3.OperationalError:
            connection.close()
    raise SystemExit(f"cannot read AGY conversation store: {database}")


def conversation_step_max(conversation_id: str | None) -> int:
    if not conversation_id:
        return -1
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(
            f"conversation state is missing for {conversation_id}: {database}"
        )
    connection = connect_conversation(database)
    try:
        row = connection.execute("select max(idx) from steps").fetchone()
    finally:
        connection.close()
    return -1 if not row or row[0] is None else int(row[0])


def round_document_digests(profile: dict, task_key: str) -> dict[str, str | None]:
    """The oracle and injection as they read at snapshot time.

    These two files are the only statement of what the round asked for and how
    it will be judged, and the controller is the one party able to edit them
    after the worker is already running. Everything else in the round -- the
    permission surface, the dispatch contract, the protected repo files, the
    conversation lineage -- is frozen and compared, so freeze these the same
    way; an oracle that can be retro-fitted to the answer it received is not an
    oracle. `None` records a document that was absent, which has to stay absent:
    a round that grew an injection after the snapshot is exactly as suspect as
    one whose injection changed.
    """
    return {
        "oracle": (
            sha256(path) if (path := oracle_path(profile, task_key)).is_file() else None
        ),
        "injection": (
            sha256(path)
            if (path := injection_path(profile, task_key)).is_file()
            else None
        ),
    }


def assert_round_documents_unchanged(
    profile: dict, task_key: str, snapshot_data: dict
) -> bool:
    """Whether the round's two documents were compared and matched.

    Returns False for a snapshot taken before this check existed, so callers
    can say "not compared" instead of "unchanged" -- the distinction the old
    unverified `oracle sha256=` line erased.
    """
    expected = snapshot_data.get("round_documents")
    if expected is None:
        print("note: snapshot predates round-document freezing; not compared")
        return False
    actual = round_document_digests(profile, task_key)
    for name in ("oracle", "injection"):
        if actual[name] == expected[name]:
            continue
        raise SystemExit(
            f"VOID: {name} changed after snapshot: "
            f"{expected[name]} -> {actual[name]}"
        )
    return True


def snapshot(profile: dict, task_key: str) -> Path:
    readiness = require_project_ready(profile)
    task_state = frozen_task_state(profile, task_key)
    root = Path(profile["root"])
    state = Path(profile["state_dir"])
    (state / "snapshots").mkdir(parents=True, exist_ok=True)
    result = subprocess.run(
        [*GIT, "-C", str(root), "status", "--porcelain=v1", "--untracked-files=all"],
        text=True,
        capture_output=True,
        check=True,
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
    payload = {
        "task_key": task_key,
        "session_policy": task_session_policy(profile),
        "issue": profile["task_contract"].get("issue"),
        "at": datetime.now(timezone.utc).isoformat(),
        "git_status": result.stdout,
        "manifest": manifest(root, profile["snapshot_paths"]),
        "protected_artifacts": artifact_hashes,
        "protected_contents_base64": protected_contents,
        "writable_contents": writable_contents,
        "dispatch_contract": dispatch_contract(profile),
        "round_documents": round_document_digests(profile, task_key),
        "agy_project_id": agy_project_id(profile),
        "permission_state_digest": readiness["permission_state_digest"],
        "conversation_id": conversation_id,
        "conversation_step_floor": conversation_step_max(conversation_id),
        "task_state": task_state,
        "live_issue": (
            task_state
            if task_session_policy(profile) == "ticketed"
            else None
        ),
    }
    output = state / "snapshots" / f"{task_key}.json"
    output.write_text(json.dumps(payload, indent=2) + "\n")
    print(output)
    return output


HTML_COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)


def controller_notes_removed(text: str) -> str:
    """Strip the round form's own guidance before the worker sees the document.

    Both halves of the round are authored by filling a form whose slots carry
    the rule that governs them, and those rules are addressed to the controller:
    what a vacuous measurement looks like, why a stale quote is dangerous, which
    slot a condition belongs in. Reaching the worker they stop being guidance
    and read as task instructions, and a slot's rationale is the one thing a
    bounded executor has no use for -- it is material to reason around a
    constraint with. `lint` already refuses a document with `<!-- fill -->`
    slots left in it, so this is what makes the separation structural rather
    than a habit: nothing written in a comment can reach the worker.
    """
    return HTML_COMMENT.sub("", text).strip()


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
    families = profile["task_commands"].get("allow_prefix", [])
    denied = profile["task_commands"].get("deny", [])
    # A worker with no ticket command and no project allow rule cannot observe
    # anything. Demanding PASS/FAIL per criterion from it manufactures the exact
    # fabrication the controller then has to detect, so the report contract asks
    # for a description of the work instead of a verdict on it.
    no_shell = (
        not allowed
        and not families
        and not profile["project_permissions"].get("allow", [])
    )
    report_contract = (
        NO_SHELL_REPORT_CONTRACT if no_shell else VERDICT_REPORT_CONTRACT
    )
    injection = ""
    inject_prompt_file = profile.get("inject_prompt_file")
    if inject_prompt_file:
        injection = controller_notes_removed(Path(inject_prompt_file).read_text())
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
    # A family is only shown when one was declared. Printing an empty heading
    # would invite the worker to read the absence as permission to invent a
    # prefix, which is the failure this whole section exists to prevent.
    family_block = (
        "Shell command prefixes authorized for this task, each of which may be "
        "followed\nby further arguments:\n"
        + "\n".join(f"- {command} ..." for command in families)
        + "\n\n"
        if families
        else ""
    )
    copy_rule = (
        "Every Bash tool call must either copy one exact command line "
        "byte-for-byte, or\nbegin with one of the authorized prefixes above and "
        "add only arguments after it.\nA reordered flag, changed quote or escape "
        "form, appended pipeline, or\nsemantically equivalent helper is a "
        "different command, and so is anything\nthat starts with no listed "
        "prefix."
        if families
        else "Every Bash tool call must copy one authorized command line "
        "byte-for-byte. A\nnarrower `sed` range, reordered flag, changed quote "
        "or escape form, appended\npipeline, or semantically equivalent helper "
        "is a different command."
    )
    return f"""{phase} {identity}
Repository root: {profile['root']}
GitHub repo: {profile['repo']}
AGY project: {agy_project_id(profile)}
Task kind: {profile['task_contract']['kind']}
Session policy: {policy}
Persistent project-policy digest: {permission_state_digest(profile)}

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

{family_block}Shell command lines explicitly forbidden for this task:
{chr(10).join(f"- {command}" for command in denied) or "- none"}

Do not change branches, create worktrees, commit, push, mutate a tracker, or
write any repository path outside the exact allowlist. Read-only git commands
are permitted only when explicitly listed above. Use absolute paths. If a
command is unavailable, a path is missing, or the task conflicts with the
injected contract, stop and report FAIL instead of improvising.

{copy_rule} Use the
built-in read-file tool for additional source inspection; do not synthesize a
new shell command.

## Controller oracle (injected, immutable)

{controller_notes_removed(oracle_text)}

## Round-specific controller injection

{injection or "(none)"}

{report_contract}
"""


VERDICT_REPORT_CONTRACT = """\
Your terminal answer must end with one final line-anchored `## EXEC REPORT`
block containing PASS or FAIL per criterion, exact changed paths, runnable
selectors, witnesses, and unfinished steps. Progress chatter may precede that
block; the dispatcher normalizes the last report marker. Do not claim a command
you were denied or did not run. A PASS is provisional until controller
verification. Any tracker comment is optional unverified input; the local report
is mandatory. The controller independently verifies every claim and alone
decides acceptance, publication, and any closure."""


NO_SHELL_REPORT_CONTRACT = """\
Your terminal answer must end with one final line-anchored `## EXEC REPORT`
block. Progress chatter may precede it; the dispatcher normalizes the last
report marker. This round grants you NO shell: the ticket command allowlist and
the project allow policy are both empty, so you cannot build, test, run, or
observe anything. Therefore:

- Do not write `PASS`, `FAIL`, or any per-criterion verdict. You have no
  evidence for one and the controller will read it as fabrication.
- Do not state a test result, a test count, a build outcome, or any other
  observation of program behavior.
- Report only what you wrote: exact changed paths, and for each one the exact
  symbols, signatures, or lines you added or changed, quoted from what you
  actually wrote.
- If the round-specific injection above prescribes its own `## EXEC REPORT`
  question list, answer exactly those questions in that order and ignore this
  paragraph's default shape.
- Name any injected requirement you could not carry out, and why.

The controller independently verifies every claim from the repository bytes and
alone decides acceptance, publication, and any closure."""


def conversation_id_from_log(path: Path) -> str | None:
    if not path.exists():
        return None
    matches = re.findall(
        r"(?:Created conversation\s+|conversation=)([0-9a-fA-F-]{36})",
        path.read_text(errors="replace"),
    )
    return matches[-1] if matches else None


def extract_exec_report(raw: str) -> str | None:
    stripped = raw.lstrip()
    markers = list(re.finditer(r"^## EXEC REPORT", stripped, flags=re.MULTILINE))
    if not markers:
        return None
    return stripped[markers[-1].start() :].lstrip()


def extract_run_command_lines(payload: bytes) -> list[str]:
    text = payload.decode(errors="replace")
    values = re.findall(
        r'\{"CommandLine":(?P<value>"(?:\\.|[^"\\])*")',
        text,
    )
    return [json.loads(value) for value in values]


def requested_run_commands(
    conversation_id: str,
    *,
    after_step: int = -1,
) -> list[dict]:
    """Every shell command AGY asked for after the floor, with its outcome.

    A request (`step_type` 15) and what became of it (`step_type` 21) are two
    rows. The request row's status is 3 for all of them -- it records that a
    request was made, not what happened next -- so reading it alone cannot tell
    a command that ran from one the permission layer refused (#3427). The
    outcome row is the next one, and status 7 there means denied.

    `denied` is asserted only on that positive evidence. An outcome row that is
    missing, because the process died mid-command, reads as *ran*: the caller
    treats an unaudited execution as fatal, and the conservative direction for
    an unknown is the fatal one.
    """
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(
            f"conversation state is missing for {conversation_id}: {database}"
        )
    connection = connect_conversation(database)
    try:
        rows = connection.execute(
            "select idx, status, step_type, step_payload from steps "
            "where idx > ? order by idx",
            (after_step,),
        ).fetchall()
    finally:
        connection.close()
    outcomes = {}
    for position, (idx, status, step_type, _payload) in enumerate(rows):
        if int(step_type) != RUN_COMMAND_REQUEST_STEP:
            continue
        following = rows[position + 1] if position + 1 < len(rows) else None
        if following and int(following[2]) == RUN_COMMAND_OUTCOME_STEP:
            outcomes[int(idx)] = int(following[1])
    return [
        {
            "step": int(idx),
            "status": int(status),
            "command": command,
            "denied": outcomes.get(int(idx)) == RUN_COMMAND_DENIED_STATUS,
        }
        for idx, status, step_type, payload in rows
        if int(step_type) == RUN_COMMAND_REQUEST_STEP
        for command in extract_run_command_lines(payload or b"")
    ]


def task_allowlist_families(profile: dict) -> list[str]:
    return profile.get("task_commands", {}).get("allow_prefix", [])


def task_allowlist_admits(profile: dict, command: str) -> bool:
    """Does the round's own allowlist name this command line?

    Exact entries are compared as strings, because that is what the controller
    meant by writing a whole command line. Prefix entries are compared with
    `command_rule_matches`, the same function that decides whether the command
    is permitted to run at all -- so a family entry admits here exactly what it
    admits there, and the two layers cannot drift apart.
    """
    if command in profile.get("task_commands", {}).get("allow", []):
        return True
    return any(
        command_rule_matches(f"command({entry})", command)
        for entry in task_allowlist_families(profile)
    )


def audit_task_commands(
    profile: dict,
    task_key: str,
    snapshot_data: dict,
) -> tuple[list[dict], list[str], list[str]]:
    """The commands that ran, the ones worth adjudicating, and the ones to note.

    Findings block the round at `review`; notes do not. The split is by which
    permission surface admitted a command, not by whether the controller wrote
    it down: an unlisted command the round's own `project_permissions` grant is
    a vocabulary gap in the profile, while one only an inherited global rule
    allowed means the worker had a surface the round never declared.
    """
    current_id = conversation_id_for_task(profile, task_key)
    snapshot_id = snapshot_data.get("conversation_id")
    if snapshot_id and current_id != snapshot_id:
        raise SystemExit(
            "VOID: task conversation changed after snapshot: "
            f"{snapshot_id} -> {current_id or '<missing>'}"
        )
    if not current_id:
        return [], [], []
    commands = requested_run_commands(
        current_id,
        after_step=int(snapshot_data.get("conversation_step_floor", -1)),
    )
    denied_commands = set(profile["task_commands"]["deny"])
    forbidden = [item for item in commands if item["command"] in denied_commands]
    unlisted = [
        item
        for item in commands
        if not task_allowlist_admits(profile, item["command"])
        and item["command"] not in denied_commands
    ]
    executed = [item for item in unlisted if not item["denied"]]
    refused = [item for item in unlisted if item["denied"]]

    # A command that ran was, by definition, permitted by something. Judging it
    # solely against the exact allowlist made VOID report the controller's
    # vocabulary rather than the worker's conduct: an entry the permission layer
    # reads as a prefix admits argument forms no exact entry can equal, so the
    # command executes and the whole round dies at the end for it. Ask instead
    # which surface admitted it, because that is the question integrity turns on.
    declared = expected_project_surface(profile)
    empty = {kind: [] for kind in PERMISSION_KINDS}
    global_surface = global_permission_surface()
    under_declared, escaped, unauthorized = [], [], []
    for item in executed:
        if permission_decision(declared, empty, item["command"])[0] == "allow":
            under_declared.append(item)
        elif permission_decision(empty, global_surface, item["command"])[0] == "allow":
            escaped.append(item)
        else:
            unauthorized.append(item)

    if forbidden or unauthorized:
        details = {
            "forbidden": forbidden,
            "unauthorized": unauthorized,
        }
        raise SystemExit(
            "VOID: AGY ran shell commands nothing authorized: "
            + json.dumps(details, sort_keys=True)
        )

    findings = [
        f"AGY asked for `{item['command']}` at step {item['step']}, which the "
        "task allowlist does not name; the permission layer refused it and "
        "nothing ran. Tighten the prompt, or add the command to the profile "
        "and re-snapshot, before dispatching again."
        for item in refused
    ] + [
        f"AGY ran `{item['command']}` at step {item['step']}, which neither "
        "`task_commands` nor this round's `project_permissions` names. Only a "
        "permission rule inherited from outside the round let it through, so "
        "the round's declared surface is not the surface the worker had. "
        "Adjudicate what it observed before accepting."
        for item in escaped
    ]
    notes = [
        f"AGY ran `{item['command']}` at step {item['step']}: this round's "
        "`project_permissions` authorize it, but `task_commands` does not name "
        "it. Add it to `task_commands.allow` if the exact line matters, or to "
        "`task_commands.allow_prefix` if any argument form is intended."
        for item in under_declared
    ]
    audited = [
        item for item in commands if task_allowlist_admits(profile, item["command"])
    ]
    return audited, findings, notes


def denied(profile: dict, task_key: str) -> None:
    validate_task_key(profile, task_key)
    log_dir = Path(profile["state_dir"]) / "runs"
    conversation_path = log_dir / f"{task_key}.conversation"
    conversation_id = (
        conversation_path.read_text().strip()
        if conversation_path.exists()
        else conversation_id_from_log(log_dir / f"{task_key}.agy.log")
    )
    if not conversation_id:
        raise SystemExit(
            f"cannot inspect {task_key}: no conversation id in state"
        )
    database = conversation_database(conversation_id)
    if not database.is_file():
        raise SystemExit(f"cannot inspect {task_key}: missing {database}")
    connection = connect_conversation(database)
    try:
        rows = connection.execute(
            "select idx, step_payload from steps "
            "where status = ? order by idx",
            (RUN_COMMAND_DENIED_STATUS,),
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


def load_snapshot(profile: dict, task_key: str) -> dict:
    path = Path(profile["state_dir"]) / "snapshots" / f"{task_key}.json"
    if not path.is_file():
        raise SystemExit("missing pre-dispatch snapshot")
    return json.loads(path.read_text())


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
    return conversation_id


ORACLE_SECTIONS = ("Claim", "Measurements", "Gate", "Fabrication tells")
INJECTION_SECTIONS = (
    "Task",
    "Current behavior",
    "Required change",
    "Shape to follow",
    "Reference",
    "Out of scope",
    "Definition of done",
)
# Only the section quoting what already exists, and the one naming the gate, may
# carry a fenced block.  Anywhere else a fence means the controller pasted the
# answer, and a round whose answer is already written has nothing left to
# dispatch.
FENCE_BEARING_SECTIONS = ("Current behavior", "Definition of done")
NUMBERED_STEP = re.compile(r"^[ \t]*\d+[.)][ \t]+\S", re.MULTILINE)
# Enough to name a convention and say to follow it; not enough to describe one.
SHAPE_LINE_BUDGET = 4
ORACLE_HEADING = re.compile(r"^##[ \t]+(.+?)[ \t]*$", re.MULTILINE)
ORACLE_FENCE = re.compile(r"^```[^\n]*\n(.*?)^```", re.MULTILINE | re.DOTALL)
INFO_FENCE = re.compile(r"^```([^\n]*)\n(.*?)^```", re.MULTILINE | re.DOTALL)
# Current behavior is sometimes a thing the binary *does*, not a thing a file
# says, and the two are grounded differently: a source quote is checked against
# the checkout, a transcript against the command that produced it.
TRANSCRIPT_INFO = re.compile(r"^console$", re.IGNORECASE)
TRANSCRIPT_COMMAND = re.compile(r"^\$[ \t]+\S")
NEGATIVE_CONTROL = re.compile(r"negative control", re.IGNORECASE)
LIST_ITEM = re.compile(r"^[ \t]*(?:[-*+]|\d+[.)])[ \t]+\S", re.MULTILINE)
TABLE_ROW = re.compile(r"^[ \t]*\|.*\|[ \t]*$", re.MULTILINE)
TABLE_DIVIDER = re.compile(r"^[ \t]*\|[\s:|-]+\|[ \t]*$")
FILL_MARKER = re.compile(r"<!--[ \t]*fill\b.*?-->", re.DOTALL)
BACKTICKED = re.compile(r"`([^`\n]+)`")
LINE_SUFFIX = re.compile(r":\d+(?:[:-]\d+)?$")
# A quoted excerpt may skip lines; the marker for that is not itself a quote.
ELISION = re.compile(r"(?://|#)?[ \t]*(?:\.{3}|…|snip|omitted)[ \t]*", re.IGNORECASE)


def oracle_sections(text: str) -> dict[str, str]:
    """Split an oracle into its `## ` sections, preserving document order."""
    matches = list(ORACLE_HEADING.finditer(text))
    sections: dict[str, str] = {}
    for index, match in enumerate(matches):
        end = matches[index + 1].start() if index + 1 < len(matches) else len(text)
        sections[match.group(1).strip()] = text[match.end() : end].strip()
    return sections


def missing_or_misordered(text: str, required: tuple[str, ...], label: str) -> list[str]:
    """Which required `## ` sections are absent, and are the rest in order."""
    sections = oracle_sections(text)
    findings = [
        f"{label} is missing the `## {name}` section"
        for name in required
        if name not in sections
    ]
    present = [name for name in sections if name in required]
    if present != [name for name in required if name in sections]:
        findings.append(
            f"{label} sections are out of order; expected "
            + " -> ".join(f"## {name}" for name in required)
        )
    return findings


def gate_commands_in(section: str) -> list[str]:
    """The command lines inside a section's fenced blocks, in order.

    A block that prompts with `$ ` is a transcript: its commands are the
    prompted lines and everything else is output. Reading output as a command
    is how `## Definition of done` came to "name a different gate" than the
    oracle while naming the same one -- once as `$ cargo test ...` against the
    oracle's bare `cargo test ...`, and again with the expected `test result:`
    line counted as a second gate. Showing the controller what green looks like
    is worth keeping, so the prompt is what marks the command.
    """
    commands = []
    for block in ORACLE_FENCE.findall(section):
        lines = [line.strip() for line in block.splitlines() if line.strip()]
        prompted = [line for line in lines if line.startswith("$ ")]
        commands.extend(
            [line[2:].strip() for line in prompted] if prompted else lines
        )
    return commands


def unjudged_gate_commands(profile: dict, gate_commands: list[str]) -> list[str]:
    """Which of an oracle's `## Gate` commands `prove` will never run.

    `prove` runs `task_contract.gate_command` and nothing else -- one command,
    whose red decides the round. An oracle's `## Gate` block is free to list
    several, and `lint` already checks each one is authorized, so a second
    command reads as judged when it is only authorized. The round then carries
    a row whose observation no proof ever makes, which is the false green this
    whole scaffold exists to refuse: the documents say two things decide the
    round and the machinery lets one decide it.

    The way out is to name the compound command in the profile, or to state the
    extra observation as prose the controller checks by hand and keep the fence
    to the one command that is actually judged.
    """
    judged = profile.get("task_contract", {}).get("gate_command")
    if not judged:
        return []
    return [command for command in gate_commands if command != judged]


def unquoted_current_behavior_lines(
    root: str | None, section: str, candidates: list[str]
) -> list[str]:
    """Lines an injection quotes as current behavior that no candidate file has.

    `injection_findings` already refuses a `## Current behavior` with no fenced
    quote, on the reasoning that a round should be grounded in what was read
    rather than what was remembered. That check reaches the form and stops: a
    block pasted from an earlier round, from a base two commits back, or from
    memory satisfies it exactly as well as a block copied out of the file.

    A stale quote is worse than no quote. It is the one part of the injection a
    worker is entitled to treat as ground truth -- it is labelled as the code as
    it stands -- so a worker that greps for it, finds nothing, and improvises has
    been sent to do that by the document. Rounds are re-based often here (a
    follow-up reuses the previous worktree block; documents get drafted while an
    earlier round is still in flight), which is exactly when a quote goes stale
    without anyone editing it.

    Matching is on the stripped line, so re-indenting a quote is not a finding:
    the dangerous class is content that is gone, not content that moved. Lines
    that are only an elision marker are skipped for the same reason.

    A ```console fence is skipped, because it is not claiming to be in a file.
    Current behavior is often what the binary *prints* -- an envelope, a refusal,
    an exit code -- and there is no file to find those lines in. Checking them
    against the source anyway reported every real transcript as stale, whose only
    available fix was to strip the fence and paraphrase the output; that trades a
    verbatim observation for prose, which is the failure this whole rule exists
    to prevent. `transcript_findings` grounds these instead, by requiring the
    command that produced them.
    """
    if not root:
        return []
    haystacks: list[set[str]] = []
    for rel in candidates:
        path = Path(rel) if Path(rel).is_absolute() else Path(root) / rel
        try:
            haystacks.append({line.strip() for line in path.read_text().splitlines()})
        except (OSError, UnicodeDecodeError):
            continue
    if not haystacks:
        return []
    missing: list[str] = []
    for info, block in INFO_FENCE.findall(section):
        if TRANSCRIPT_INFO.match(info.strip()):
            continue
        for line in block.splitlines():
            bare = line.strip()
            if not bare or ELISION.fullmatch(bare):
                continue
            if any(bare in hay for hay in haystacks):
                continue
            if bare not in missing:
                missing.append(bare)
    return missing


def referenced_paths(text: str) -> list[str]:
    """Backticked tokens that name a repository path.

    A token qualifies only when it carries a separator, which keeps Rust module
    paths, flag names, field accesses, and prose identifiers out of the check.
    An optional `:line` or `:line-line` suffix is dropped so the usual
    `path:line` citation form resolves.
    """
    tokens: list[str] = []
    for token in BACKTICKED.findall(text):
        if " " in token or "/" not in token:
            continue
        if token.startswith(("http://", "https://", "-")) or "*" in token:
            continue
        bare = LINE_SUFFIX.sub("", token)
        if bare and bare not in tokens:
            tokens.append(bare)
    return tokens


def transcript_body(block: str) -> list[str]:
    """A console block's output lines, normalized for comparison.

    Blank lines and trailing spaces survive a copy-paste unevenly, and neither
    carries any of the observation, so neither may decide whether a transcript
    matches the run it claims to be.
    """
    return [line.rstrip() for line in block.splitlines()[1:] if line.strip()]


def transcript_findings(section: str, captures: dict[str, list[str]]) -> list[str]:
    """Console blocks that are not the output of a command anyone ran.

    Exempting a ```console fence from the file comparison removes the only check
    that block had, so it has to gain one of its own. The prompt line was that
    check, and it turned out to be an assertion rather than a measurement:
    nothing compared the lines under it to anything, so a paraphrase typed from
    memory and a capture taken a second ago read identically. Two shipped in one
    round -- one naming a flag the verb does not accept, one whose behavior only
    existed in a build newer than the installed binary -- and lint reported the
    round clean (#3426).

    So the block is now compared against a stored capture. `capture` runs the
    command and writes what it printed; this checks the pasted block against
    that. The honest path stays the cheapest one, because pasting what `capture`
    prints is less work than typing output, and fabricating now means forging a
    record that names its own command, directory, and exit code.
    """
    findings: list[str] = []
    unprompted = False
    uncaptured: list[str] = []
    diverged: list[str] = []
    for info, block in INFO_FENCE.findall(section):
        if not TRANSCRIPT_INFO.match(info.strip()):
            continue
        lines = [line for line in block.splitlines() if line.strip()]
        if not lines or not TRANSCRIPT_COMMAND.match(lines[0].strip()):
            unprompted = True
            continue
        command = lines[0].strip()[2:].strip()
        if command not in captures:
            uncaptured.append(command)
        elif transcript_body(block) != captures[command]:
            diverged.append(command)
    if unprompted:
        findings.append(
            "a ```console block in `## Current behavior` does not open with "
            "the `$ <command>` that produced it: a transcript is exempt from "
            "the file comparison, so the command is the only thing that lets "
            "a reader reproduce it rather than trust it"
        )
    if uncaptured:
        findings.append(
            "`## Current behavior` shows output for command(s) this round never "
            "captured: "
            + ", ".join(f"`{command}`" for command in uncaptured)
            + ". Run `capture <profile> <task_key> <command>` and paste what it "
            "prints; output typed from memory is what the worker will treat as "
            "the behavior as it stands"
        )
    if diverged:
        findings.append(
            "`## Current behavior` shows output that differs from what the "
            "captured run of "
            + ", ".join(f"`{command}`" for command in diverged)
            + " printed. Re-capture and paste the result rather than editing "
            "the block, or the document says one thing and the record another"
        )
    return findings


def document_findings(
    root: str | None,
    text: str,
    label: str,
    declared: set[str] | None = None,
) -> list[str]:
    """Checks that apply to any controller-authored round document.

    Both defects here are of one kind: the document says something the
    controller never actually established. An unfilled slot is a form dispatched
    before it was written; a path that does not resolve is a citation from
    memory rather than from the checkout the worker is about to see.

    A path the round declares writable is exempt: a round may create a file, and
    naming the file it is about to create is the opposite of citing from memory.
    """
    findings: list[str] = []
    if FILL_MARKER.search(text):
        findings.append(
            f"{label} still carries {len(FILL_MARKER.findall(text))} unfilled "
            "`<!-- fill -->` slot(s) from the scaffold"
        )
    if root:
        declared = declared or set()
        missing = [
            token
            for token in referenced_paths(text)
            if token not in declared
            and not (Path(token) if Path(token).is_absolute() else Path(root) / token).exists()
        ]
        if missing:
            findings.append(
                f"{label} cites path(s) that do not exist in the worker's "
                "checkout: " + ", ".join(missing)
            )
    return findings


def marks_a_negative_control(row: str) -> bool:
    """Whether this table row is itself the control, not prose about one.

    A row is a negative control because of what it *feeds* and what that must
    not produce, so the marker belongs in its input or its expected
    observation. It does not belong in a trailing rationale cell, which is
    where a controller naturally writes about a *different* row: "row 7 is the
    negative control for the new row" is a true sentence that leaves the table
    without a control of its own. Matching the row as one string accepted that
    sentence and reported the table conformant.

    A rationale cell is one past `# | input | expected observation`, so it is
    dropped only when the row actually has one. In the three-column table the
    last cell is the observation and stays in scope.
    """
    cells = [cell.strip() for cell in row.strip().strip("|").split("|")]
    identity = cells[:-1] if len(cells) > 3 else cells
    return any(NEGATIVE_CONTROL.search(cell) for cell in identity)


def oracle_findings(profile: dict, text: str) -> list[str]:
    """Structural check on the injected oracle. Never reads for meaning.

    The generic scaffold around the oracle has always had a fixed shape while
    the oracle itself was free prose, and every false green so far entered
    through that prose: a gate nobody cross-checked against the authorized
    commands, a table with no control row, no statement of what fabrication
    would look like. These four sections are expressible for any bounded task,
    so requiring them costs no project knowledge and closes that gap.
    """
    sections = oracle_sections(text)
    findings = missing_or_misordered(text, ORACLE_SECTIONS, "oracle")
    findings.extend(
        document_findings(
            profile.get("root"), text, "oracle", set(profile.get("allowed_repo_writes") or [])
        )
    )

    # A missing section is one defect. Reporting its emptiness and its missing
    # rows as further defects inflates the count and buries the real fix, so
    # each section's content checks run only once the section exists.
    if "Claim" in sections and not sections["Claim"]:
        findings.append("`## Claim` is empty: state one falsifiable sentence")

    if "Measurements" in sections:
        rows = [
            row
            for row in TABLE_ROW.findall(sections["Measurements"])
            if not TABLE_DIVIDER.match(row)
        ]
        data_rows = rows[1:] if rows else []
        if len(data_rows) < 2:
            findings.append(
                f"`## Measurements` needs at least 2 measured rows, found "
                f"{len(data_rows)}"
            )
        elif not any(marks_a_negative_control(row) for row in data_rows):
            findings.append(
                "`## Measurements` has no row marked `negative control`: "
                "without one, an implementation that changes nothing can "
                "satisfy the table. Mark the control in its input or its "
                "expected observation -- naming one in a rationale cell "
                "describes a control, it does not add one"
            )

    if "Gate" in sections:
        gate_commands = gate_commands_in(sections["Gate"])
        allowed = profile["task_commands"].get("allow", [])
        families = profile["task_commands"].get("allow_prefix", [])
        if not gate_commands:
            findings.append(
                "`## Gate` has no fenced command block: put each gate command "
                "on its own line inside one ``` fence"
            )
        else:
            if allowed or families:
                undeclared = [
                    c for c in gate_commands if not task_allowlist_admits(profile, c)
                ]
                if undeclared:
                    findings.append(
                        "`## Gate` names command(s) the worker is not "
                        "authorized to run: " + ", ".join(undeclared)
                    )
            unjudged = unjudged_gate_commands(profile, gate_commands)
            if unjudged:
                findings.append(
                    "`## Gate` names command(s) `prove` will never run, so no "
                    "proof covers what they observe: "
                    + ", ".join(unjudged)
                    + ". `prove` runs `task_contract.gate_command` alone "
                    f"(`{profile.get('task_contract', {}).get('gate_command')}`). "
                    "Name the compound command in the profile, or keep the "
                    "fence to the judged command and state the rest as prose "
                    "the controller checks by hand."
                )

    if "Fabrication tells" in sections and not LIST_ITEM.search(
        sections["Fabrication tells"]
    ):
        findings.append(
            "`## Fabrication tells` needs at least one list item naming what a "
            "fabricated pass would look like"
        )
    return findings


def injection_findings(
    profile: dict,
    text: str,
    oracle_text: str,
    captures: dict[str, list[str]] | None = None,
) -> list[str]:
    """Structural check on the round-specific injection.

    The oracle says how the round will be judged; this document says what to do
    and what to read, and it is the half a controller is most tempted to write
    from memory. Requiring a verbatim quote of the current behavior means the
    round cannot be dispatched without the controller having opened the file,
    and requiring the same gate as the oracle means the instruction and the
    judgement cannot drift apart the way they did when each was free prose.

    The remaining rules all defend the same thing: a dispatched round is only
    worth its cost if the worker still has the design left to do.  A pasted
    implementation, a numbered recipe, or a `## Shape to follow` that grew into
    a plan all mean the controller already did the work and is paying a second
    time to have it typed out.
    """
    sections = oracle_sections(text)
    findings = missing_or_misordered(text, INJECTION_SECTIONS, "injection")
    findings.extend(
        document_findings(
            profile.get("root"), text, "injection", set(profile.get("allowed_repo_writes") or [])
        )
    )

    for name in ("Task", "Required change"):
        if name in sections and not sections[name]:
            findings.append(f"`## {name}` is empty")

    for name, body in sections.items():
        if name in FENCE_BEARING_SECTIONS or name not in INJECTION_SECTIONS:
            continue
        if ORACLE_FENCE.search(body):
            findings.append(
                f"`## {name}` contains a fenced block: quote existing code only "
                "under `## Current behavior`. State the requirement; handing the "
                "worker the implementation leaves it nothing to design"
            )

    for name in ("Required change", "Shape to follow"):
        if name in sections and NUMBERED_STEP.search(sections[name]):
            findings.append(
                f"`## {name}` reads as numbered steps: say what must become "
                "true, not the order to type it in"
            )

    if "Current behavior" in sections:
        # A transcript does not satisfy the quote rule. It is grounded by its
        # command instead of by the checkout, so a section holding only
        # transcripts has never been checked against the tree the worker will
        # open -- which is the one thing this rule was added to force.
        quotes = [
            body
            for info, body in INFO_FENCE.findall(sections["Current behavior"])
            if body.strip() and not TRANSCRIPT_INFO.match(info.strip())
        ]
        findings.extend(
            transcript_findings(sections["Current behavior"], captures or {})
        )
        if not quotes:
            findings.append(
                "`## Current behavior` has no non-empty fenced quote: paste the "
                "code as it stands today so the round is grounded in what was "
                "read rather than what was remembered"
            )
        else:
            candidates = list(profile.get("allowed_repo_writes") or [])
            candidates += [p for p in referenced_paths(text) if p not in candidates]
            stale = unquoted_current_behavior_lines(
                profile.get("root"), sections["Current behavior"], candidates
            )
            if stale:
                shown = ", ".join(f"`{line}`" for line in stale[:3])
                more = f" (+{len(stale) - 3} more)" if len(stale) > 3 else ""
                findings.append(
                    "`## Current behavior` quotes line(s) that appear in none of "
                    f"the round's files: {shown}{more}. The worker is entitled to "
                    "treat that block as the code as it stands; re-read the file "
                    "at this round's base and paste what is actually there"
                )

    if "Shape to follow" in sections:
        body = sections["Shape to follow"]
        lines = [line for line in body.splitlines() if line.strip()]
        if not BACKTICKED.search(body):
            findings.append(
                "`## Shape to follow` names no existing symbol or file: point at "
                "the convention already in the tree that the change must match, "
                "so the worker does not invent a second one"
            )
        if len(lines) > SHAPE_LINE_BUDGET:
            findings.append(
                f"`## Shape to follow` is {len(lines)} lines; keep it within "
                f"{SHAPE_LINE_BUDGET}. Past that it stops being a constraint and "
                "becomes the design the worker was dispatched to produce"
            )

    if "Reference" in sections:
        rows = [
            row
            for row in TABLE_ROW.findall(sections["Reference"])
            if not TABLE_DIVIDER.match(row)
        ]
        if not LIST_ITEM.search(sections["Reference"]) and len(rows[1:]) < 1:
            findings.append(
                "`## Reference` names nothing to read: list each path the "
                "worker must consult and why"
            )

    if "Out of scope" in sections and not LIST_ITEM.search(sections["Out of scope"]):
        findings.append(
            "`## Out of scope` needs at least one list item; the write "
            "allowlist bounds where the worker may write, not what it may "
            "redesign"
        )

    if "Definition of done" in sections:
        declared = gate_commands_in(sections["Definition of done"])
        judged = gate_commands_in(oracle_sections(oracle_text).get("Gate", ""))
        if not declared:
            findings.append(
                "`## Definition of done` has no fenced command block: name the "
                "gate the worker must leave green"
            )
        elif judged and declared != judged:
            findings.append(
                "`## Definition of done` names a different gate than the "
                f"oracle judges by: {declared} vs {judged}"
            )
        prose = ORACLE_FENCE.sub("", sections["Definition of done"])
        if not BACKTICKED.search(prose):
            findings.append(
                "`## Definition of done` names the gate but not where its check "
                "lands: name the module, file, or suite the new check joins, or "
                "the worker guesses and the diff arrives in the wrong place"
            )
    return findings


def injection_path(profile: dict, task_key: str) -> Path:
    """Where this round's injection lives.

    A profile may point `inject_prompt_file` anywhere; when it does not, the
    scaffold has a deterministic home beside the oracle so the two halves of a
    round stay together.
    """
    declared = profile.get("inject_prompt_file")
    if declared:
        return Path(declared)
    return Path(profile["state_dir"]) / "injections" / f"{task_key}.md"


def oracle_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "oracles" / f"{task_key}.md"


ORACLE_SKELETON = """\
## Claim

<!-- fill: one falsifiable sentence about behavior observable from outside the
     change; not a description of the edit. Revert the change in your head: a
     claim that stays true either way is not this round's claim. -->

## Measurements

<!-- fill: the rows the gate has to make. At least two, at least one of them
     the negative control, and the control marked in its input or its expected
     observation -- a control named only in the rationale cell is a sentence
     about a control, and the table passes lint while measuring nothing.

     Every row must be a state the product can actually reach. A row resting on
     a value the product never produces is vacuous: it stays green whatever the
     worker writes, and that is how this round most plausibly ends green and
     empty. Where a row rests on something you measured, measure it against the
     base this round starts from -- a stale "measured" is worse than silence,
     because the worker builds on it. -->

| # | input | expected observation | why it cannot hold by accident |
|---|---|---|---|
| 1 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 2 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 3 | <!-- fill --> (negative control) | <!-- fill: must FAIL --> | <!-- fill --> |

## Gate

<!-- fill: prefilled from the profile. `prove` runs this one command and
     nothing else, so a second command here creates a row no proof ever
     makes. -->

```
{gate}
```

## Fabrication tells

<!-- fill: what a passing report would look like if the worker faked it. Not
     the worker lying -- the shapes you would otherwise accept. A gate green
     because its rows are unreachable. An assertion on a value the check itself
     just wrote. A name borrowed from a vocabulary the code under test never
     reads. One list item each. -->
-
"""

INJECTION_SKELETON = """\
## Task

<!-- fill: one imperative sentence naming the change. The worker reads this
     first and reads it as the whole job, so a sentence naming two things buys
     a diff that does one of them. -->

## Current behavior

<!-- fill: quote the code as it stands, citing the file and line in backticks.
     Lint checks that every quoted line still exists, because the worker is
     told this block is the code as it stands and will go looking for it: a
     quote that was true at an earlier base sends the worker off to improvise.
     Re-indenting is fine, content that has moved is not. Quote what the change
     must displace, not the whole neighbourhood. -->

```
```

## Required change

<!-- fill: what becomes true afterwards, as conditions someone outside the
     change could check. No code and no numbered steps: the worker is being
     paid to derive the implementation, so writing it here buys nothing and
     costs twice. A condition you can only state by naming the lines that
     satisfy it is a measurement -- it belongs in the oracle. -->
-

## Shape to follow

<!-- fill: at most {shape_budget} lines. Name the convention already in the
     tree that this change must match -- an existing function, module, type, or
     error shape, in backticks -- and say to follow it rather than invent a
     second one. Where two conventions in the tree could both apply, saying
     which one wins is exactly this slot's job. A constraint on the answer, not
     the answer. -->

## Reference

<!-- fill: one row per file, and the reason must say what the worker will learn
     there. "Relevant context" is not a reason and gets the file skimmed. -->

| path | why the worker must read it |
|---|---|
{reference}

## Out of scope

<!-- fill: what must not be touched, beyond what the write allowlist already
     blocks mechanically. The allowlist bounds where the worker may write; this
     bounds what it may redesign, rename, or clean up on the way past. Do not
     restate the allowlists, the report shape, or the stop-and-report rule --
     the dispatcher already sends those, and a second copy is one that
     drifts. -->
-

## Definition of done

<!-- fill: name in backticks where the gate's check lands -- the module, file,
     or suite it joins. The gate says what to run; without this the worker
     guesses and a correct diff arrives in the wrong place. The fence must
     match the oracle's `## Gate` exactly. -->

```
{gate}
```
"""


def scaffold(profile: dict, task_key: str) -> None:
    """Write the blank round form for the controller to fill.

    The structural contract used to be enforced only after the fact, which put
    the controller in the position of authoring from memory and learning what
    was required from a rejection. Handing out the slots first makes the same
    contract constructive: the form states what a round must say, and the
    remaining `<!-- fill -->` markers are themselves a finding, so a form that
    was never filled cannot be dispatched.
    """
    gate = profile["task_contract"].get("gate_command") or (
        "<!-- fill: the exact command that must be green -->"
    )
    design_inputs = profile["task_contract"].get("design_inputs", [])
    reference = "\n".join(
        f"| `{entry['path']}` | <!-- fill --> |" for entry in design_inputs
    ) or "| `<!-- fill: path -->` | <!-- fill --> |"

    written, kept = [], []
    for path, body in (
        (oracle_path(profile, task_key), ORACLE_SKELETON.format(gate=gate)),
        (
            injection_path(profile, task_key),
            INJECTION_SKELETON.format(
                gate=gate, reference=reference, shape_budget=SHAPE_LINE_BUDGET
            ),
        ),
    ):
        if path.exists():
            kept.append(path)
            continue
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(body)
        written.append(path)

    for path in written:
        print(f"wrote  {path}")
    for path in kept:
        print(f"kept   {path} (already authored; scaffold never overwrites)")
    if not profile.get("inject_prompt_file"):
        print(
            "\nnote: this profile declares no `inject_prompt_file`, so the "
            "injection above is not read at dispatch and not checked. Point "
            "`inject_prompt_file` at it to make it part of the round."
        )
    print("\nfill both files, then run `lint` before `dispatch`.")


def round_findings(profile: dict, task_key: str) -> list[str]:
    """Every structural finding across both halves of the round document."""
    oracle = oracle_path(profile, task_key)
    if not oracle.exists():
        raise SystemExit(f"no oracle at {oracle}")
    oracle_text = oracle.read_text()
    findings = oracle_findings(profile, oracle_text)
    # An injection is optional: a measure-only round can carry its whole
    # instruction in the oracle. Declaring one and leaving it unstructured is
    # not, because that is the half where the last false green entered.
    if profile.get("inject_prompt_file"):
        injection = injection_path(profile, task_key)
        if not injection.exists():
            findings.append(f"declared injection is missing at {injection}")
        else:
            findings.extend(
                injection_findings(
                    profile,
                    injection.read_text(),
                    oracle_text,
                    load_captures(profile, task_key),
                )
            )
    return findings


def lint(profile: dict, task_key: str) -> None:
    """Report the round document's structural findings without dispatching."""
    oracle = oracle_path(profile, task_key)
    findings = round_findings(profile, task_key)
    allowed = profile["task_commands"].get("allow", [])
    families = profile["task_commands"].get("allow_prefix", [])
    print(f"oracle   : {oracle}")
    print(f"sections : {', '.join(oracle_sections(oracle.read_text())) or 'none'}")
    injection = profile.get("inject_prompt_file")
    print(f"injection: {injection or 'none declared; oracle carries the round'}")
    if injection and Path(injection).exists():
        print(
            "sections : "
            + (", ".join(oracle_sections(Path(injection).read_text())) or "none")
        )
    print(
        "gate cross-check: "
        + (
            f"against {len(allowed)} exact command(s) and "
            f"{len(families)} prefix famil{'y' if len(families) == 1 else 'ies'}"
            if allowed or families
            else "skipped; this round grants the worker no shell"
        )
    )
    if not findings:
        print("\nfindings: none")
        return
    print(f"\nfindings ({len(findings)}):")
    for item in findings:
        print(f"  - {item}")
    sys.exit(EXIT_FINDINGS)


def run_agent(profile: dict, task_key: str, *, resume: bool) -> None:
    require_project_ready(profile)
    task_state = frozen_task_state(profile, task_key)
    state = Path(profile["state_dir"])
    oracle = state / "oracles" / f"{task_key}.md"
    snapshot_data = load_snapshot(profile, task_key)
    assert_snapshot_identity(profile, task_key, snapshot_data)
    assert_permission_state_unchanged(profile, snapshot_data)
    assert_round_documents_unchanged(profile, task_key, snapshot_data)
    # Called for its VOID: a refused request is `verify`'s to report, not a
    # reason to refuse the dispatch that has not happened yet.
    audit_task_commands(profile, task_key, snapshot_data)
    conversation_id = validate_conversation_action(
        profile,
        task_key,
        resume=resume,
    )
    if not oracle.exists():
        raise SystemExit(
            "refusing dispatch: create the oracle first (`scaffold` writes it)"
        )
    problems = round_findings(profile, task_key)
    if problems:
        raise SystemExit(
            "refusing dispatch: the round document does not satisfy the "
            "injection contract (run `lint`):\n"
            + "\n".join(f"  - {item}" for item in problems)
        )
    log_dir = state / "runs"
    log_dir.mkdir(parents=True, exist_ok=True)
    suffix = ""
    if resume:
        attempt = 1
        while True:
            suffix = ".resume" if attempt == 1 else f".resume.{attempt}"
            if not (log_dir / f"{task_key}{suffix}.log").exists():
                break
            attempt += 1
    prompt = render_prompt(
        profile,
        task_key,
        oracle.read_text(),
        task_state,
        continuation=resume,
    )
    prompt_path = log_dir / f"{task_key}{suffix}.prompt.md"
    prompt_path.write_text(prompt)
    conversation_path = log_dir / f"{task_key}.conversation"
    command = ["agy", "--project", agy_project_id(profile)]
    if resume:
        assert conversation_id is not None
        conversation_path.write_text(conversation_id + "\n")
        command.extend(["--conversation", conversation_id])
    command.extend(
        [
            "-p",
            prompt,
            "--model",
            profile.get("model", "gemini-3.6-flash-high"),
            "--effort",
            "high",
            "--print-timeout",
            profile.get("timeout", "30m"),
            "--log-file",
            str(log_dir / f"{task_key}{suffix}.agy.log"),
        ]
    )
    report_path = log_dir / f"{task_key}{suffix}.log"
    with report_path.open("w") as log:
        completed = subprocess.run(
            command,
            cwd=profile["root"],
            stdout=log,
            stderr=subprocess.STDOUT,
            check=False,
        )
    if not resume:
        conversation_id = conversation_id_from_log(
            log_dir / f"{task_key}.agy.log"
        )
        if conversation_id:
            conversation_path.write_text(conversation_id + "\n")
        else:
            raise SystemExit(
                f"dispatch failed for {task_key}: AGY conversation id is "
                "missing, so command and session lineage cannot be audited"
            )
    report = report_path.read_text(errors="replace")
    normalized_report = extract_exec_report(report)
    print(
        f"prompt sha256={sha256(prompt_path)}; "
        f"oracle sha256={sha256(oracle)}; exit={completed.returncode}"
    )
    if completed.returncode != 0:
        verb = "resume" if resume else "dispatch"
        if timed_out(report, log_dir / f"{task_key}{suffix}.agy.log"):
            raise SystemExit(
                f"{verb} timed out for {task_key} at the profile's "
                f"{profile['timeout']}; this is not a denial. The worker was "
                "cut off mid-run, so its checkout may already hold complete "
                "work with no report: read the worktree diff and run the gate "
                "before deciding whether to raise `timeout` and redispatch"
            )
        raise SystemExit(
            f"{verb} failed for {task_key}: "
            f"AGY exited {completed.returncode}; inspect `denied`, verify the "
            "snapshot, and update the persistent Project policy only when the "
            "missing command is a reusable project capability"
        )
    if not report.strip():
        raise SystemExit(
            f"{'resume' if resume else 'dispatch'} failed for {task_key}: "
            "empty local report; inspect the AGY log and repository diff"
        )
    if normalized_report is None:
        raise SystemExit(
            f"{'resume' if resume else 'dispatch'} failed for {task_key}: "
            "local output has no valid terminal `## EXEC REPORT`"
        )
    normalized_path = log_dir / f"{task_key}{suffix}.report.md"
    normalized_path.write_text(normalized_report)
    print(
        f"reported {task_key}; run status, then independently verify before "
        "acceptance"
    )


TIMEOUT_MARKERS = (
    "timeout waiting for response",
    "Print mode: timed out after",
)


def timed_out(report: str, agy_log: Path) -> bool:
    """Whether AGY's non-zero exit was its own deadline rather than a refusal.

    Both failures exit non-zero and the caller cannot tell them apart, so the
    generic message sent the controller to `denied` for a round where nothing
    was ever denied -- and `denied` correctly reported no denial, which reads
    like the tooling is broken rather than like the diagnosis was wrong. The
    local report is authoritative and cheap; the AGY log is the fallback for a
    deadline that killed the process before it wrote one.
    """
    if any(marker in report for marker in TIMEOUT_MARKERS):
        return True
    if not agy_log.is_file():
        return False
    tail = agy_log.read_text(errors="replace")[-20000:]
    return any(marker in tail for marker in TIMEOUT_MARKERS)


def dispatch(profile: dict, task_key: str) -> None:
    run_agent(profile, task_key, resume=False)


def resume(profile: dict, task_key: str) -> None:
    run_agent(profile, task_key, resume=True)


def revise(
    profile: dict, raw_path: str, task_key: str, next_key: str, injection: str
) -> None:
    """Mint a new run id for a round that must go back, keeping its checkout.

    `resume` is for a ticketed round. A one-shot run id is spent the moment a
    conversation exists, and the refusal says only "create a new run id" -- so
    the obvious next move is to author a new round and run `worktree`. That
    throws the candidate away: the worker's work is uncommitted in the very
    worktree `worktree` would re-create from HEAD, and nothing warns first.

    A revision is two changes and no more: a fresh run id, and the delta
    contract that says what was wrong. Everything else is deliberately carried
    over -- root, worktree, policy, protected artifacts, budgets -- so the next
    round measures the same tree under the same rules. The oracle is copied
    unchanged, because a revision exists to satisfy the sealed claim rather than
    to move it; `lint` still grades the new pair against the checkout.

    The budget is carried too, and that is the point of carrying it: the
    revision's diff is judged against the same ceiling as the round it
    continues, not given a second one.
    """
    validate_task_key(profile, task_key)
    if next_key == task_key:
        raise SystemExit(
            "the revision needs its own run id: a one-shot id is spent once a "
            "conversation exists, so reusing it cannot dispatch"
        )
    spec = worktree_spec(profile)
    if str(Path(profile["root"])) != str(Path(spec["path"])):
        raise SystemExit(
            f"profile root {profile['root']} is not the worker checkout "
            f"{spec['path']}: there is no round in progress to revise. Author "
            "a fresh round and run `worktree` instead"
        )
    touched = worker_touched_paths(profile)
    if not touched:
        raise SystemExit(
            f"refusing to revise {task_key}: the worker changed nothing, so "
            "there is no candidate to carry forward and a fresh round costs "
            "nothing to author"
        )
    source = Path(injection)
    if not source.is_file():
        raise SystemExit(f"revision injection does not exist: {injection}")

    state = Path(profile["state_dir"])
    target = state / "rounds" / f"{next_key}.profile.json"
    if target.exists():
        raise SystemExit(
            f"refusing to overwrite an existing round: {target}. A round in "
            "flight owns its profile; choose a run id that is not taken"
        )
    oracle = oracle_path(profile, task_key)
    if not oracle.is_file():
        raise SystemExit(
            f"the round being revised has no oracle at {oracle}, so there is "
            "no sealed claim for the revision to inherit"
        )

    revised = json.loads(Path(raw_path).read_text())
    revised["task_contract"]["run_id"] = next_key
    next_injection = state / "injections" / f"{next_key}.md"
    next_injection.parent.mkdir(parents=True, exist_ok=True)
    next_injection.write_text(source.read_text())
    revised["inject_prompt_file"] = str(next_injection)
    (state / "oracles" / f"{next_key}.md").write_text(oracle.read_text())
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(revised, indent=2) + "\n")

    print(f"carried  : {len(touched)} changed path(s) in {profile['root']}")
    print(f"injection: {next_injection}")
    print(f"oracle   : copied unchanged from {task_key}")
    print(f"profile  : {target}")
    print(f"next     : lint, grant, doctor, snapshot, dispatch -- {next_key}")


def abandon(profile: dict, task_key: str) -> None:
    """Release a run id whose dispatch produced nothing.

    A one-shot run id is spent the moment a conversation exists, because a
    second dispatch onto a live conversation forks a history the controller
    cannot audit. A dispatch killed before the worker wrote a path, ran a
    command, or filed a report left no history to fork: the round documents are
    still the ones the controller authored and `lint` graded, and minting a
    fresh id would only re-do authoring that was never consumed.

    Emptiness is measured, never asserted — against the worker's own tree, its
    command log, and its report — so a round that did produce something cannot
    be discarded by calling it dead. The dead attempt's logs are parked rather
    than deleted, both to keep the evidence and because
    `conversation_id_for_task` recovers the id from the run log, so leaving it
    in place would re-spend the id on the next dispatch.
    """
    validate_task_key(profile, task_key)
    state = Path(profile["state_dir"])
    log_dir = state / "runs"
    conversation_id = conversation_id_for_task(profile, task_key)
    if not conversation_id:
        raise SystemExit(
            f"nothing to abandon for {task_key}: no conversation is recorded, "
            "so the run id is already free to dispatch"
        )
    touched = worker_touched_paths(profile, task_key)
    if touched:
        raise SystemExit(
            f"refusing to abandon {task_key}: the worker changed "
            f"{len(touched)} path(s) ({', '.join(touched)}); a round that "
            "produced a candidate is judged with `review` and `verify`, "
            "not discarded as dead"
        )
    snapshot_data = load_snapshot(profile, task_key)
    commands = [
        item
        for item in requested_run_commands(
            conversation_id,
            after_step=int(snapshot_data.get("conversation_step_floor", -1)),
        )
        if not item["denied"]
    ]
    if commands:
        raise SystemExit(
            f"refusing to abandon {task_key}: the worker ran "
            f"{len(commands)} command(s), whose effects reach outside the "
            "checkout this check can see; verify the round instead"
        )
    report_path = log_dir / f"{task_key}.log"
    report = report_path.read_text(errors="replace") if report_path.is_file() else ""
    if extract_exec_report(report) is not None:
        raise SystemExit(
            f"refusing to abandon {task_key}: the worker filed an EXEC REPORT; "
            "run `status` and judge what it claims"
        )
    attempt = 1
    while (log_dir / ABANDONED_RUNS / f"{task_key}.{attempt}").exists():
        attempt += 1
    parked = log_dir / ABANDONED_RUNS / f"{task_key}.{attempt}"
    parked.mkdir(parents=True)
    moved = []
    for path in sorted(log_dir.glob(f"{task_key}.*")):
        if path.is_file():
            path.rename(parked / path.name)
            moved.append(path.name)
    (parked / "abandoned.json").write_text(
        json.dumps(
            {
                "task_key": task_key,
                "conversation_id": conversation_id,
                "at": datetime.now(timezone.utc).isoformat(),
                "parked": moved,
                "measured_empty": {
                    "worker_touched_paths": [],
                    "requested_commands": 0,
                    "exec_report": None,
                },
            },
            indent=2,
        )
        + "\n"
    )
    print(f"abandoned {task_key}: conversation {conversation_id} produced nothing")
    print(f"parked {len(moved)} run artifact(s) under {parked}")
    print(f"next: snapshot, then dispatch {task_key} again")


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


def decisions_path(state_dir: Path, task_key: str) -> Path:
    return state_dir / "decisions" / f"{task_key}.json"


def load_decisions(state_dir: Path, task_key: str) -> list[dict]:
    path = decisions_path(state_dir, task_key)
    if not path.is_file():
        return []
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError):
        return []


def has_valid_adjudication(
    state_dir: Path,
    task_key: str,
    artifact_path_str: str,
    current_hash: str | None,
) -> bool:
    decisions = load_decisions(state_dir, task_key)
    if not decisions or current_hash is None:
        return False

    target_path = Path(artifact_path_str).resolve()

    for dec in decisions:
        if not isinstance(dec, dict) or "path" not in dec:
            continue
        try:
            dec_path = Path(dec["path"]).resolve()
        except Exception:
            continue
        if dec_path == target_path:
            if dec.get("action") in ("admit", "reject") and dec.get("sha256") == current_hash:
                return True
    return False


def scope_decision(state_dir: Path, task_key: str, finding: str) -> str | None:
    """The recorded decision on one scope finding, or None if never judged.

    A protected artifact binds its decision to a content hash. A scope finding
    has no content to hash, but its own text already carries the measurement --
    `442 changed lines exceeds the 60-line budget` names the number it was
    judged at. Keying on the whole string therefore binds the decision the same
    way: one more write changes the delta, changes the string, and the earlier
    decision stops matching instead of silently covering a bigger overrun.
    """
    for dec in load_decisions(state_dir, task_key):
        if not isinstance(dec, dict) or dec.get("kind") != "scope":
            continue
        if dec.get("finding") == finding and dec.get("action") in ("admit", "reject"):
            return dec["action"]
    return None


def split_scope_findings(
    profile: dict, task_key: str, findings: list[str]
) -> tuple[list[str], list[str], list[str]]:
    """Partition findings into unjudged, admitted, and refused.

    `admit` clears a finding; `reject` records that a human read it and refused
    it, which is not the same as never having looked. Both are worth keeping:
    the first is what lets a round proceed, the second is what stops a rejected
    round from being accepted by a later verb that only counted open findings.
    """
    state_dir = Path(profile["state_dir"])
    open_, admitted, rejected = [], [], []
    for finding in findings:
        decision = scope_decision(state_dir, task_key, finding)
        if decision == "admit":
            admitted.append(finding)
        elif decision == "reject":
            rejected.append(finding)
        else:
            open_.append(finding)
    return open_, admitted, rejected


def print_scope_findings(
    open_: list[str], admitted: list[str], rejected: list[str]
) -> None:
    """Report the three states separately, because they mean different things.

    An admitted finding is a judgement already on record. A rejected one is a
    judgement too, and still blocks -- `reject` on a scope finding has nothing
    to restore, so the only thing it can mean is that the round must be revised.
    An open one is the only state where nobody has looked yet, and the hint line
    names the command that changes that.
    """
    if admitted:
        print(f"\nscope findings admitted ({len(admitted)}), not blocking:")
        for item in admitted:
            print(f"  - {item}")
    blocking = open_ + rejected
    if not blocking:
        print("scope findings: none open")
        return
    print(f"\nscope findings ({len(blocking)}) blocking `accept`:")
    for item in open_:
        print(f"  - {item}")
    for item in rejected:
        print(f"  - [rejected, revise or re-admit] {item}")
    print(
        '\n  adjudicate <profile> <key> admit|reject "<finding text, verbatim>"'
    )


def round_scope_findings(profile: dict, task_key: str) -> list[str]:
    """The scope findings this round currently has, whichever mode it runs in."""
    if profile.get("worktree"):
        return scope_findings(
            profile, worker_touched_paths(profile, task_key), task_key
        )
    return inplace_budget_findings(profile, load_snapshot(profile, task_key))


def adjudicate_scope(
    profile: dict, task_key: str, action: str, finding_input: str
) -> None:
    """Record a decision on one scope finding.

    A protected artifact is adjudicated against its bytes, and `reject` restores
    them. A scope finding has no bytes to restore: `reject` therefore records
    that a controller read the finding and refused it, and leaves it blocking,
    because the only thing that clears a refused overrun is the worker writing
    less. That is why the two actions are not symmetric here and are in
    `adjudicate`.
    """
    state_dir = Path(profile["state_dir"])
    current = round_scope_findings(profile, task_key)
    if finding_input not in current:
        listing = (
            "\n  the round's current scope findings are:\n"
            + "\n".join(f"    - {item}" for item in current)
            if current
            else "\n  the round has no scope findings"
        )
        raise SystemExit(
            f"refused: no such finding exists in round {task_key}: "
            f"{finding_input}{listing}"
        )

    decisions = [
        d
        for d in load_decisions(state_dir, task_key)
        if not (
            isinstance(d, dict)
            and d.get("kind") == "scope"
            and d.get("finding") == finding_input
        )
    ]
    decisions.append(
        {
            "finding": finding_input,
            "kind": "scope",
            "action": action,
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }
    )
    d_path = decisions_path(state_dir, task_key)
    d_path.parent.mkdir(parents=True, exist_ok=True)
    d_path.write_text(json.dumps(decisions, indent=2) + "\n")

    print(
        f"adjudicated {task_key}: "
        f"{'admitted' if action == 'admit' else 'rejected'} scope finding\n"
        f"  {finding_input}"
    )
    if action == "reject":
        print("  a rejected scope finding still blocks `accept`; revise the round")


def adjudicate(
    profile: dict,
    task_key: str,
    action: str,
    finding_input: str,
) -> None:
    validate_task_key(profile, task_key)
    if action not in ("admit", "reject"):
        raise SystemExit("action must be admit or reject")

    root = Path(profile["root"])
    state_dir = Path(profile["state_dir"])

    matched_entry = None
    matched_artifact = None
    matched_actual = None

    for entry in profile["protected_artifacts"]:
        artifact = Path(entry["path"])
        actual = sha256(artifact) if artifact.is_file() else None
        if actual != entry["sha256"]:
            finding_str = f"protected artifact changed: {entry['path']}"
            rel_str = (
                str(artifact.relative_to(root))
                if resolved_under(artifact, root)
                else entry["path"]
            )
            candidates = {
                finding_str,
                entry["path"],
                str(artifact),
                rel_str,
                f"protected artifact changed: {rel_str}",
                f"protected artifact changed: {str(artifact)}",
            }
            if (
                finding_input in candidates
                or (
                    Path(finding_input).is_absolute()
                    and Path(finding_input).resolve() == artifact.resolve()
                )
                or (root / finding_input).resolve() == artifact.resolve()
            ):
                matched_entry = entry
                matched_artifact = artifact
                matched_actual = actual
                break

    if matched_entry is None or matched_artifact is None or matched_actual is None:
        # Not a protected artifact. The other kind of finding this round can
        # raise is a scope finding, which `verify` and `review` both name as
        # something to adjudicate; until it could be recorded here, that
        # instruction pointed at a verb that refused every input it was given.
        return adjudicate_scope(profile, task_key, action, finding_input)

    decisions = load_decisions(state_dir, task_key)
    decisions = [
        d
        for d in decisions
        if isinstance(d, dict)
        and Path(d.get("path", "")).resolve() != matched_artifact.resolve()
    ]

    if action == "admit":
        record = {
            "finding": f"protected artifact changed: {matched_entry['path']}",
            "path": str(matched_artifact),
            "action": "admit",
            "sha256": matched_actual,
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }
        decisions.append(record)
        d_path = decisions_path(state_dir, task_key)
        d_path.parent.mkdir(parents=True, exist_ok=True)
        d_path.write_text(json.dumps(decisions, indent=2) + "\n")
        print(
            f"adjudicated {task_key}: admitted {matched_entry['path']} "
            f"(sha256 {matched_actual[:12]})"
        )
    elif action == "reject":
        snapshot_data = load_snapshot(profile, task_key)
        protected_contents = snapshot_data.get("protected_contents_base64", {})
        encoded = (
            protected_contents.get(matched_entry["path"])
            or protected_contents.get(str(matched_artifact))
        )
        if encoded is None:
            try:
                rel = str(matched_artifact.relative_to(root))
                encoded = protected_contents.get(rel)
            except ValueError:
                pass
        if encoded is not None:
            park_original(state_dir, str(matched_artifact), encoded)
            parked = recovery_path(state_dir, str(matched_artifact))
            if parked.is_file():
                matched_artifact.write_bytes(parked.read_bytes())
        elif matched_artifact.is_file():
            matched_artifact.unlink()

        restored_hash = (
            sha256(matched_artifact) if matched_artifact.is_file() else None
        )
        record = {
            "finding": f"protected artifact changed: {matched_entry['path']}",
            "path": str(matched_artifact),
            "action": "reject",
            "sha256": restored_hash or matched_entry["sha256"],
            "recorded_at": datetime.now(timezone.utc).isoformat(),
        }
        decisions.append(record)
        d_path = decisions_path(state_dir, task_key)
        d_path.parent.mkdir(parents=True, exist_ok=True)
        d_path.write_text(json.dumps(decisions, indent=2) + "\n")
        print(
            f"adjudicated {task_key}: rejected {matched_entry['path']}; "
            "restored to snapshot content"
        )


def git_output(root: Path, *args: str) -> str:
    result = subprocess.run(
        [*GIT, "-C", str(root), *args],
        text=True,
        capture_output=True,
        check=True,
    )
    return result.stdout


def default_worktree_path(controller_root: Path, task_key: str) -> Path:
    return (
        controller_root.parent
        / ".agy-worktrees"
        / f"{controller_root.name}-{task_key}"
    )


def grants_baseline_path(state_dir: Path, project_id: str) -> Path:
    return state_dir / f"grants-baseline.{project_id}.json"


def capture_grants_baseline(state_dir: Path, project_id: str) -> Path | None:
    """Record the Project's grants before a round can widen them.

    A round routinely needs one or two commands the persistent Project does not
    grant. Adding them is correct for the round and wrong to keep, but nothing
    remembered the prior state, so the shared Project ratcheted wider with every
    round. Depth is deliberately one: the Project binding moves to the round's
    worktree, so two rounds cannot be outstanding against one Project, and an
    existing baseline therefore means a round is still open.
    """
    path = grants_baseline_path(state_dir, project_id)
    if path.exists():
        return None
    project = json.loads(project_path_by_id(project_id).read_text())
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(project_permission_surface(project), indent=2) + "\n"
    )
    return path


def restore_grants_baseline(state_dir: Path, project_id: str) -> dict | None:
    """Return the Project's grants to their pre-round state and clear the mark."""
    path = grants_baseline_path(state_dir, project_id)
    if not path.exists():
        return None
    baseline = normalize_permission_surface(json.loads(path.read_text()))
    project_path = project_path_by_id(project_id)
    project = json.loads(project_path.read_text())
    before = project_permission_surface(project)
    project.setdefault("permissionGrants", {})["permissionGrants"] = baseline
    project_path.write_text(json.dumps(project, indent=2) + "\n")
    path.unlink()
    return {
        "removed": {
            kind: sorted(set(before[kind]) - set(baseline[kind]))
            for kind in PERMISSION_KINDS
        },
        "restored": baseline,
    }


def unnamed_protective_revocations(profile: dict) -> list[str]:
    """Guards the Project holds that this profile would silently drop.

    `grant` installs the declared surface verbatim, so a generated profile's
    empty `deny` did not add this round's allow rules to the Project -- it
    replaced the Project's twenty deny rules with none, which is the inverse of
    what a bounded-write round is for. #3479 fixed the same shape on the `allow`
    side, where an empty list left the worker unable to work and so announced
    itself within one round; an empty `deny` leaves the worker able to do
    anything and announces nothing.

    Measured against the recorded baseline rather than the live Project, so a
    second `grant` on an already-narrowed Project still reports what the round
    started from. Returns the `<kind> <rule>` lines the profile has not named in
    `intentional_revocations`.
    """
    project_id = agy_project_id(profile)
    state_dir = Path(profile["state_dir"])
    path = grants_baseline_path(state_dir, project_id)
    if not path.exists():
        return []
    baseline = normalize_permission_surface(json.loads(path.read_text()))
    declared = expected_project_surface(profile)
    named = set(profile["project_permissions"].get("intentional_revocations", []))
    return [
        entry
        for kind in PROTECTIVE_KINDS
        for rule in sorted(set(baseline[kind]) - set(declared[kind]))
        if (entry := f"{kind} {rule}") not in named
    ]


def uncovered_task_commands(profile: dict, surface: dict[str, list[str]]) -> list[str]:
    """Which `task_commands` a permission surface would still refuse.

    A round's profile is normally derived from the previous round's, and the
    gate command is the one field that always changes. `project_permissions` is
    a second, hand-maintained statement of the same surface, so the derived
    profile keeps granting the *previous* round's gate. `grant` then compares
    live grants against those stale declarations, finds them equal, and reports
    "nothing to change" -- a green that reads as ready while the worker cannot
    run the only command it is being judged on. Resolving through
    `permission_decision`, the same function `doctor` uses, is deliberate: a
    second coverage rule here would be the same defect one layer down.

    A prefix family is probed by its own text, which is the shortest command
    line the family admits. A surface that refuses that refuses the whole
    family; one that admits it may still refuse a longer member, but no
    surface-level check can enumerate an open family, and reporting the
    shortest member is the honest half of the answer rather than a guess.
    """
    global_surface = global_permission_surface()
    return [
        command
        for command in (
            *profile["task_commands"].get("allow", []),
            *profile["task_commands"].get("allow_prefix", []),
        )
        if permission_decision(surface, global_surface, command)[0] != "allow"
    ]


def unauthorized_surface_refusals(profile: dict) -> list[str]:
    """Ways a declared surface authorizes nothing the round is judged by.

    `grant` installs the declared surface verbatim, so it cannot tell a profile
    that *declares* the empty surface from one that never declared a surface at
    all — and `make_profile.py` used to emit the second. Installing it revoked
    every command the Project held, and the checks downstream then all iterate
    `task_commands.allow`: an empty allow leaves nothing uncovered, matches the
    live surface it just overwrote, and probes zero commands, so `doctor`
    printed `dispatch_ready: true` for a worker that could not run its own gate.

    The two omissions are named here once. `grant` refuses on them, so a round
    cannot start beneath them, and `doctor` blocks on them, so a surface
    installed by some route other than `grant` is caught too.
    """
    findings = []
    declared = expected_project_surface(profile)
    permissions = profile.get("project_permissions", {})
    if not declared["allow"] and not permissions.get("no_shell", False):
        findings.append(
            "`project_permissions.allow` is empty, so the declared surface "
            "authorizes no command at all; a round whose worker genuinely "
            'needs no shell must say so with `"no_shell": true`'
        )
    gate = profile.get("task_contract", {}).get("gate_command")
    # A `no_shell` round is exempt because its gate is the controller's own
    # instrument, not the worker's: `prove` runs it here, after the worker is
    # done. `oracle_findings` already skips its gate cross-check for the same
    # reason, and refusing here would make measure-only rounds unrunnable.
    if (
        gate
        and not permissions.get("no_shell", False)
        and not task_allowlist_admits(profile, gate)
    ):
        findings.append(
            "`task_commands` names the round's own gate command neither exactly "
            "nor by prefix, so the worker cannot run what it is judged by: "
            f"{gate}"
        )
    return findings


def grant(profile: dict) -> None:
    """Make the live Project grants match what this profile declares.

    The declared surface already had to equal the live one for `doctor` to pass,
    so the round-local widening was being applied by hand to AGY's JSON — the
    one place the skill told its user never to hand-edit. Doing it here keeps
    the profile the single statement of the round's surface, and refusing
    without a baseline means nothing is ever widened that `discard` cannot
    put back.
    """
    project_id = agy_project_id(profile)
    state_dir = Path(profile["state_dir"])
    if not grants_baseline_path(state_dir, project_id).exists():
        raise SystemExit(
            "refusing to widen: no grants baseline recorded, so `discard` "
            "could not restore the Project. Run `worktree` first."
        )
    declared = expected_project_surface(profile)
    refusals = unauthorized_surface_refusals(profile)
    if refusals:
        raise SystemExit(
            "refusing to install this surface, because it would leave the "
            "worker unable to work:\n"
            + "\n".join(f"  - {finding}" for finding in refusals)
            + "\n\nThe Project's live grants are unchanged."
        )
    dropped = unnamed_protective_revocations(profile)
    if dropped:
        raise SystemExit(
            "refusing to install this surface, because it would revoke guards "
            "the Project holds and this profile does not name:\n"
            + "\n".join(f"  - {entry}" for entry in dropped)
            + "\n\nThe Project's live grants are unchanged. Either add these to "
            "`project_permissions.deny`/`.ask`, or, if the round means to give "
            "them up, copy the lines above into "
            "`project_permissions.intentional_revocations`."
        )
    uncovered = uncovered_task_commands(profile, declared)
    if uncovered:
        raise SystemExit(
            "the profile's `project_permissions` do not cover its own "
            "`task_commands`, so installing them would leave the worker "
            "unable to run:\n"
            + "\n".join(f"  - {command}" for command in uncovered)
            + "\n\nAdd `command(...)` (and `unsandboxed(...)` where the "
            "command needs the network or writes outside the worktree) to "
            "`project_permissions.allow`."
        )
    project_path = project_path_by_id(project_id)
    project = json.loads(project_path.read_text())
    before = project_permission_surface(project)
    if before == declared:
        print("live grants already match the profile; nothing to change")
        return
    project.setdefault("permissionGrants", {})["permissionGrants"] = declared
    project_path.write_text(json.dumps(project, indent=2) + "\n")
    for kind in PERMISSION_KINDS:
        for rule in sorted(set(declared[kind]) - set(before[kind])):
            print(f"+ {kind} {rule}")
    # A removal used to print as `- allow command(sed)`, shaped exactly like an
    # addition and reading as a deliberate narrowing whether or not it was one.
    # Narrowing stays legal; it just has to announce itself as a revocation.
    revoked = [
        (kind, rule)
        for kind in PERMISSION_KINDS
        for rule in sorted(set(before[kind]) - set(declared[kind]))
    ]
    if revoked:
        print(
            f"\nrevocations ({len(revoked)}) — the Project held these and this "
            "round's profile does not:"
        )
        for kind, rule in revoked:
            print(f"- {kind} {rule}")
    print(f"\n`discard` restores the baseline at {grants_baseline_path(state_dir, project_id)}")


def repoint_project_root(project_id: str, root: Path) -> Path:
    """Rebind one persistent AGY Project to `root` and return its prior root.

    `agy --project <id>` forces the worker's working directory to the Project's
    registered folder and ignores the caller's cwd, so a derived worktree is
    only reachable by moving that binding. Moving it is deliberately preferred
    over cloning the Project: one Project per work area means one reviewed
    permission surface that cannot drift from a stale copy, and one registry
    entry instead of one per round.

    Only `projectResources` moves. The permission grants this dispatcher
    audits are the same object before and after, so the permission digest is
    unchanged by construction.
    """
    path = project_path_by_id(project_id)
    document = json.loads(path.read_text())
    previous = project_root(document)
    document["projectResources"] = {
        "resources": [{"gitFolder": {"folderUri": root.as_uri()}}]
    }
    document["updatedAt"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    path.write_text(json.dumps(document, indent=2) + "\n")
    return previous


def worktree(profile_path: str, task_key: str) -> None:
    """Derive the worker's checkout and branch from the controller's."""
    if not TASK_KEY_PATTERN.match(task_key):
        raise SystemExit(f"invalid task key: {task_key}")
    raw = json.loads(Path(profile_path).read_text())
    declared = raw.get("controller_root")
    if not declared:
        raise SystemExit(
            "profile missing controller_root: the checkout this dispatch "
            "derives its worker worktree from"
        )
    controller_root = Path(declared).resolve()
    if not (controller_root / ".git").exists():
        raise SystemExit(f"controller_root is not a git checkout: {controller_root}")

    spec = dict(raw.get("worktree") or {})
    branch = spec.get("branch") or f"{DERIVED_BRANCH_PREFIX}{task_key}"
    if not branch.startswith(DERIVED_BRANCH_PREFIX):
        raise SystemExit(
            f"worker branch must start with {DERIVED_BRANCH_PREFIX!r} so it is "
            f"never confused with a persistent ref: {branch}"
        )
    path = Path(
        spec.get("path") or default_worktree_path(controller_root, task_key)
    ).resolve()
    if path == controller_root or path.is_relative_to(controller_root):
        raise SystemExit(
            "the worker worktree must live outside controller_root so the "
            "controller's own git status stays clean"
        )
    base_ref = spec.get("base_ref") or "HEAD"
    base_sha = git_output(controller_root, "rev-parse", base_ref).strip()

    if path.exists():
        current = git_output(path, "rev-parse", "--abbrev-ref", "HEAD").strip()
        if current != branch:
            raise SystemExit(
                f"{path} already exists on branch {current}, not {branch}"
            )
        base_sha = spec.get("base_sha") or base_sha
        print(f"reusing worker worktree {path} on {branch}")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            [
                *GIT,
                "-C",
                str(controller_root),
                "worktree",
                "add",
                "-b",
                branch,
                str(path),
                base_sha,
            ],
            check=True,
        )
        print(f"created worker worktree {path} on {branch} at {base_sha[:12]}")

    project_id = raw.get("agy_project_id") or spec.get("agy_project_id")
    if not project_id:
        candidates = project_ids_for_root(controller_root) or project_ids_for_root(
            path
        )
        if len(candidates) != 1:
            detail = "none" if not candidates else ", ".join(candidates)
            raise SystemExit(
                f"expected one AGY project for {controller_root}; found "
                f"{detail}. Register the work area once with "
                "`agy --new-project`, then set agy_project_id in the profile."
            )
        project_id = candidates[0]

    baseline = capture_grants_baseline(Path(raw["state_dir"]), project_id)
    home_root = spec.get("project_home_root")
    previous = repoint_project_root(project_id, path)
    if home_root is None:
        home_root = str(previous) if previous else str(controller_root)
    print(f"AGY project {project_id} now runs in {path}")
    print(f"  (its home root {home_root} is restored by `discard`)")
    if baseline:
        print(f"  (its grants are recorded at {baseline} and restored by `discard`)")
    else:
        print(
            f"  (a grants baseline already exists at "
            f"{grants_baseline_path(Path(raw['state_dir']), project_id)}; "
            "an earlier round was never discarded)"
        )

    raw["root"] = str(path)
    raw["agy_project_id"] = project_id
    raw["worktree"] = {
        **spec,
        "branch": branch,
        "path": str(path),
        "base_ref": base_ref,
        "base_sha": base_sha,
        "derived_from": str(controller_root),
        "project_home_root": home_root,
    }
    Path(profile_path).write_text(json.dumps(raw, indent=2) + "\n")
    print(f"profile bound to the worker worktree: {profile_path}")


def worktree_spec(profile: dict) -> dict:
    spec = profile.get("worktree") or {}
    if not spec.get("base_sha"):
        raise SystemExit(
            "profile has no worktree.base_sha; run `worktree PROFILE TASK_KEY` "
            "first to derive the worker checkout"
        )
    return spec


def round_baseline(profile: dict, task_key: str) -> dict[str, str | None]:
    """Pre-round content of every path the snapshot recorded, by repo path.

    `base_sha` is only the round's baseline when the round started from a clean
    checkout. A revision continues on the prior round's worktree, where that
    round's candidate is still uncommitted, so measuring against `base_sha`
    charges this worker for its predecessor's diff. The snapshot is the tree as
    it stood when this round was dispatched, which is what "what did this worker
    change" means for both cases. `None` marks a path absent before the round.
    """
    try:
        snap = load_snapshot(profile, task_key)
    except SystemExit:
        return {}
    baseline: dict[str, str | None] = dict(snap.get("writable_contents") or {})
    for relative, encoded in (snap.get("protected_contents_base64") or {}).items():
        baseline.setdefault(
            relative, base64.b64decode(encoded).decode(errors="surrogateescape")
        )
    return baseline


def current_text(root: Path, relative: str) -> str | None:
    path = root / relative
    return path.read_text(errors="surrogateescape") if path.is_file() else None


def worker_touched_paths(profile: dict, task_key: str | None = None) -> list[str]:
    root = Path(profile["root"])
    base = worktree_spec(profile)["base_sha"]
    changed = git_output(root, "diff", "--name-only", base).splitlines()
    untracked = git_output(
        root, "ls-files", "--others", "--exclude-standard"
    ).splitlines()
    touched = {line for line in [*changed, *untracked] if line}
    if task_key:
        # Drop paths the git comparison reports only because an earlier round on
        # this same worktree left them uncommitted. A path whose content still
        # equals what the snapshot recorded was not written by this worker.
        baseline = round_baseline(profile, task_key)
        touched = {
            relative
            for relative in touched
            if relative not in baseline
            or baseline[relative] != current_text(root, relative)
        }
    return sorted(touched)


def budget_overrun(relative: str, delta: int, budget: int | str) -> str:
    """The one sentence a budget overrun is ever written as.

    Two paths measure this -- `scope_findings` from the round baseline, the
    in-place branch of `verify` from the snapshot -- and they will not always
    agree on `delta`, because they compare against different befores. What they
    must agree on is the shape of the sentence, because that sentence is the
    ledger key: a decision recorded from one path has to be findable from the
    other, and a decision recorded at one delta must stop matching when the
    worker writes more.
    """
    return f"{relative}: {delta} changed lines exceeds the {budget}-line budget"


def inplace_budget_findings(profile: dict, snapshot_data: dict) -> list[str]:
    """Budget overruns measured against the snapshot, for in-place rounds.

    Derived-worktree rounds get this from `scope_findings`; an in-place round
    has no worktree to diff, so the before is the snapshot's recorded text.
    """
    root = Path(profile["root"])
    findings = []
    for relative, budget in (profile.get("path_change_budgets") or {}).items():
        before_text = snapshot_data.get("writable_contents", {}).get(relative)
        path = root / relative
        after_text = (
            path.read_text(errors="surrogateescape") if path.is_file() else None
        )
        before_lines = [] if before_text is None else before_text.splitlines()
        after_lines = [] if after_text is None else after_text.splitlines()
        delta = sum(
            1
            for line in difflib.ndiff(before_lines, after_lines)
            if line.startswith("+ ") or line.startswith("- ")
        )
        if delta > int(budget):
            findings.append(budget_overrun(relative, delta, budget))
    return findings


def scope_findings(
    profile: dict, touched: list[str], task_key: str | None = None
) -> list[str]:
    """Classify the candidate against the declared contract.

    In derived-worktree mode these are review inputs, not verdicts. The
    worker's tree is its own, so writing outside the declared set costs the
    controller a read, never a lost round.

    The two halves ask different questions and are measured differently.
    "Wrote outside the declared set" is about this worker, so it reads the
    per-revision `touched` its caller passed. "Declared but did not write" is
    about the candidate: `revise` carries uncommitted work forward, so a path
    written in an earlier revision is in the tree and absent from `touched`,
    and charging the revision for it fires the finding on correct work.
    """
    findings = []
    root = Path(profile["root"]).resolve()
    allowed = set(profile["allowed_repo_writes"])
    protected_rel = set()
    for entry in profile.get("protected_artifacts", []):
        p = Path(entry["path"])
        if not p.is_absolute():
            p = (root / p).resolve()
        try:
            protected_rel.add(str(p.relative_to(root)))
        except ValueError:
            protected_rel.add(entry["path"])

    outside = [path for path in touched if path not in allowed and path not in protected_rel]
    if outside:
        findings.append(
            f"wrote {len(outside)} path(s) outside allowed_repo_writes: "
            + ", ".join(outside)
        )
    candidate = worker_touched_paths(profile) if profile.get("worktree") else touched
    unwritten = sorted(allowed - set(candidate))
    if unwritten:
        findings.append(
            f"declared but did not write {len(unwritten)} path(s): "
            + ", ".join(unwritten)
        )
    for relative, budget in (profile.get("path_change_budgets") or {}).items():
        delta = changed_line_count(profile, relative, task_key)
        if delta > int(budget):
            findings.append(budget_overrun(relative, delta, budget))
    for entry in profile["protected_artifacts"]:
        artifact = Path(entry["path"])
        actual = sha256(artifact) if artifact.is_file() else None
        if actual != entry["sha256"]:
            if task_key and has_valid_adjudication(
                Path(profile["state_dir"]), task_key, str(artifact), actual
            ):
                continue
            findings.append(f"protected artifact changed: {entry['path']}")
    head = git_output(root, "rev-parse", "HEAD").strip()
    if head != worktree_spec(profile)["base_sha"]:
        findings.append(
            f"branch HEAD moved from the frozen base to {head[:12]}; the "
            "worker is not permitted to run git mutations"
        )
    return findings


def changed_line_count(profile: dict, relative: str, task_key: str | None = None) -> int:
    root = Path(profile["root"])
    baseline = round_baseline(profile, task_key) if task_key else {}
    if relative in baseline:
        # Same reason as `worker_touched_paths`: on a revision the git baseline
        # is the commit before the *previous* round, so an untracked file the
        # predecessor created bills its whole length to this worker.
        before = (baseline[relative] or "").splitlines()
        after = (current_text(root, relative) or "").splitlines()
        return sum(
            1
            for line in difflib.unified_diff(before, after, n=0, lineterm="")
            if line[:1] in "+-" and line[:3] not in ("+++", "---")
        )
    base = worktree_spec(profile)["base_sha"]
    numstat = git_output(root, "diff", "--numstat", base, "--", relative)
    for line in numstat.splitlines():
        added, removed, *_ = line.split("\t")
        if added == "-" or removed == "-":
            return 0
        return int(added) + int(removed)
    path = root / relative
    if path.is_file():
        return len(path.read_text(errors="surrogateescape").splitlines())
    return 0


def review(profile: dict, task_key: str) -> None:
    """Print the candidate diff and its scope classification.

    This is the acceptance surface: the controller decides from the diff, not
    from the worker's report and not from an exit code.
    """
    validate_task_key(profile, task_key)
    spec = worktree_spec(profile)
    root = Path(profile["root"])
    touched = worker_touched_paths(profile, task_key)
    open_, admitted, rejected = split_scope_findings(
        profile, task_key, scope_findings(profile, touched, task_key)
    )
    findings = open_ + rejected
    allowed = set(profile["allowed_repo_writes"])
    # On a revised round the diff below spans every revision while `touched`
    # names only this one. Naming the difference keeps the header from reading
    # as the whole candidate, which is how a two-file candidate got reviewed
    # as one file on #3351 R6.
    carried = [
        path for path in worker_touched_paths(profile) if path not in touched
    ]

    print(f"worktree : {root}")
    print(f"branch   : {spec['branch']}")
    print(f"base     : {spec['base_sha']}")
    print(f"touched  : {len(touched)} path(s) written this revision")
    for path in touched:
        mark = "  " if path in allowed else "! "
        print(f"  {mark}{path}")
    if carried:
        print(
            f"carried  : {len(carried)} path(s) from an earlier revision, "
            "committed by `accept` with the rest"
        )
        for path in carried:
            print(f"    {path}")
    print_scope_findings(open_, admitted, rejected)

    print("\n" + git_output(root, "diff", "--stat", spec["base_sha"]).rstrip())
    print("\n" + git_output(root, "diff", spec["base_sha"]).rstrip())
    untracked = [
        line
        for line in git_output(
            root, "ls-files", "--others", "--exclude-standard"
        ).splitlines()
        if line
    ]
    for path in untracked:
        print(f"\n=== new file: {path} ===")
        print((root / path).read_text(errors="replace").rstrip())
    if findings:
        sys.exit(EXIT_FINDINGS)


PROOF_LABELS = ("mutant", "candidate")


def candidate_tree_digest(profile: dict) -> str:
    """Identify the worker's candidate by the bytes of what it was allowed to
    write, so a proof cannot be recorded against one tree and spent on another."""
    return json_digest(
        manifest(Path(profile["root"]), profile["allowed_repo_writes"])
    )


def gate_command(profile: dict) -> str:
    command = profile["task_contract"].get("gate_command")
    if not command:
        raise SystemExit(
            "profile has no task_contract.gate_command: name the one command "
            "whose red/green decides this round"
        )
    return command


def proof_path(profile: dict, task_key: str, label: str) -> Path:
    return Path(profile["state_dir"]) / "proofs" / f"{task_key}.{label}.json"


def prove(profile: dict, task_key: str, label: str) -> None:
    """Run the round's gate over the current worktree and record the result.

    A gate nobody has seen fail proves nothing: a test written against the
    implementation that was just produced passes by construction. The
    controller reverts the product change, records `mutant`, restores it, and
    records `candidate`; `accept` then requires that pair. This measures the
    gate, so it stays free of any knowledge of what the gate runs.
    """
    validate_task_key(profile, task_key)
    worktree_spec(profile)
    if label not in PROOF_LABELS:
        raise SystemExit(f"proof label must be one of {', '.join(PROOF_LABELS)}")
    command = gate_command(profile)
    root = Path(profile["root"])
    print(f"gate  : {command}")
    print(f"tree  : {root}")
    result = subprocess.run(
        shlex.split(command),
        cwd=root,
        text=True,
        capture_output=True,
    )
    output = (result.stdout + result.stderr).strip()
    record = {
        "task_key": task_key,
        "label": label,
        "command": command,
        "exit_code": result.returncode,
        "compiled": "could not compile" not in output,
        "tree_digest": candidate_tree_digest(profile),
        "output_tail": output.splitlines()[-20:],
        "recorded_at": datetime.now(timezone.utc).isoformat(),
    }
    path = proof_path(profile, task_key, label)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2) + "\n")
    print(f"exit  : {result.returncode}")
    if not record["compiled"]:
        print("note  : did not compile; this red says nothing about behaviour")
    print(f"digest: {record['tree_digest'][:12]}")
    print(f"saved : {path}")


def capture_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "transcripts" / f"{task_key}.json"


def load_captures(profile: dict, task_key: str) -> dict[str, list[str]]:
    """Every transcript this round captured, keyed by the command that made it.

    An absent file is an empty set rather than an error: a round whose current
    behavior is entirely a code quote never captures anything, and that is a
    complete round.
    """
    path = capture_path(profile, task_key)
    if not path.exists():
        return {}
    try:
        records = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError):
        return {}
    return {
        record["command"]: record["output"]
        for record in records
        if isinstance(record, dict) and "command" in record
    }


def capture(profile: dict, task_key: str, command: str, cwd: str | None) -> None:
    """Run a command and store what it printed, for `## Current behavior`.

    The section quoting source is grounded by the checkout; the section showing
    behavior had nothing, because there is no file holding what a binary prints.
    The prompt line was meant to ground it and could not: it says a command was
    run, and saying so is free. This makes the claim cost a run.

    The record keeps the command, the directory, and the exit code, so a reader
    who doubts the block can reproduce it instead of trusting it -- which is the
    property the prompt line only ever promised.
    """
    validate_task_key(profile, task_key)
    root = Path(cwd) if cwd else Path(profile["root"])
    if not root.is_dir():
        raise SystemExit(f"capture directory does not exist: {root}")
    result = subprocess.run(
        ["bash", "-lc", command],
        cwd=root,
        text=True,
        capture_output=True,
    )
    output = [
        line.rstrip()
        for line in (result.stdout + result.stderr).splitlines()
        if line.strip()
    ]
    record = {
        "task_key": task_key,
        "command": command,
        "cwd": str(root),
        "exit_code": result.returncode,
        "output": output,
        "recorded_at": datetime.now(timezone.utc).isoformat(),
    }
    path = capture_path(profile, task_key)
    path.parent.mkdir(parents=True, exist_ok=True)
    records = []
    if path.exists():
        try:
            records = [
                item
                for item in json.loads(path.read_text())
                if isinstance(item, dict) and item.get("command") != command
            ]
        except (OSError, json.JSONDecodeError):
            records = []
    records.append(record)
    path.write_text(json.dumps(records, indent=2) + "\n")

    print(f"cwd   : {root}")
    print(f"exit  : {result.returncode}")
    print(f"saved : {path}")
    print("\npaste this into `## Current behavior`:\n")
    print("```console")
    print(f"$ {command}")
    for line in output:
        print(line)
    print("```")


def sweep_path(profile: dict, task_key: str) -> Path:
    return Path(profile["state_dir"]) / "proofs" / f"{task_key}.sweep.json"


def sweep(profile: dict, task_key: str, script: str) -> None:
    """Run the controller's mutation sweep over the worktree and record it.

    The proof pair shows the gate notices *this* change. It says nothing about
    the neighbouring wrong changes a reviewer actually fears: a constant where
    a computed value belongs, an untouched arm of the same enum, a guard
    relaxed by one clause. Only a sweep measures those, and until now the
    strongest evidence a round produced lived in a scratch file that was
    deleted with the job -- so the round's published claim of "8/8 killed" was
    the one number nobody else could re-run. This stores the script's own text
    beside its result.

    It also answers a question the sweep cannot ask about itself: a sweep that
    fails to restore the tree leaves every later mutant measured against the
    wrong baseline. Comparing the digest across the run is cheap and turns that
    silent corruption into a refusal.
    """
    validate_task_key(profile, task_key)
    worktree_spec(profile)
    source = Path(script)
    if not source.exists():
        raise SystemExit(f"sweep script does not exist: {script}")
    root = Path(profile["root"])
    text = source.read_text()
    before = candidate_tree_digest(profile)
    print(f"sweep : {source}")
    print(f"tree  : {root}")
    command = (
        [sys.executable, str(source)]
        if source.suffix == ".py"
        else [str(source)]
    )
    result = subprocess.run(command, cwd=root, text=True, capture_output=True)
    output = (result.stdout + result.stderr).strip()
    after = candidate_tree_digest(profile)
    record = {
        "task_key": task_key,
        "script": str(source.resolve()),
        "script_sha256": hashlib.sha256(text.encode()).hexdigest(),
        "script_source": text,
        "exit_code": result.returncode,
        "restored": before == after,
        "tree_digest": after,
        "output": output.splitlines(),
        "recorded_at": datetime.now(timezone.utc).isoformat(),
    }
    path = sweep_path(profile, task_key)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(record, indent=2) + "\n")
    for line in output.splitlines()[-30:]:
        print(f"  {line}")
    print(f"exit  : {result.returncode}")
    print(f"saved : {path}")
    # Recorded first, then refused: the record is the evidence that the sweep
    # went wrong, and discarding it would leave the controller with a refusal
    # and nothing to diagnose it from.
    if not record["restored"]:
        raise SystemExit(
            "the tree digest changed across the sweep: it did not restore what "
            "it mutated, so every result after the first was measured against a "
            "corrupted baseline and the kills it reports are unearned. Restore "
            "the tree, fix the script's restore path -- `write_text`, never "
            "`copy2`, whose preserved mtime makes cargo skip the rebuild -- and "
            "record the sweep again"
        )


def proof_findings(profile: dict, task_key: str) -> list[str]:
    """Why this round's gate is not yet shown to discriminate."""
    records = {}
    for label in PROOF_LABELS:
        path = proof_path(profile, task_key, label)
        if not path.exists():
            return [
                f"no `{label}` proof: run `prove PROFILE {task_key} {label}` "
                "with the product change reverted (mutant) and restored "
                "(candidate)"
            ]
        records[label] = json.loads(path.read_text())

    findings = []
    command = gate_command(profile)
    for label, record in records.items():
        if record["command"] != command:
            findings.append(
                f"`{label}` proof ran a different gate: {record['command']!r}"
            )
    if records["mutant"]["exit_code"] == 0:
        findings.append(
            "the gate passed with the product change reverted: it does not "
            "measure this round's change"
        )
    if records["candidate"]["exit_code"] != 0:
        findings.append(
            f"the gate fails on the candidate (exit "
            f"{records['candidate']['exit_code']})"
        )
    if records["mutant"]["tree_digest"] == records["candidate"]["tree_digest"]:
        findings.append(
            "both proofs ran over an identical tree: nothing was actually "
            "reverted between them"
        )
    current = candidate_tree_digest(profile)
    if records["candidate"]["tree_digest"] != current:
        findings.append(
            "the worktree changed after the `candidate` proof was recorded; "
            "re-run it"
        )

    # A recorded sweep is evidence the round will publish, so a broken one is
    # worse than none: it reads as discrimination while measuring nothing.
    path = sweep_path(profile, task_key)
    if path.exists():
        record = json.loads(path.read_text())
        if not record.get("restored", True):
            findings.append(
                "the recorded sweep did not restore the tree it mutated, so "
                "its results are measured against a corrupted baseline; fix "
                "the restore and re-run `sweep`"
            )
        if record["exit_code"] != 0:
            findings.append(
                f"the recorded sweep exited {record['exit_code']}: at least "
                "one mutant did not match its expected verdict, so the gate "
                "is not yet shown to notice the neighbouring wrong changes"
            )
    return findings


def proof_notes(profile: dict, task_key: str) -> list[str]:
    """What the recorded proofs do not establish, short of blocking acceptance.

    A mutant that failed to *compile* is red for a reason that has nothing to
    do with behaviour: the symbol the gate names did not exist yet. For a round
    introducing a new function that is the only answer the revert can give, so
    it cannot block acceptance -- but stored as an ordinary non-zero exit it
    reads exactly like a behavioural kill, which is the false confidence this
    note exists to deny. The remedy is a sweep whose mutants keep the product
    compiling, so that a specific row is what goes red.
    """
    notes = []
    record = {}
    for label in PROOF_LABELS:
        path = proof_path(profile, task_key, label)
        if not path.exists():
            return notes
        record[label] = json.loads(path.read_text())
    if record["mutant"].get("compiled") is False:
        notes.append(
            "the `mutant` proof did not compile, so the gate is shown to need "
            "the new symbol, not to measure its behaviour; discrimination "
            "rests on a sweep whose mutants still compile"
        )
    # Presence was the whole test, so a sweep that went wrong counted exactly
    # as much as one that went right: `sweep` refuses an unrestored tree, but
    # the record it wrote first is still on disk, and a controller who pressed
    # on past that refusal had the note cleared by the very file proving the
    # sweep was worthless.
    recorded = sweep_path(profile, task_key)
    if not recorded.exists():
        notes.append(
            "no mutation sweep is recorded, so the gate is shown to notice "
            "this change and nothing else; a constant standing in for a "
            "computed value, or an untouched arm of the same enum, would pass "
            "it unseen. Record one with `sweep PROFILE TASK_KEY SCRIPT`"
        )
        return notes
    try:
        sweep_record = json.loads(recorded.read_text())
    except (OSError, json.JSONDecodeError):
        notes.append(
            f"the mutation sweep record at {recorded} cannot be read, so the "
            "sweep counts for nothing; record it again"
        )
        return notes
    if not sweep_record.get("restored"):
        notes.append(
            "the recorded mutation sweep did not restore the tree, so every "
            "result after its first mutant was measured against a corrupted "
            "baseline; the kills it reports are unearned"
        )
    elif sweep_record.get("exit_code"):
        notes.append(
            f"the recorded mutation sweep exited {sweep_record['exit_code']}: "
            "either a mutant survived or the sweep itself failed, and the "
            "round's evidence has to say which"
        )
    return notes


def accept(profile: dict, task_key: str) -> None:
    """Commit the candidate on its own branch and name the integration step.

    The controller commits; the worker never runs a git mutation. Integration
    into a persistent branch stays a separate, explicitly invoked command.

    What is committed is the whole candidate, not this revision's writes. The
    two differ the moment a round is revised: `revise` exists to carry the
    worker's uncommitted work forward under a new run id, so a revised round's
    candidate spans every revision while `task_key` names only the last. Passing
    the key here once committed one file of a two-file candidate whose second
    file defined a symbol the first called -- a commit that does not build, and
    `discard` deletes the worktree holding the missing half. `review` still asks
    the narrower question, because "what did this revision write" is exactly
    what a scope finding is about.
    """
    validate_task_key(profile, task_key)
    spec = worktree_spec(profile)
    root = Path(profile["root"])
    touched = worker_touched_paths(profile)
    if not touched:
        raise SystemExit("nothing to accept: the worker changed no files")
    # `verify` reports scope findings and `accept` used to ignore them, so a
    # round could be accepted and committed while `verify` went on exiting 2
    # forever, with nothing anywhere recording which of the two a controller
    # had believed. Refusing here is what makes the earlier report mean
    # something: clearing it costs one `adjudicate`, and that decision is then
    # on the record beside the commit.
    open_, _admitted, rejected = split_scope_findings(
        profile,
        task_key,
        scope_findings(profile, worker_touched_paths(profile, task_key), task_key),
    )
    if open_ or rejected:
        raise SystemExit(
            "refusing to accept: this round has scope findings nobody has "
            "admitted:\n"
            + "\n".join(f"  - {item}" for item in [*open_, *rejected])
            + '\n  adjudicate <profile> <key> admit "<finding text, verbatim>"'
        )
    if profile["mode"] == "bounded-write":
        unproven = proof_findings(profile, task_key)
        if unproven:
            raise SystemExit(
                "refusing to accept: this round's gate is not shown to "
                "discriminate:\n"
                + "\n".join(f"  - {item}" for item in unproven)
            )
        for note in proof_notes(profile, task_key):
            print(f"note: {note}")
    subprocess.run(
        [*GIT, "-C", str(root), "add", "--", *touched],
        check=True,
    )
    issue = profile["task_contract"].get("issue")
    trailer = f"\n\nRefs #{issue}" if issue else ""
    subprocess.run(
        [
            *GIT,
            "-C",
            str(root),
            "commit",
            "-m",
            f"agy({task_key}): accepted worker candidate{trailer}",
        ],
        check=True,
    )
    sha = git_output(root, "rev-parse", "HEAD").strip()
    print(f"\naccepted {sha[:12]} on {spec['branch']} ({len(touched)} path(s))")
    print("integrate from the controller checkout with:")
    print(f"  git -C {spec['derived_from']} cherry-pick {sha}")


def discard(profile_path: str, task_key: str, *, keep_branch: bool = False) -> None:
    """Return the AGY project home, then remove the round's worktree/branch.

    The project binding is restored first so an interrupted cleanup never
    leaves the shared work area pointing at a deleted directory.
    """
    raw = json.loads(Path(profile_path).read_text())
    spec = raw.get("worktree") or {}
    controller_root = spec.get("derived_from") or raw.get("controller_root")
    if not controller_root:
        raise SystemExit("profile has no derived worktree to discard")

    project_id = raw.get("agy_project_id")
    home_root = spec.get("project_home_root") or controller_root
    if project_id:
        repoint_project_root(project_id, Path(home_root).resolve())
        print(f"AGY project {project_id} restored to {home_root}")
        grants = restore_grants_baseline(Path(raw["state_dir"]), project_id)
        if grants is None:
            print("  (no grants baseline recorded; grants left as they are)")
        else:
            withdrawn = sorted(
                rule for kind in PERMISSION_KINDS for rule in grants["removed"][kind]
            )
            print(
                f"  grants restored to baseline; withdrew {len(withdrawn)} "
                "round-local rule(s)"
            )
            for rule in withdrawn:
                print(f"    - {rule}")

    branch = spec.get("branch", "")
    path = spec.get("path")
    if path and Path(path).exists():
        subprocess.run(
            [*GIT, "-C", controller_root, "worktree", "remove", "--force", path],
            check=True,
        )
        print(f"removed worktree {path}")
    subprocess.run(
        [*GIT, "-C", controller_root, "worktree", "prune"],
        check=True,
    )
    if keep_branch:
        print(f"kept branch {branch} as requested")
    elif branch.startswith(DERIVED_BRANCH_PREFIX):
        subprocess.run(
            [*GIT, "-C", controller_root, "branch", "-D", branch],
            check=False,
        )
        print(f"deleted branch {branch}")
    elif branch:
        print(f"kept branch {branch}: not a {DERIVED_BRANCH_PREFIX}* worker branch")

    raw["root"] = str(home_root)
    raw["worktree"] = {**spec, "released_at": datetime.now(timezone.utc).isoformat()}
    Path(profile_path).write_text(json.dumps(raw, indent=2) + "\n")


def verify(profile: dict, task_key: str) -> None:
    require_project_ready(profile)
    validate_task_key(profile, task_key)
    snapshot_data = load_snapshot(profile, task_key)
    assert_snapshot_identity(profile, task_key, snapshot_data)
    if snapshot_data.get("dispatch_contract") != dispatch_contract(profile):
        raise SystemExit("VOID: dispatch contract changed after snapshot")
    if snapshot_data.get("agy_project_id") != agy_project_id(profile):
        raise SystemExit("VOID: AGY project changed after snapshot")
    assert_permission_state_unchanged(profile, snapshot_data)
    documents_frozen = assert_round_documents_unchanged(
        profile, task_key, snapshot_data
    )
    audited_commands, denial_findings, command_notes = audit_task_commands(
        profile, task_key, snapshot_data
    )
    documents = (
        "oracle and injection unchanged since snapshot"
        if documents_frozen
        else "oracle and injection NOT compared"
    )

    state = Path(profile["state_dir"])
    root = Path(profile["root"])
    result = subprocess.run(
        [*GIT, "-C", str(root), "status", "--porcelain=v1", "--untracked-files=all"],
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

    if profile.get("worktree"):
        # Derived-worktree mode. Integrity is proven above; everything the
        # worker did to its own checkout is a diff for the controller to read.
        # A write outside the declared set costs a read, not the round, so it
        # is reported here and adjudicated by `review`.
        open_, admitted, rejected = split_scope_findings(
            profile,
            task_key,
            scope_findings(
                profile, worker_touched_paths(profile, task_key), task_key
            ),
        )
        oracle = state / "oracles" / f"{task_key}.md"
        if not oracle.exists():
            raise SystemExit("VOID: oracle disappeared")
        print(
            "integrity holds: static Project policy, conversation lineage, and "
            f"{len(audited_commands)} task-local shell command(s) match; "
            f"{documents}; oracle sha256={sha256(oracle)}"
        )
        if command_notes:
            print(f"\ncommand notes ({len(command_notes)}), not blocking:")
            for item in command_notes:
                print(f"  - {item}")
        if denial_findings:
            print(f"\ncommand findings ({len(denial_findings)}):")
            for item in denial_findings:
                print(f"  - {item}")
        print_scope_findings(open_, admitted, rejected)
        if open_ or rejected or denial_findings:
            sys.exit(EXIT_FINDINGS)
        return

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

    # An in-place overrun is a write into the controller's own checkout, so it
    # stays fatal -- but adjudicable, exactly like the protected-artifact check
    # immediately below, which is the other in-place VOID a controller can
    # knowingly waive. Both read the same ledger; neither can be waived by
    # accident, because an admitted decision names the delta it was given.
    overruns = inplace_budget_findings(profile, snapshot_data)
    unwaived = [
        item
        for item in overruns
        if scope_decision(state, task_key, item) != "admit"
    ]
    if unwaived:
        raise SystemExit(
            "VOID: diff budget exceeded:\n"
            + "\n".join(f"  - {item}" for item in unwaived)
            + '\n  waive with: adjudicate <profile> <key> admit "<finding>"'
        )

    protected = snapshot_data.get("protected_contents_base64", {})
    for path, expected in snapshot_data["protected_artifacts"].items():
        actual = sha256(Path(path))
        if actual != expected:
            if has_valid_adjudication(state, task_key, path, actual):
                continue
            hint = park_original(state, path, protected.get(path))
            raise SystemExit(
                f"VOID: protected artifact changed: {path}\n  {hint}"
            )
    oracle = state / "oracles" / f"{task_key}.md"
    if not oracle.exists():
        raise SystemExit("VOID: oracle disappeared")
    print(
        "snapshot, protected artifacts, static Project policy, and "
        f"{len(audited_commands)} task-local shell command(s) match; "
        f"{documents}; oracle sha256={sha256(oracle)}"
    )
    if command_notes:
        print(f"\ncommand notes ({len(command_notes)}), not blocking:")
        for item in command_notes:
            print(f"  - {item}")
    if denial_findings:
        print(f"\nfindings ({len(denial_findings)}) for you to adjudicate:")
        for item in denial_findings:
            print(f"  - {item}")
        sys.exit(EXIT_FINDINGS)


def status(profile: dict) -> None:
    log_dir = Path(profile["state_dir"]) / "runs"
    for log in sorted(log_dir.glob("*.log")):
        if log.name.endswith(".agy.log"):
            continue
        text = log.read_text(errors="replace")
        verdict = (
            "DENIED"
            if "auto-denied" in text or "soft-denying" in text
            else "EMPTY"
            if not text.strip()
            else "REPORTED"
            if extract_exec_report(text) is not None
            else "INVALID REPORT"
        )
        print(f"{log.stem}: {verdict}")


# Verbs that operate on the profile file itself, before or after the worker's
# checkout exists, so they must not go through `load_profile`'s root check.
RAW_PROFILE_VERBS = ("worktree", "discard")
TASK_KEY_VERBS = (
    "worktree",
    "scaffold",
    "lint",
    "capture",
    "prove",
    "sweep",
    "dispatch",
    "resume",
    "revise",
    "abandon",
    "snapshot",
    "verify",
    "review",
    "adjudicate",
    "accept",
    "discard",
    "denied",
)


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="verb", required=True)
    for verb in (
        "worktree",
        "grant",
        "doctor",
        "scaffold",
        "lint",
        "capture",
        "prove",
        "sweep",
        "dispatch",
        "resume",
        "revise",
        "abandon",
        "snapshot",
        "verify",
        "review",
        "adjudicate",
        "accept",
        "discard",
        "status",
        "denied",
    ):
        item = sub.add_parser(verb)
        item.add_argument("profile")
        if verb in TASK_KEY_VERBS:
            item.add_argument(
                "task_key",
                help="ticket issue id or explicit one-shot run id",
            )
        if verb == "discard":
            item.add_argument(
                "--keep-branch",
                action="store_true",
                help="release the worktree but retain the worker branch",
            )
        if verb == "revise":
            item.add_argument(
                "next_key",
                help="run id for the revision; a spent one-shot id cannot "
                "dispatch again",
            )
            item.add_argument(
                "injection",
                help="delta contract naming what was wrong and what must "
                "become true",
            )
        if verb == "capture":
            item.add_argument(
                "command",
                help="shell command to run; what it prints becomes the "
                "transcript `## Current behavior` may show",
            )
            item.add_argument(
                "--cwd",
                default=None,
                help="directory to run in; defaults to the round root",
            )
        if verb == "prove":
            item.add_argument(
                "label",
                choices=PROOF_LABELS,
                help="mutant: product change reverted; candidate: as accepted",
            )
        if verb == "sweep":
            item.add_argument(
                "script",
                help="mutation sweep the controller wrote; its text is stored "
                "with its result so the published claim can be re-run",
            )
        if verb == "adjudicate":
            item.add_argument(
                "action",
                choices=("admit", "reject"),
                help="admit the write into the candidate, or reject it: a "
                "protected artifact is restored, a scope finding keeps blocking",
            )
            item.add_argument(
                "finding",
                help="protected artifact path, or a scope finding copied "
                "verbatim from `verify`/`review`",
            )
    args = parser.parse_args()

    if args.verb in RAW_PROFILE_VERBS:
        {
            "worktree": lambda: worktree(args.profile, args.task_key),
            "discard": lambda: discard(
                args.profile, args.task_key, keep_branch=args.keep_branch
            ),
        }[args.verb]()
        return

    task_key = getattr(args, "task_key", None)

    profile = load_profile(
        args.profile,
        # `scaffold` and `lint` run while the round is still being authored, so
        # they must not require the design inputs to be frozen yet.
        validate_design=args.verb
        not in (
            "verify",
            "status",
            "denied",
            "review",
            "grant",
            "scaffold",
            "lint",
            "capture",
            "adjudicate",
        ),
        # `doctor` preflights the round before `scaffold` writes the injection.
        require_injection=args.verb
        not in (
            "verify",
            "status",
            "denied",
            "review",
            "grant",
            "doctor",
            "scaffold",
            "lint",
            "capture",
            "adjudicate",
        ),
        task_key=task_key,
    )
    {
        "grant": lambda: grant(profile),
        "doctor": lambda: doctor(profile),
        "scaffold": lambda: scaffold(profile, args.task_key),
        "lint": lambda: lint(profile, args.task_key),
        "capture": lambda: capture(
            profile, args.task_key, args.command, args.cwd
        ),
        "prove": lambda: prove(profile, args.task_key, args.label),
        "sweep": lambda: sweep(profile, args.task_key, args.script),
        "snapshot": lambda: snapshot(profile, args.task_key),
        "dispatch": lambda: dispatch(profile, args.task_key),
        "resume": lambda: resume(profile, args.task_key),
        "revise": lambda: revise(
            profile, args.profile, args.task_key, args.next_key, args.injection
        ),
        "abandon": lambda: abandon(profile, args.task_key),
        "verify": lambda: verify(profile, args.task_key),
        "review": lambda: review(profile, args.task_key),
        "adjudicate": lambda: adjudicate(
            profile, args.task_key, args.action, args.finding
        ),
        "accept": lambda: accept(profile, args.task_key),
        "status": lambda: status(profile),
        "denied": lambda: denied(profile, args.task_key),
    }[args.verb]()


if __name__ == "__main__":
    main()
