#!/usr/bin/env python3
"""Dispatch one bounded task to headless Codex and independently verify it.

The controller sets the question, briefs the worker, and accepts. Everything
else is a verb here. The worker's report is a claim *about* the diff and is
never the evidence; the evidence is the diff, the command log, and a gate the
controller has seen fail.

Why this is not `agy_dispatch.py` with names swapped
----------------------------------------------------
AGY's permission surface is persistent Project state that the dispatcher has to
install, diff, and restore, and its audit trail is a conversation database.
Codex carries three primitives that replace all of that, and the design here is
shaped by them:

* `CODEX_HOME` selects the whole configuration layer. A round gets its own home
  containing only its rules and its config, so there is no global state to
  mutate and nothing to restore on `discard`. The ambient home is never read.
* execpolicy `.rules` is a first-class command allowlist, and
  `codex execpolicy check` evaluates one command against a rule file offline.
  `doctor` therefore does not merely *describe* the surface, it *tries* it --
  including one command that must not be allowed, which is the only direction
  that can tell a real allowlist from an empty one.
* `--output-schema` constrains the worker's final message to a JSON Schema, so
  the report shape is machine-refused rather than requested in prose.

Isolation is not optional here, and this is measured rather than assumed. The
ambient `~/.codex/config.toml` on this machine installs a `PreToolUse` hook that
rewrites every Bash invocation through a wrapper, so a command the model intends
as `echo hi` reaches the OS as `/bin/zsh -lc "…/cap run 'echo hi'"`. Byte-exact
command auditing against the ambient home is therefore impossible by
construction, and the accumulated user rule file allows `git commit`,
`git push`, and `gh issue close` besides.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

EXIT_VOID = 1
EXIT_FINDINGS = 2

# This checkout enables core.fsmonitor, and a wedged daemon makes any command
# that reads the index block forever. Every git call here disables it.
GIT = ["git", "-c", "core.fsmonitor=false"]

DERIVED_BRANCH_PREFIX = "codex/"
TASK_KEY_PATTERN = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")
HTML_COMMENT = re.compile(r"<!--.*?-->", re.DOTALL)
FENCE = re.compile(r"```.*?```", re.DOTALL)
BACKTICKED = re.compile(r"`([^`\n]+)`")

PROOF_LABELS = ("mutant", "candidate")

# A round is judged on whether it kept the rules, and this is the command the
# controller runs to see the permission surface refuse something. It is chosen
# to be harmless, present on every machine, and never plausibly on an allowlist.
PERMISSION_CONTROL_COMMAND = "git push --force origin main"


# --------------------------------------------------------------------------
# primitives
# --------------------------------------------------------------------------


def sha256_bytes(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def json_digest(value: object) -> str:
    return sha256_bytes(json.dumps(value, sort_keys=True).encode())


def git_output(root: Path, *args: str) -> str:
    return subprocess.run(
        [*GIT, "-C", str(root), *args], capture_output=True, text=True, check=True
    ).stdout


def parse_timeout(value: str) -> int:
    """`45m` / `90s` / `2h` to seconds. A bare integer is seconds."""
    text = str(value).strip()
    if text.isdigit():
        return int(text)
    unit, number = text[-1], text[:-1]
    if not number.isdigit() or unit not in {"s", "m", "h"}:
        raise SystemExit(f"timeout must look like 45m, 90s, or 2h: {value}")
    return int(number) * {"s": 1, "m": 60, "h": 3600}[unit]


# --------------------------------------------------------------------------
# profile
# --------------------------------------------------------------------------

REQUIRED_FIELDS = (
    "controller_root",
    "root",
    "repo",
    "state_dir",
    "mode",
    "task_contract",
    "task_commands",
    "protected_artifacts",
    "snapshot_paths",
    "allowed_repo_writes",
)


def task_session_policy(profile: dict) -> str:
    contract = profile.get("task_contract") or {}
    policy = contract.get("session_policy")
    if policy not in {"ticketed", "one-shot"}:
        raise SystemExit(
            "task_contract.session_policy must be 'ticketed' or 'one-shot': a "
            "ticketed round resumes one conversation per issue, a one-shot "
            "round is spent when its conversation exists"
        )
    return policy


def task_key_of(profile: dict) -> str:
    contract = profile["task_contract"]
    if task_session_policy(profile) == "ticketed":
        return str(contract["issue"])
    return str(contract["run_id"])


def validate_task_key(profile: dict, task_key: str) -> None:
    expected = task_key_of(profile)
    if task_key != expected:
        raise SystemExit(
            f"task key {task_key} is not this profile's task ({expected}). A "
            "profile owns exactly one round; generate a new profile rather "
            "than pointing this one at another key"
        )


def load_profile(path: str, *, require_root: bool = True) -> dict:
    profile = json.loads(Path(path).read_text())
    missing = [field for field in REQUIRED_FIELDS if field not in profile]
    if missing:
        raise SystemExit(f"profile missing: {', '.join(missing)}")
    if profile["mode"] not in {"measure-only", "bounded-write"}:
        raise SystemExit("mode must be measure-only or bounded-write")
    if profile["mode"] == "measure-only" and profile["allowed_repo_writes"]:
        raise SystemExit("measure-only profile cannot grant repository writes")
    task_session_policy(profile)
    if require_root:
        root = Path(profile["root"])
        if not root.is_dir():
            raise SystemExit(
                f"root is not a directory: {root}. Run `worktree` first; it "
                "derives the worker checkout and writes its path here"
            )
        if str(root.resolve()) == str(Path(profile["controller_root"]).resolve()):
            raise SystemExit(
                "root still points at controller_root: the worker would run in "
                "your own checkout. Run `worktree` first"
            )
    for entry in profile.get("task_commands", {}).get("allow", []):
        if not isinstance(entry, str) or not entry.strip():
            raise SystemExit("task_commands.allow holds exact command lines")
    return profile


def state_dir(profile: dict) -> Path:
    path = Path(profile["state_dir"])
    path.mkdir(parents=True, exist_ok=True)
    return path


def round_home(profile: dict) -> Path:
    return state_dir(profile) / "home"


def round_rules_path(profile: dict) -> Path:
    return round_home(profile) / "rules" / "round.rules"


def oracle_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "oracles" / f"{task_key}.md"


def injection_path(profile: dict, task_key: str) -> Path:
    declared = profile.get("inject_prompt_file")
    if declared:
        return Path(declared)
    return state_dir(profile) / "injections" / f"{task_key}.md"


def run_log_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "runs" / f"{task_key}.jsonl"


def report_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "runs" / f"{task_key}.report.json"


def schema_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "runs" / f"{task_key}.schema.json"


def snapshot_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "snapshots" / f"{task_key}.json"


def capture_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "captures" / f"{task_key}.json"


def proof_path(profile: dict, task_key: str, label: str) -> Path:
    return state_dir(profile) / "proofs" / f"{task_key}.{label}.json"


def sweep_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "sweeps" / f"{task_key}.json"


def decisions_path(profile: dict, task_key: str) -> Path:
    return state_dir(profile) / "decisions" / f"{task_key}.json"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


# --------------------------------------------------------------------------
# worktree
# --------------------------------------------------------------------------


def default_worktree_path(controller_root: Path, task_key: str) -> Path:
    return controller_root.parent / f"{controller_root.name}-codex-{task_key}"


def worktree(profile_path: str, task_key: str) -> None:
    """Derive the worker's checkout and branch from the controller's.

    The branch is cut from the controller's current `HEAD`, so anything
    uncommitted in the controller checkout -- the design input, the fixture, the
    file the round extends -- is invisible to the worker. Nothing here can
    detect that, because an uncommitted file and a file that was never written
    look identical from `HEAD`.
    """
    if not TASK_KEY_PATTERN.match(task_key):
        raise SystemExit(f"invalid task key: {task_key}")
    raw = json.loads(Path(profile_path).read_text())
    declared = raw.get("controller_root")
    if not declared:
        raise SystemExit("profile missing controller_root")
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

    raw["root"] = str(path)
    raw["worktree"] = {
        **spec,
        "branch": branch,
        "path": str(path),
        "base_ref": base_ref,
        "base_sha": base_sha,
        "derived_from": str(controller_root),
    }
    Path(profile_path).write_text(json.dumps(raw, indent=2) + "\n")
    print(f"profile bound to the worker worktree: {profile_path}")
    print("next: rules, then doctor")


def worktree_spec(profile: dict) -> dict:
    spec = profile.get("worktree") or {}
    if not spec.get("base_sha"):
        raise SystemExit(
            "profile has no worktree.base_sha; run `worktree PROFILE TASK_KEY` "
            "first to derive the worker checkout"
        )
    return spec


# --------------------------------------------------------------------------
# rules: the round's own CODEX_HOME
# --------------------------------------------------------------------------


def rule_patterns(command: str) -> list[list[str]]:
    """The execpolicy patterns that authorize one exact command line.

    Codex evaluates a linear chain of safe operators per-command, but a line
    carrying a redirection, substitution, or environment assignment reaches the
    policy as a single `["/bin/zsh", "-lc", "<script>"]` invocation. Emitting
    only the tokenized form would therefore authorize the simple case and
    silently deny the compound one, and the denial arrives as a mid-round
    failure the worker reports as a product problem.
    """
    patterns: list[list[str]] = []
    try:
        tokens = shlex.split(command)
    except ValueError as error:
        raise SystemExit(f"cannot tokenize allowed command {command!r}: {error}")
    if not tokens:
        raise SystemExit("task_commands.allow holds a blank command line")
    patterns.append(tokens)
    if any(ch in command for ch in "><$&|;*?()`"):
        patterns.append(["/bin/zsh", "-lc", command])
    return patterns


def authorized_entries(profile: dict) -> list[str]:
    commands = profile["task_commands"]
    return list(commands.get("allow", [])) + list(commands.get("allow_prefix", []))


def render_rules(profile: dict) -> str:
    lines = [
        "# Generated by codex_dispatch.py; this round's entire command surface.",
        "# The ambient ~/.codex/rules is never loaded: the round runs under its",
        "# own CODEX_HOME, so nothing accumulated there can widen this round.",
        "",
    ]
    for command in profile["task_commands"].get("deny", []):
        for pattern in rule_patterns(command):
            lines.append(
                f"prefix_rule(pattern={json.dumps(pattern)}, "
                f'decision="forbidden", justification="denied by this round")'
            )
    # execpolicy matches by prefix either way; the `allow` / `allow_prefix`
    # split is about what `verify` will accept afterwards, not about what the
    # policy layer permits during the run.
    for command in authorized_entries(profile):
        for pattern in rule_patterns(command):
            lines.append(
                f'prefix_rule(pattern={json.dumps(pattern)}, decision="allow")'
            )
    return "\n".join(lines) + "\n"


def render_round_config(profile: dict) -> str:
    """The round's config.toml.

    Trust is declared explicitly because project trust lives in the *config*
    layer, and this home has never seen the derived worktree: without it the
    round's first turn stalls on a trust decision no one is there to make.
    """
    root = str(Path(profile["root"]).resolve())
    lines = [
        f'model = "{profile.get("model", "gpt-5.6-sol")}"',
        f'model_reasoning_effort = "{profile.get("reasoning_effort", "high")}"',
        f'sandbox_mode = "{profile.get("sandbox_mode", "workspace-write")}"',
        "",
        "[shell_environment_policy]",
        'inherit = "core"',
        "",
        f'[projects."{root}"]',
        'trust_level = "trusted"',
    ]
    writable = [
        str((Path(profile["root"]) / rel).resolve())
        for rel in profile.get("extra_writable_roots", [])
    ]
    if writable:
        lines += ["", "[sandbox_workspace_write]", f"writable_roots = {json.dumps(writable)}"]
    return "\n".join(lines) + "\n"


def rules(profile: dict) -> None:
    """Materialize the round's CODEX_HOME: auth, config, and the rule file.

    Nothing outside `state_dir` is touched, which is the whole point. AGY's
    equivalent installs a persistent permission set and has to restore it on
    `discard`; a round-owned home has no such obligation and therefore no way
    to leave the machine changed.
    """
    home = round_home(profile)
    (home / "rules").mkdir(parents=True, exist_ok=True)

    ambient = Path(os.environ.get("CODEX_HOME_SOURCE", Path.home() / ".codex"))
    auth = ambient / "auth.json"
    if not auth.is_file():
        raise SystemExit(
            f"no Codex credentials at {auth}: run `codex login` first. The "
            "round home carries a copy so the round never reads the ambient "
            "configuration for anything else"
        )
    shutil.copy2(auth, home / "auth.json")
    (home / "config.toml").write_text(render_round_config(profile))
    round_rules_path(profile).write_text(render_rules(profile))

    print(f"round home : {home}")
    print(f"rules      : {round_rules_path(profile)}")
    allow = profile["task_commands"].get("allow", [])
    deny = profile["task_commands"].get("deny", [])
    print(f"allowed    : {len(allow)} command line(s)")
    print(f"forbidden  : {len(deny)} command line(s)")
    print("next: doctor")


def execpolicy_decision(rules_file: Path, command: str) -> str | None:
    """The strictest decision a rule file gives one command, or None if unmatched.

    An unmatched command returns `{"matchedRules": []}` with no `decision` key
    at all, so `None` and `"allow"` are the two answers that matter and they are
    distinguishable. Treating a missing key as permissive would make an empty
    rule file look like a complete one.
    """
    tokens = shlex.split(command)
    proc = subprocess.run(
        ["codex", "execpolicy", "check", "--rules", str(rules_file), "--", *tokens],
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0 and not proc.stdout.strip():
        raise SystemExit(
            f"codex execpolicy check failed for {command!r}: {proc.stderr.strip()}"
        )
    try:
        return json.loads(proc.stdout).get("decision")
    except json.JSONDecodeError:
        raise SystemExit(
            f"codex execpolicy check returned no JSON for {command!r}: "
            f"{proc.stdout[:400]}"
        )


# --------------------------------------------------------------------------
# doctor
# --------------------------------------------------------------------------


def manifest(root: Path, relative_paths: list[str]) -> dict[str, str]:
    out: dict[str, str] = {}
    for relative in relative_paths:
        target = root / relative
        if target.is_file():
            out[relative] = sha256_file(target)
    return out


def policy_report(profile: dict) -> dict:
    rules_file = round_rules_path(profile)
    report: dict[str, object] = {"rules_file": str(rules_file)}
    problems: list[str] = []

    if not rules_file.is_file():
        problems.append(f"no round rule file at {rules_file}; run `rules` first")
        report["problems"] = problems
        return report

    home = round_home(profile)
    if not (home / "auth.json").is_file():
        problems.append(f"round home has no auth.json; re-run `rules`")
    stray = sorted(
        p.name for p in (home / "rules").iterdir() if p.name != "round.rules"
    )
    if stray:
        problems.append(
            f"round rules directory holds files this round did not generate: "
            f"{', '.join(stray)}. Delete them; every one of them widens the "
            "worker past the declared surface"
        )

    unauthorized = []
    for command in authorized_entries(profile):
        decision = execpolicy_decision(rules_file, command)
        if decision != "allow":
            unauthorized.append(f"{command!r} resolves to {decision!r}, not 'allow'")
    if unauthorized:
        problems.append(
            "the round's own gate commands are not authorized by its rules:\n  "
            + "\n  ".join(unauthorized)
        )
    report["authorized_commands"] = len(authorized_entries(profile))

    # Codex reads files by running commands. A round that lets the worker write
    # but never read hands it a task it can only guess at, and the guesses come
    # back formatted as observations.
    if profile["allowed_repo_writes"] and not profile["task_commands"].get(
        "allow_prefix"
    ):
        problems.append(
            "this round authorizes writes but no command prefix, so the worker "
            "cannot read any file it was not quoted. Add read commands with "
            "`--allow-prefix`, or state in the injection that every fact it "
            "needs is already in the brief"
        )

    # The only direction that discriminates. An empty rule file authorizes the
    # allowlist vacuously if the default is permissive, and every check above
    # would still pass.
    control = execpolicy_decision(rules_file, PERMISSION_CONTROL_COMMAND)
    report["control_command"] = PERMISSION_CONTROL_COMMAND
    report["control_decision"] = control
    if control == "allow":
        problems.append(
            f"the control command {PERMISSION_CONTROL_COMMAND!r} is allowed by "
            "this round's rules, so the rule file is not bounding anything"
        )

    report["problems"] = problems
    return report


def doctor(profile: dict) -> None:
    problems: list[str] = []
    report = policy_report(profile)
    problems += list(report.get("problems", []))

    root = Path(profile["root"])
    spec = profile.get("worktree") or {}
    if spec.get("branch"):
        current = git_output(root, "rev-parse", "--abbrev-ref", "HEAD").strip()
        if current != spec["branch"]:
            problems.append(
                f"worker checkout is on {current}, not the round branch "
                f"{spec['branch']}"
            )
    dirty = git_output(root, "status", "--porcelain=v1", "--untracked-files=all").strip()
    if dirty:
        problems.append(
            "the worker checkout is not clean before dispatch, so this round's "
            "diff would include changes it did not make:\n  "
            + "\n  ".join(dirty.splitlines()[:20])
        )

    for entry in profile["protected_artifacts"]:
        target = root / entry["path"]
        if not target.is_file():
            problems.append(f"protected artifact is missing: {entry['path']}")
        elif sha256_file(target) != entry["sha256"]:
            problems.append(
                f"protected artifact already differs from the frozen hash: "
                f"{entry['path']}. The profile describes another tree; "
                "regenerate it rather than editing the write set"
            )

    for entry in profile["task_contract"].get("design_inputs", []):
        target = root / entry["path"]
        if not target.is_file():
            problems.append(f"design input is missing: {entry['path']}")
        elif sha256_file(target) != entry["sha256"]:
            problems.append(f"design input hash mismatch: {entry['path']}")

    if profile["mode"] == "bounded-write":
        if not profile["task_contract"].get("gate_command"):
            problems.append(
                "a bounded-write round needs task_contract.gate_command: it is "
                "the one command the round is judged by"
            )
        if not profile["task_contract"].get("design_inputs"):
            problems.append("a bounded-write round needs at least one design input")

    print(json.dumps(report, indent=2))
    for problem in problems:
        print(f"- {problem}")
    print(f"dispatch_ready={'false' if problems else 'true'}")
    if problems:
        raise SystemExit(EXIT_FINDINGS)


# --------------------------------------------------------------------------
# authoring: forms whose slots carry their own rules
# --------------------------------------------------------------------------

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

ORACLE_SKELETON = """\
## Claim

<!-- fill: one falsifiable sentence about behavior observable from outside the
     change; not a description of the edit. Revert the change in your head: a
     claim that stays true either way is not this round's claim. -->

## Measurements

<!-- fill: the rows the gate has to make. At least two, at least one of them the
     negative control, and the control marked in its input or its expected
     observation -- a control named only in the rationale cell is a sentence
     about a control, and the table passes lint while measuring nothing.

     Every row must be a state the product can actually reach. A row resting on
     a value the product never produces is vacuous: it stays green whatever the
     worker writes, and that is how this round most plausibly ends green and
     empty. -->

| # | input | expected observation | why it cannot hold by accident |
|---|---|---|---|
| 1 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 2 | <!-- fill --> | <!-- fill --> | <!-- fill --> |
| 3 | <!-- fill --> (negative control) | <!-- fill: must FAIL --> | <!-- fill --> |

## Gate

<!-- fill: prefilled from the profile. `prove` runs this one command and nothing
     else, so a second command here creates a row no proof ever makes. -->

```
{gate}
```

## Fabrication tells

<!-- fill: what a passing report would look like if the worker faked it. Not the
     worker lying -- the shapes you would otherwise accept. A gate green because
     its rows are unreachable. An assertion on a value the check itself just
     wrote. One list item each. -->
-
"""

INJECTION_SKELETON = """\
## Task

<!-- fill: one imperative sentence naming the change. The worker reads this
     first and reads it as the whole job, so a sentence naming two things buys
     a diff that does one of them. -->

## Current behavior

<!-- fill: quote the artifact as it stands, citing file and line in backticks.
     Lint checks that every quoted line still exists, because the worker is told
     this block is the state as it stands and will go looking for it: a quote
     that was true at an earlier base sends the worker off to improvise. -->

```
```

## Required change

<!-- fill: what becomes true afterwards, as conditions someone outside the
     change could check. No code and no numbered steps: the worker is being paid
     to derive the implementation. A condition you can only state by naming the
     lines that satisfy it is a measurement and belongs in the oracle. -->
-

## Shape to follow

<!-- fill: name the convention already in the tree that this change must match
     -- an existing function, section, or artifact, in backticks -- and say to
     follow it rather than invent a second one. A constraint on the answer, not
     the answer. -->

## Reference

<!-- fill: one row per file, and the reason must say what the worker will learn
     there. "Relevant context" is not a reason and gets the file skimmed. -->

| path | why the worker must read it |
|---|---|
| <!-- fill --> | <!-- fill --> |

## Out of scope

<!-- fill: what must not be touched, beyond what the write allowlist already
     blocks mechanically. The allowlist bounds where the worker may write; this
     bounds what it may redesign, rename, or clean up on the way past. Do not
     restate the allowlists or the report shape -- the dispatcher already sends
     those, and a second copy is one that drifts. -->
-

## Definition of done

<!-- fill: name in backticks where the gate's check lands. The gate says what to
     run; without this the worker guesses and a correct diff arrives in the
     wrong place. The fence must match the oracle's `## Gate` exactly. -->
"""


def scaffold(profile: dict, task_key: str) -> None:
    validate_task_key(profile, task_key)
    gate = profile["task_contract"].get("gate_command", "")
    oracle = oracle_path(profile, task_key)
    injection = injection_path(profile, task_key)
    for target, body in (
        (oracle, ORACLE_SKELETON.replace("{gate}", gate or "<!-- fill -->")),
        (injection, INJECTION_SKELETON),
    ):
        if target.exists():
            print(f"kept {target} (already authored)")
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(body)
        print(f"wrote {target}")
    print(
        "\nFill both, then `lint`. Every slot states the rule that governs it; "
        "the comments are stripped before the worker sees anything."
    )


def controller_notes_removed(text: str) -> str:
    """Strip the slot rationale before the worker reads the document.

    The rules in those comments are addressed to the controller: what a vacuous
    measurement looks like, why a stale quote is dangerous. Reaching the worker
    they stop being guidance and read as task instructions, and a slot's
    rationale is the one thing a bounded executor has no use for -- it is
    material to reason around a constraint with.
    """
    return HTML_COMMENT.sub("", text).strip()


# --------------------------------------------------------------------------
# capture: the only quotable transcript
# --------------------------------------------------------------------------


def load_captures(profile: dict, task_key: str) -> dict[str, list[str]]:
    path = capture_path(profile, task_key)
    if not path.is_file():
        return {}
    return json.loads(path.read_text())


def capture(profile: dict, task_key: str, command: str, cwd: str | None) -> None:
    validate_task_key(profile, task_key)
    root = Path(cwd) if cwd else Path(profile["root"])
    proc = subprocess.run(
        command, shell=True, cwd=str(root), capture_output=True, text=True
    )
    output = (proc.stdout + proc.stderr).splitlines()
    captures = load_captures(profile, task_key)
    captures[command] = output
    write_json(capture_path(profile, task_key), captures)
    print(f"$ {command}")
    for line in output[:40]:
        print(line)
    if len(output) > 40:
        print(f"... {len(output) - 40} more line(s) recorded")
    print(f"\nexit {proc.returncode}; recorded at {capture_path(profile, task_key)}")


# --------------------------------------------------------------------------
# lint
# --------------------------------------------------------------------------


def sections_of(text: str) -> dict[str, str]:
    out: dict[str, str] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.startswith("## "):
            current = line[3:].strip()
            out[current] = ""
        elif current is not None:
            out[current] += line + "\n"
    return out


def missing_or_misordered(text: str, required: tuple[str, ...], label: str) -> list[str]:
    found = [line[3:].strip() for line in text.splitlines() if line.startswith("## ")]
    findings = []
    for name in required:
        if name not in found:
            findings.append(f"{label} is missing `## {name}`")
    present = [name for name in found if name in required]
    if present != [name for name in required if name in present]:
        findings.append(
            f"{label} sections are out of order: {present}. The order is the "
            f"reading order the worker is given: {list(required)}"
        )
    return findings


def gate_commands_in(section: str) -> list[str]:
    commands: list[str] = []
    for block in re.findall(r"```(.*?)```", section, re.DOTALL):
        lines = [line.strip() for line in block.strip().splitlines() if line.strip()]
        if not lines:
            continue
        if lines[0].startswith("$ "):
            # A `$ `-prompted block is a transcript, not a gate.
            continue
        commands.append(lines[-1] if len(lines) == 1 else "\n".join(lines))
    return commands


def unfilled_slots(text: str, label: str) -> list[str]:
    if "<!-- fill" in text:
        count = text.count("<!-- fill")
        return [
            f"{label} still holds {count} unfilled slot(s). Each one names the "
            "rule it wants; a document that keeps them was never authored"
        ]
    return []


def cited_paths(text: str) -> list[str]:
    return [
        value
        for value in BACKTICKED.findall(FENCE.sub("", text))
        if "/" in value and " " not in value and not value.startswith("--")
    ]


def oracle_findings(profile: dict, text: str) -> list[str]:
    findings = unfilled_slots(text, "the oracle")
    findings += missing_or_misordered(text, ORACLE_SECTIONS, "the oracle")
    sections = sections_of(text)

    measurements = sections.get("Measurements", "")
    rows = [
        line
        for line in measurements.splitlines()
        if line.strip().startswith("|")
        and not re.match(r"^\|[\s:|-]+\|$", line.strip())
        and not line.strip().lower().startswith("| #")
    ]
    if len(rows) < 2:
        findings.append(
            "`## Measurements` needs at least two rows: one row cannot show a "
            "gate discriminating"
        )
    # Only the input and expected-observation cells count. Scanning the whole
    # row would let the rationale cell do the marking, and a control named only
    # in the rationale is a sentence about a control: the table then passes
    # while measuring nothing, which is the exact shape the skeleton warns of.
    marked = []
    for row in rows:
        cells = row.split("|")
        measured = " ".join(cells[2:4]).lower()
        if (
            "negative control" in measured
            or "must fail" in measured
            or "must be red" in measured
        ):
            marked.append(row)
    if not marked:
        findings.append(
            "no row marks the negative control in its input or its expected "
            "observation. A control named only in the rationale cell is a "
            "sentence about a control and measures nothing"
        )

    gate_section = sections.get("Gate", "")
    gates = gate_commands_in(gate_section)
    if len(gates) != 1:
        findings.append(
            f"`## Gate` holds {len(gates)} command(s); `prove` runs exactly one, "
            "so any other count creates a row no proof ever makes"
        )
    else:
        declared = profile["task_contract"].get("gate_command")
        if declared and gates[0].strip() != declared.strip():
            findings.append(
                f"`## Gate` is {gates[0]!r} but the profile is judged by "
                f"{declared!r}; `prove` runs the profile's"
            )

    tells = sections.get("Fabrication tells", "")
    if not [line for line in tells.splitlines() if line.strip().startswith("- ")]:
        findings.append(
            "`## Fabrication tells` is empty: name the shapes you would "
            "otherwise accept, or acceptance has nothing to check against"
        )
    return findings


def injection_findings(
    profile: dict, text: str, oracle_text: str, captures: dict[str, list[str]]
) -> list[str]:
    findings = unfilled_slots(text, "the injection")
    findings += missing_or_misordered(text, INJECTION_SECTIONS, "the injection")
    sections = sections_of(text)
    root = Path(profile["root"])

    task = FENCE.sub("", sections.get("Task", "")).strip()
    if task.count(".") > 1:
        findings.append(
            "`## Task` reads as more than one sentence. The worker reads it as "
            "the whole job, so two sentences buy a diff that does one of them"
        )

    quoted: list[str] = []
    for block in re.findall(r"```(.*?)```", sections.get("Current behavior", ""), re.DOTALL):
        quoted += [line.strip() for line in block.splitlines() if line.strip()]
    if not quoted:
        findings.append(
            "`## Current behavior` quotes nothing. The worker is told this is "
            "the state as it stands; without a quote it improvises one"
        )
    else:
        recorded = {line.strip() for lines in captures.values() for line in lines}
        haystack = ""
        for relative in cited_paths(sections.get("Current behavior", "")):
            candidate = root / relative.split(":")[0]
            if candidate.is_file():
                haystack += candidate.read_text(errors="replace")
        for line in quoted:
            if line in haystack or line in recorded:
                continue
            findings.append(
                f"`## Current behavior` quotes a line that is in neither a cited "
                f"file nor a recorded `capture`: {line!r}. A quote true at an "
                "earlier base sends the worker looking for code that moved"
            )
            break

    # The round's own deliverable is cited before it exists, so a writable path
    # is exempt. Without this a greenfield round cannot name its landing site in
    # `## Definition of done` without lint calling it a broken citation.
    deliverables = set(profile["allowed_repo_writes"])
    deliverables |= {str((root / rel).resolve()) for rel in profile["allowed_repo_writes"]}
    for relative in cited_paths(text):
        bare = relative.split(":")[0]
        if bare in deliverables or str((root / bare).resolve()) in deliverables:
            continue
        if not (root / bare).exists():
            findings.append(
                f"the injection cites a path that does not exist in the worker "
                f"checkout: {relative}"
            )

    for name in ("Required change", "Out of scope"):
        body = sections.get(name, "")
        if not [line for line in body.splitlines() if line.strip().startswith("- ")]:
            findings.append(f"`## {name}` has no list items")

    done = sections.get("Definition of done", "")
    declared = gate_commands_in(done)
    judged = gate_commands_in(sections_of(oracle_text).get("Gate", ""))
    if declared and judged and declared[0].strip() != judged[0].strip():
        findings.append(
            f"`## Definition of done` names a different gate than the oracle "
            f"judges by: {declared[0]!r} vs {judged[0]!r}"
        )
    if not BACKTICKED.search(FENCE.sub("", done)):
        findings.append(
            "`## Definition of done` names the gate but not where its check "
            "lands: name the artifact, module, or suite it joins, or the worker "
            "guesses and the diff arrives in the wrong place"
        )
    return findings


def round_findings(profile: dict, task_key: str) -> list[str]:
    oracle = oracle_path(profile, task_key)
    injection = injection_path(profile, task_key)
    if not oracle.is_file():
        return [f"no oracle at {oracle}; run `scaffold` first"]
    if not injection.is_file():
        return [f"no injection at {injection}; run `scaffold` first"]
    oracle_text = oracle.read_text()
    captures = load_captures(profile, task_key)
    return oracle_findings(profile, oracle_text) + injection_findings(
        profile, injection.read_text(), oracle_text, captures
    )


def lint(profile: dict, task_key: str) -> None:
    validate_task_key(profile, task_key)
    findings = round_findings(profile, task_key)
    for finding in findings:
        print(f"- {finding}")
    if findings:
        raise SystemExit(EXIT_FINDINGS)
    print("both documents are structurally sound; next: snapshot")


# --------------------------------------------------------------------------
# snapshot
# --------------------------------------------------------------------------


def round_documents_digest(profile: dict, task_key: str) -> dict[str, str | None]:
    out: dict[str, str | None] = {}
    for label, path in (
        ("oracle", oracle_path(profile, task_key)),
        ("injection", injection_path(profile, task_key)),
    ):
        out[label] = sha256_file(path) if path.is_file() else None
    return out


def snapshot(profile: dict, task_key: str) -> Path:
    validate_task_key(profile, task_key)
    findings = round_findings(profile, task_key)
    if findings:
        raise SystemExit(
            "refusing to snapshot an unlinted round:\n  " + "\n  ".join(findings)
        )
    root = Path(profile["root"])
    writable_contents: dict[str, str | None] = {}
    for relative in profile["allowed_repo_writes"]:
        target = root / relative
        writable_contents[relative] = (
            base64.b64encode(target.read_bytes()).decode() if target.is_file() else None
        )
    payload = {
        "task_key": task_key,
        "contract_digest": json_digest(profile["task_contract"]),
        "rules_digest": sha256_file(round_rules_path(profile)),
        "documents": round_documents_digest(profile, task_key),
        "protected": manifest(root, [e["path"] for e in profile["protected_artifacts"]]),
        "writable_contents": writable_contents,
        "head": git_output(root, "rev-parse", "HEAD").strip(),
    }
    write_json(snapshot_path(profile, task_key), payload)
    print(f"snapshot: {snapshot_path(profile, task_key)}")
    print(f"head    : {payload['head'][:12]}")
    print("next: dispatch (use the Bash tool's run_in_background; it is long)")
    return snapshot_path(profile, task_key)


def load_snapshot(profile: dict, task_key: str) -> dict:
    path = snapshot_path(profile, task_key)
    if not path.is_file():
        raise SystemExit("missing pre-dispatch snapshot; run `snapshot` first")
    return json.loads(path.read_text())


# --------------------------------------------------------------------------
# prompt + report contract
# --------------------------------------------------------------------------

VERDICT_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["status", "changed_paths", "commands_run", "criteria", "notes"],
    "properties": {
        "status": {"enum": ["PASS", "FAIL", "BLOCKED"]},
        "changed_paths": {"type": "array", "items": {"type": "string"}},
        "commands_run": {
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": ["command", "exit_code"],
                "properties": {
                    "command": {"type": "string"},
                    "exit_code": {"type": "integer"},
                },
            },
        },
        "criteria": {
            "type": "array",
            "items": {
                "type": "object",
                "additionalProperties": False,
                "required": ["id", "verdict", "evidence"],
                "properties": {
                    "id": {"type": "string"},
                    "verdict": {"enum": ["PASS", "FAIL"]},
                    "evidence": {"type": "string"},
                },
            },
        },
        "notes": {"type": "string"},
    },
}

# A worker with no authorized command has no evidence for a verdict, so asking
# for one manufactures exactly the fabrication the controller then has to
# detect. It is asked to describe the work instead.
NO_SHELL_SCHEMA = {
    "type": "object",
    "additionalProperties": False,
    "required": ["changed_paths", "what_changed", "uncertainties"],
    "properties": {
        "changed_paths": {"type": "array", "items": {"type": "string"}},
        "what_changed": {"type": "string"},
        "uncertainties": {"type": "array", "items": {"type": "string"}},
    },
}


def render_prompt(profile: dict, task_key: str, oracle_text: str) -> str:
    writes = profile["allowed_repo_writes"] or ["none"]
    allowed = profile["task_commands"].get("allow", [])
    prefixes = profile["task_commands"].get("allow_prefix", [])
    denied = profile["task_commands"].get("deny", [])
    contract = profile["task_contract"]
    policy = task_session_policy(profile)
    identity = f"Ticket: #{task_key}." if policy == "ticketed" else f"One-shot run id: {task_key}."
    design = (
        "\n".join(
            f"- {entry['path']} (sha256 {entry['sha256']})"
            for entry in contract.get("design_inputs", [])
        )
        or "- none; this is a read-only evidence task"
    )
    injection = ""
    path = injection_path(profile, task_key)
    if path.is_file():
        injection = controller_notes_removed(path.read_text())

    return f"""Execute exactly one bounded task. {identity}
Repository root: {profile['root']}
GitHub repo: {profile['repo']}
Task kind: {contract['kind']}
Session policy: {policy}

Frozen design inputs:
{design}

You are a bounded executor, not the project owner. The controller froze this
task and this command surface immediately before the run. Obey the write and
command allowlists below; the controller will void any out-of-scope mutation.

Exact repository write allowlist:
{chr(10).join(f"- {p}" for p in writes)}

Exact shell command lines authorized for this task:
{chr(10).join(f"- {c}" for c in allowed) or "- none"}

Command prefixes authorized for this task, with any arguments:
{chr(10).join(f"- {c} ..." for c in prefixes) or "- none"}

Shell command lines explicitly forbidden for this task:
{chr(10).join(f"- {c}" for c in denied) or "- none"}

Do not change branches, create worktrees, commit, push, mutate a tracker, or
write any repository path outside the exact allowlist. Use absolute paths. If a
command is unavailable, a path is missing, or the task conflicts with the
injected contract, stop and report FAIL instead of improvising.

Every shell call must either copy one authorized command line byte-for-byte, or
start with one of the authorized prefixes above. Against an exact line, a
narrower range, reordered flag, changed quote form, or appended pipeline is a
different command and will be refused.

You read files by running a command. If you need to read something and no
authorized command can reach it, that is a briefing defect, not something to
route around: report BLOCKED and name the path. Do not treat a claim in this
prompt as an observation you made — a premise you were handed is the
controller's, and a digest quoted here was measured by the controller, not by
you. If the task asks you to verify something, verify it with an authorized
command or report that you could not.

## Controller oracle (injected, immutable)

{controller_notes_removed(oracle_text)}

## Round-specific controller injection

{injection or "(none)"}
"""


# --------------------------------------------------------------------------
# dispatch
# --------------------------------------------------------------------------


def run_agent(profile: dict, task_key: str, *, resume_id: str | None) -> None:
    validate_task_key(profile, task_key)
    snap = load_snapshot(profile, task_key)
    if snap["contract_digest"] != json_digest(profile["task_contract"]):
        raise SystemExit(
            "the task contract changed after `snapshot`: re-run `snapshot` so "
            "the round is judged against the contract it was dispatched under"
        )
    if snap["rules_digest"] != sha256_file(round_rules_path(profile)):
        raise SystemExit(
            "the round's rule file changed after `snapshot`: re-run `rules` "
            "then `doctor` then `snapshot`"
        )

    oracle_text = oracle_path(profile, task_key).read_text()
    prompt = render_prompt(profile, task_key, oracle_text)
    schema = VERDICT_SCHEMA if authorized_entries(profile) else NO_SHELL_SCHEMA
    write_json(schema_path(profile, task_key), schema)

    log = run_log_path(profile, task_key)
    log.parent.mkdir(parents=True, exist_ok=True)
    prompt_file = state_dir(profile) / "runs" / f"{task_key}.prompt.md"
    prompt_file.write_text(prompt)

    env = dict(os.environ)
    env["CODEX_HOME"] = str(round_home(profile))
    argv = ["codex", "exec"]
    if resume_id:
        argv += ["resume", resume_id]
    argv += [
        "--json",
        "-C",
        profile["root"],
        "-s",
        profile.get("sandbox_mode", "workspace-write"),
        "-o",
        str(report_path(profile, task_key)),
        "--output-schema",
        str(schema_path(profile, task_key)),
        "-",
    ]

    print(f"prompt : {prompt_file}")
    print(f"home   : {env['CODEX_HOME']}")
    print(f"log    : {log}")
    with log.open("w") as sink:
        proc = subprocess.run(
            argv,
            input=prompt,
            text=True,
            stdout=sink,
            stderr=subprocess.STDOUT,
            env=env,
            cwd=profile["root"],
            timeout=parse_timeout(profile.get("timeout", "45m")),
        )
    print(f"codex exec exited {proc.returncode}")
    print("next: verify")


def dispatch(profile: dict, task_key: str) -> None:
    run_agent(profile, task_key, resume_id=None)


def resume(profile: dict, task_key: str) -> None:
    thread = thread_id(profile, task_key)
    if not thread:
        raise SystemExit(
            "no recorded thread id for this round; there is no conversation to "
            "resume. Dispatch it first"
        )
    run_agent(profile, task_key, resume_id=thread)


# --------------------------------------------------------------------------
# audit
# --------------------------------------------------------------------------


def events(profile: dict, task_key: str) -> list[dict]:
    log = run_log_path(profile, task_key)
    if not log.is_file():
        return []
    out = []
    for line in log.read_text(errors="replace").splitlines():
        line = line.strip()
        if not line.startswith("{"):
            continue
        try:
            out.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return out


def thread_id(profile: dict, task_key: str) -> str | None:
    for event in events(profile, task_key):
        if event.get("type") == "thread.started":
            return event.get("thread_id")
    return None


def executed_commands(profile: dict, task_key: str) -> list[dict]:
    seen: dict[str, dict] = {}
    for event in events(profile, task_key):
        item = event.get("item") or {}
        if item.get("type") != "command_execution":
            continue
        seen[item.get("id", item.get("command", ""))] = {
            "command": item.get("command", ""),
            "exit_code": item.get("exit_code"),
            "status": item.get("status"),
        }
    return list(seen.values())


def authorized_forms(command: str) -> set[str]:
    """Every literal form the OS may show for one authorized command line."""
    forms = {command.strip()}
    forms.add(f'/bin/zsh -lc "{command}"')
    forms.add(f"/bin/zsh -lc '{command}'")
    forms.add(f'bash -lc "{command}"')
    forms.add(f"bash -lc '{command}'")
    return forms


SHELL_WRAPPERS = ("/bin/zsh -lc ", "/bin/bash -lc ", "bash -lc ", "zsh -lc ")


def unwrapped(command: str) -> str:
    """The script a `sh -c`-style wrapper actually runs, or the line itself."""
    text = command.strip()
    for prefix in SHELL_WRAPPERS:
        if text.startswith(prefix):
            inner = text[len(prefix) :].strip()
            if len(inner) >= 2 and inner[0] == inner[-1] and inner[0] in "\"'":
                return inner[1:-1]
            return inner
    return text


def unauthorized_commands(profile: dict, task_key: str) -> list[str]:
    commands = profile["task_commands"]
    # `allow` is byte-exact on purpose: prefix-matching it would authorize the
    # gate plus anything appended to it, which is how a worker turns an audited
    # command into an unaudited one.
    exact: set[str] = set()
    for command in commands.get("allow", []):
        exact |= authorized_forms(command)
    prefixes = [entry.strip() for entry in commands.get("allow_prefix", [])]
    stray = []
    for record in executed_commands(profile, task_key):
        actual = record["command"].strip()
        if actual in exact:
            continue
        inner = unwrapped(actual)
        if inner in exact:
            continue
        if any(inner == p or inner.startswith(p + " ") for p in prefixes):
            continue
        stray.append(actual)
    return stray


# --------------------------------------------------------------------------
# scope
# --------------------------------------------------------------------------


def worker_touched_paths(profile: dict) -> list[str]:
    root = Path(profile["root"])
    out = []
    for line in git_output(root, "status", "--porcelain=v1", "--untracked-files=all").splitlines():
        if not line.strip():
            continue
        out.append(line[3:].strip().strip('"'))
    return sorted(set(out))


def changed_line_count(profile: dict, relative: str) -> int:
    root = Path(profile["root"])
    stat = git_output(root, "diff", "--numstat", "--", relative).split()
    if len(stat) >= 2 and stat[0].isdigit() and stat[1].isdigit():
        return int(stat[0]) + int(stat[1])
    target = root / relative
    return len(target.read_text(errors="replace").splitlines()) if target.is_file() else 0


def edited_paths(profile: dict, task_key: str) -> list[str]:
    """Absolute paths the write tool reported touching.

    `git status` only sees the repository. The write tool is a second channel
    and it can name any absolute path, so a write landing outside `root` is
    invisible to every other check here.
    """
    out: list[str] = []
    for event in events(profile, task_key):
        item = event.get("item") or {}
        if event.get("type") != "item.completed" or item.get("type") != "file_change":
            continue
        for change in item.get("changes") or []:
            path = change.get("path")
            if path:
                out.append(path)
    return sorted(set(out))


def scope_findings(profile: dict, task_key: str) -> list[str]:
    root = Path(profile["root"])
    findings: list[str] = []
    allowed = set(profile["allowed_repo_writes"])
    for absolute in edited_paths(profile, task_key):
        try:
            Path(absolute).resolve().relative_to(root.resolve())
        except ValueError:
            findings.append(f"the write tool touched a path outside the round: {absolute}")
    for relative in worker_touched_paths(profile):
        if relative not in allowed:
            findings.append(f"wrote outside the allowlist: {relative}")
    for entry in profile["protected_artifacts"]:
        target = root / entry["path"]
        if not target.is_file():
            findings.append(f"protected artifact was deleted: {entry['path']}")
        elif sha256_file(target) != entry["sha256"]:
            findings.append(f"protected artifact changed: {entry['path']}")
    for relative, ceiling in (profile.get("path_change_budgets") or {}).items():
        actual = changed_line_count(profile, relative)
        if actual > ceiling:
            findings.append(
                f"{relative} changed {actual} lines against a ceiling of {ceiling}"
            )
    return findings


# --------------------------------------------------------------------------
# verify
# --------------------------------------------------------------------------


def verify(profile: dict, task_key: str) -> None:
    """Were the rules kept -- not whether the change is right.

    Integrity failures raise: the evidence itself is untrustworthy and the round
    is unsalvageable. Scope failures print and exit 2: a candidate still exists
    and the controller adjudicates it by reading the diff.
    """
    validate_task_key(profile, task_key)
    snap = load_snapshot(profile, task_key)

    if snap["contract_digest"] != json_digest(profile["task_contract"]):
        raise SystemExit("VOID: the task contract changed after dispatch")
    if snap["rules_digest"] != sha256_file(round_rules_path(profile)):
        raise SystemExit("VOID: the round's command surface changed after dispatch")
    if snap["documents"] != round_documents_digest(profile, task_key):
        raise SystemExit(
            "VOID: an oracle or injection changed after dispatch, so the "
            "documents on disk are not the ones the worker was judged against"
        )

    stray = unauthorized_commands(profile, task_key)
    if stray:
        raise SystemExit(
            "VOID: the worker ran commands that are not byte-exact copies of "
            "an authorized line:\n  " + "\n  ".join(stray)
        )

    report = report_path(profile, task_key)
    if not report.is_file():
        raise SystemExit(
            f"VOID: no report at {report}. A run that filed nothing produced no "
            "claim to verify"
        )
    try:
        parsed = json.loads(report.read_text())
    except json.JSONDecodeError:
        raise SystemExit(
            f"VOID: the report at {report} is not JSON, so `--output-schema` "
            "did not constrain the final message"
        )

    print(f"thread   : {thread_id(profile, task_key)}")
    print(f"commands : {len(executed_commands(profile, task_key))} executed, all authorized")
    print(f"report   : {report} ({parsed.get('status', 'no status field')})")

    findings = scope_findings(profile, task_key)
    if findings:
        for finding in findings:
            print(f"- {finding}")
        print(
            "\nThese are questions about a candidate that still exists. Read the "
            "diff (`review`) and record a decision (`adjudicate`)."
        )
        raise SystemExit(EXIT_FINDINGS)
    print("no scope findings; next: review")


# --------------------------------------------------------------------------
# review / adjudicate
# --------------------------------------------------------------------------


def review(profile: dict, task_key: str) -> None:
    validate_task_key(profile, task_key)
    root = Path(profile["root"])
    print("=== report ===")
    path = report_path(profile, task_key)
    print(path.read_text() if path.is_file() else "(none)")
    print("\n=== commands ===")
    for record in executed_commands(profile, task_key):
        print(f"[{record['exit_code']}] {record['command']}")
    print("\n=== diff ===")
    print(git_output(root, "status", "--porcelain=v1", "--untracked-files=all"))
    print(git_output(root, "diff"))
    untracked = [
        line[3:].strip()
        for line in git_output(root, "status", "--porcelain=v1", "--untracked-files=all").splitlines()
        if line.startswith("??")
    ]
    for relative in untracked:
        target = root / relative
        if target.is_file():
            print(f"\n--- new file: {relative} ---")
            print(target.read_text(errors="replace"))


def adjudicate(profile: dict, task_key: str, action: str, finding: str) -> None:
    validate_task_key(profile, task_key)
    path = decisions_path(profile, task_key)
    decisions = json.loads(path.read_text()) if path.is_file() else []
    if action == "reject":
        relative = finding.split(":")[-1].strip()
        snap = load_snapshot(profile, task_key)
        original = (snap.get("protected") or {}).get(relative)
        target = Path(profile["root"]) / relative
        if original is None and target.is_file():
            target.unlink()
            print(f"removed {relative}")
        else:
            print(
                f"rejected {relative}; restore it from the round branch's base "
                "commit yourself -- this verb records the decision, it does not "
                "rewrite the tree"
            )
    decisions.append({"finding": finding, "action": action})
    write_json(path, decisions)
    print(f"recorded {action} for {finding!r} at {path}")


# --------------------------------------------------------------------------
# prove / sweep
# --------------------------------------------------------------------------


def candidate_tree_digest(profile: dict) -> str:
    root = Path(profile["root"])
    return sha256_bytes(
        (git_output(root, "status", "--porcelain=v1", "--untracked-files=all") + git_output(root, "diff")).encode()
    )


def gate_command(profile: dict) -> str:
    command = profile["task_contract"].get("gate_command")
    if not command:
        raise SystemExit("this round declares no gate_command")
    return command


def prove(profile: dict, task_key: str, label: str) -> None:
    """Run the gate against the tree as it currently stands and record it.

    `prove` reverts nothing. Restoring the product to baseline while keeping the
    worker's tests -- and restoring the candidate afterwards -- is the
    controller's job, and doing it with a copy that preserves the old mtime lets
    the build skip the rebuild, which is a false kill followed by a false green.
    """
    validate_task_key(profile, task_key)
    command = gate_command(profile)
    proc = subprocess.run(
        command,
        shell=True,
        cwd=profile["root"],
        capture_output=True,
        text=True,
    )
    payload = {
        "label": label,
        "command": command,
        "exit_code": proc.returncode,
        "tree_digest": candidate_tree_digest(profile),
        "tail": (proc.stdout + proc.stderr)[-4000:],
    }
    write_json(proof_path(profile, task_key, label), payload)
    print(f"{label}: exit {proc.returncode}, tree {payload['tree_digest'][:12]}")
    print(payload["tail"][-1500:])


def sweep(profile: dict, task_key: str, script: str) -> None:
    validate_task_key(profile, task_key)
    before = candidate_tree_digest(profile)
    source = Path(script)
    if not source.is_file():
        raise SystemExit(f"no sweep script at {script}")
    proc = subprocess.run(
        [sys.executable, str(source)],
        cwd=profile["root"],
        capture_output=True,
        text=True,
    )
    after = candidate_tree_digest(profile)
    if before != after:
        raise SystemExit(
            "the sweep left the tree different from how it found it, so its "
            "result describes a tree that no longer exists. Restore the "
            "candidate and re-run"
        )
    write_json(
        sweep_path(profile, task_key),
        {
            "script": source.read_text(),
            "exit_code": proc.returncode,
            "output": (proc.stdout + proc.stderr)[-20000:],
            "tree_digest": after,
        },
    )
    print(proc.stdout[-4000:])
    print(f"sweep recorded at {sweep_path(profile, task_key)} (exit {proc.returncode})")


def proof_findings(profile: dict, task_key: str) -> list[str]:
    if profile["mode"] != "bounded-write":
        return []
    findings = []
    proofs = {}
    for label in PROOF_LABELS:
        path = proof_path(profile, task_key, label)
        if not path.is_file():
            findings.append(f"no `{label}` proof; run `prove {task_key} {label}`")
        else:
            proofs[label] = json.loads(path.read_text())
    if len(proofs) < 2:
        return findings
    if proofs["mutant"]["exit_code"] == 0:
        findings.append(
            "the gate passed with the product reverted to baseline, so it does "
            "not discriminate: it would be green without this round's change"
        )
    if proofs["candidate"]["exit_code"] != 0:
        findings.append("the gate fails on the candidate")
    if proofs["mutant"]["tree_digest"] == proofs["candidate"]["tree_digest"]:
        findings.append(
            "both proofs ran against the same tree, so one of them was recorded "
            "without actually restoring anything"
        )
    if proofs["candidate"]["tree_digest"] != candidate_tree_digest(profile):
        findings.append(
            "the tree changed after the candidate proof; re-run `prove "
            f"{task_key} candidate`"
        )
    return findings


# --------------------------------------------------------------------------
# accept / discard
# --------------------------------------------------------------------------


def accept(profile: dict, task_key: str) -> None:
    """Commit the candidate on its own branch and name the integration step.

    The controller commits; the worker never runs a git mutation. Acceptance is
    controller-only and this is where that is enforced rather than asked for.
    """
    validate_task_key(profile, task_key)
    findings = scope_findings(profile, task_key)
    decided = {
        entry["finding"] for entry in json.loads(decisions_path(profile, task_key).read_text())
    } if decisions_path(profile, task_key).is_file() else set()
    undecided = [f for f in findings if f not in decided]
    if undecided:
        raise SystemExit(
            "refusing to accept with undecided scope findings:\n  "
            + "\n  ".join(undecided)
        )
    unproven = proof_findings(profile, task_key)
    if unproven:
        raise SystemExit(
            "refusing to accept: this round's gate is not shown to "
            "discriminate:\n  " + "\n  ".join(unproven)
        )

    root = Path(profile["root"])
    touched = worker_touched_paths(profile)
    if not touched:
        raise SystemExit("nothing to accept: the worker changed no path")
    subprocess.run([*GIT, "-C", str(root), "add", "--", *touched], check=True)
    contract = profile["task_contract"]
    trailer = (
        f"Refs #{contract['issue']}"
        if task_session_policy(profile) == "ticketed"
        else f"Run-Id: {contract['run_id']}"
    )
    message = f"codex({task_key}): accepted worker candidate\n\n{trailer}\n"
    subprocess.run([*GIT, "-C", str(root), "commit", "-m", message], check=True)
    sha = git_output(root, "rev-parse", "HEAD").strip()
    print(f"committed {sha[:12]} on {profile['worktree']['branch']}")
    print(f"integrate with:\n  git -C {profile['controller_root']} cherry-pick {sha}")


def discard(profile_path: str, task_key: str, *, keep_branch: bool = False) -> None:
    raw = json.loads(Path(profile_path).read_text())
    spec = raw.get("worktree") or {}
    path = spec.get("path")
    controller_root = Path(raw["controller_root"])
    if path and Path(path).exists():
        subprocess.run(
            [*GIT, "-C", str(controller_root), "worktree", "remove", "--force", path],
            check=False,
        )
        print(f"removed worktree {path}")
    if not keep_branch and spec.get("branch"):
        subprocess.run(
            [*GIT, "-C", str(controller_root), "branch", "-D", spec["branch"]],
            check=False,
        )
        print(f"deleted branch {spec['branch']}")
    raw["root"] = raw["controller_root"]
    Path(profile_path).write_text(json.dumps(raw, indent=2) + "\n")
    print("the round's CODEX_HOME is under state_dir and changed nothing outside it")


def status(profile: dict) -> None:
    runs = state_dir(profile) / "runs"
    if not runs.is_dir():
        print("no runs")
        return
    for log in sorted(runs.glob("*.jsonl")):
        key = log.stem
        stream = events(profile, key)
        errors = [
            e["item"]["message"]
            for e in stream
            if (e.get("item") or {}).get("type") == "error"
        ]
        completed = any(e.get("type") == "turn.completed" for e in stream)
        verdict = (
            "EMPTY"
            if not stream
            else "COMPLETED"
            if completed
            else "INTERRUPTED"
        )
        print(f"{key}: {verdict} ({len(stream)} events)")
        for message in errors:
            print(f"  error: {message}")


# --------------------------------------------------------------------------
# cli
# --------------------------------------------------------------------------

RAW_PROFILE_VERBS = ("worktree", "discard")
TASK_KEY_VERBS = (
    "worktree",
    "scaffold",
    "capture",
    "lint",
    "snapshot",
    "dispatch",
    "resume",
    "verify",
    "review",
    "adjudicate",
    "prove",
    "sweep",
    "accept",
    "discard",
)
NO_ROOT_VERBS = ("worktree", "discard", "status")


def main() -> None:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="verb", required=True)
    for verb in (
        "worktree",
        "rules",
        "doctor",
        "scaffold",
        "capture",
        "lint",
        "snapshot",
        "dispatch",
        "resume",
        "verify",
        "review",
        "adjudicate",
        "prove",
        "sweep",
        "accept",
        "discard",
        "status",
    ):
        item = sub.add_parser(verb)
        item.add_argument("profile")
        if verb in TASK_KEY_VERBS:
            item.add_argument("task_key", help="issue id, or the one-shot run id")
        if verb == "capture":
            item.add_argument("command")
            item.add_argument("--cwd", default=None)
        if verb == "prove":
            item.add_argument("label", choices=PROOF_LABELS)
        if verb == "sweep":
            item.add_argument("script")
        if verb == "adjudicate":
            item.add_argument("action", choices=("admit", "reject"))
            item.add_argument("finding")
        if verb == "discard":
            item.add_argument("--keep-branch", action="store_true")
    args = parser.parse_args()

    if args.verb in RAW_PROFILE_VERBS:
        if args.verb == "worktree":
            worktree(args.profile, args.task_key)
        else:
            discard(args.profile, args.task_key, keep_branch=args.keep_branch)
        return

    profile = load_profile(args.profile, require_root=args.verb not in NO_ROOT_VERBS)
    {
        "rules": lambda: rules(profile),
        "doctor": lambda: doctor(profile),
        "scaffold": lambda: scaffold(profile, args.task_key),
        "capture": lambda: capture(profile, args.task_key, args.command, args.cwd),
        "lint": lambda: lint(profile, args.task_key),
        "snapshot": lambda: snapshot(profile, args.task_key),
        "dispatch": lambda: dispatch(profile, args.task_key),
        "resume": lambda: resume(profile, args.task_key),
        "verify": lambda: verify(profile, args.task_key),
        "review": lambda: review(profile, args.task_key),
        "adjudicate": lambda: adjudicate(
            profile, args.task_key, args.action, args.finding
        ),
        "prove": lambda: prove(profile, args.task_key, args.label),
        "sweep": lambda: sweep(profile, args.task_key, args.script),
        "accept": lambda: accept(profile, args.task_key),
        "status": lambda: status(profile),
    }[args.verb]()


if __name__ == "__main__":
    main()
