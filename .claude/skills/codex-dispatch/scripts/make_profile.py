#!/usr/bin/env python3
"""Generate a codex-dispatch profile by freezing the complement of the write scope.

The contract a bounded-write round needs is not "these files are protected" but
"everything except these files is protected". Enumerating the protected side by
hand is where profiles rot: a file added since the last round is silently
unprotected, and `verify` then cannot tell a stray write from an intended one.

This script inverts the job. You name the scope and the handful of paths the
round may write; every other tracked file under the scope is frozen with its
sha256 as of HEAD. A path listed as writable does not have to exist yet, so a
greenfield round is expressed the same way as an edit round.

Usage
-----
    python3 make_profile.py \
        --root /abs/path/to/repo \
        --repo owner/name \
        --scope apps/agentic-workflow \
        --issue 3500 \
        --design-input apps/agentic-workflow/CAPABILITIES.md \
        --write apps/agentic-workflow/src/cli/issues.rs:80 \
        --gate 'cargo test -p agentic-workflow --lib' \
        --out /abs/path/to/profile.json

`--write PATH[:BUDGET]` appends an exact `allowed_repo_writes` entry and, when
BUDGET is given, a `path_change_budgets` ceiling on added+removed lines for that
path. Globs are rejected: `verify` compares exact paths, and a glob that matches
nothing looks identical to a glob that matches everything.

`--root` is the *controller's* checkout. It is written to `controller_root`; the
round's own `root` is filled in by `codex_dispatch.py worktree`, which cuts a
branch from the controller's current `HEAD`. Protected paths are therefore
emitted repo-relative so the frozen complement follows the round rather than the
tree it was generated from.

What differs from the AGY profile of the same name
--------------------------------------------------
There is no `project_permissions` block and no project id. AGY's permissions are
persistent Project state that a round has to install and restore, so its profile
must carry the Project's inherited guards or `grant` silently revokes them. A
Codex round gets a private `CODEX_HOME` whose entire rule file is generated from
`task_commands` by `codex_dispatch.py rules`, so `task_commands` is the single
place a command surface is declared and there is nothing to inherit or restore.

`--gate` accordingly lands in two places rather than three, and a round with no
gate emits an empty `task_commands.allow`, which the dispatcher reads as "this
worker has no shell" -- it then asks for a description of the work instead of a
verdict, because a verdict with no authorized command is fabricated by
construction.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

# Build outputs and caches are not part of any contract. Freezing them makes
# every profile stale the moment a tool runs, and a stale profile trains the
# controller to ignore a real VOID.
SKIP_DIRS = {
    ".venv",
    "venv",
    "__pycache__",
    ".pytest_cache",
    ".ruff_cache",
    ".mypy_cache",
    "target",
    "node_modules",
    ".git",
}

GIT = ["git", "-c", "core.fsmonitor=false"]

# Commands that mutate the tracker or the history. The worker never runs these:
# acceptance is the controller's, and a round that can commit can also erase the
# evidence its own verification depends on.
# Read-only by construction. These are prefix rules, so their arguments are not
# audited byte-for-byte; anything that can also write does not belong here.
DEFAULT_READ_PREFIXES = (
    "sed -n",
    "cat",
    "rg",
    "ls",
    "shasum -a 256",
    "git -c core.fsmonitor=false show",
    "git -c core.fsmonitor=false log",
    "git -c core.fsmonitor=false status",
)

DEFAULT_DENY = (
    "git commit",
    "git push",
    "git checkout",
    "git reset",
    "git stash",
    "git rebase",
    "gh issue close",
    "gh pr merge",
)


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def head_digests(root: Path, scope: str) -> dict[str, str]:
    """sha256 of every tracked file under `scope` as of HEAD.

    The controller's working tree is not the tree the round runs on: the worker
    gets a derived worktree checked out at HEAD, so a file the controller is
    editing would be frozen at a hash the worker's checkout never had, and every
    `doctor` would report a mismatch the worker did not cause. Freezing HEAD
    freezes what the worker will actually see. Untracked files are absent from
    that checkout and so are not frozen at all; a worker that creates one is
    caught by `verify` as a stray write, which is the check that owns that case.
    """
    listing = subprocess.run(
        [*GIT, "-C", str(root), "ls-tree", "-r", "-z", "--name-only", "HEAD", "--", scope],
        capture_output=True,
        check=True,
    )
    names = [name for name in listing.stdout.decode().split("\0") if name]
    wanted = [
        name for name in names if not any(part in SKIP_DIRS for part in Path(name).parts)
    ]
    if not wanted:
        return {}
    batch = subprocess.run(
        [*GIT, "-C", str(root), "cat-file", "--batch"],
        input="".join(f"HEAD:{name}\n" for name in wanted).encode(),
        capture_output=True,
        check=True,
    )
    out = batch.stdout
    digests: dict[str, str] = {}
    pos = 0
    for name in wanted:
        header_end = out.index(b"\n", pos)
        size = int(out[pos:header_end].split(b" ")[-1])
        body_start = header_end + 1
        digests[name] = hashlib.sha256(out[body_start : body_start + size]).hexdigest()
        pos = body_start + size + 1
    return digests


def head_digest_file(root: Path, relative: str) -> str | None:
    """sha256 of one tracked file as of HEAD, or None if HEAD does not have it.

    Design inputs are frozen from HEAD for the same reason protected artifacts
    are: `doctor` compares against the worker's derived checkout. Hashing the
    controller's working tree instead makes every design input the controller is
    currently editing report a mismatch the worker did not cause, and a preflight
    that cries wolf is one the controller learns to skip.
    """
    proc = subprocess.run(
        [*GIT, "-C", str(root), "cat-file", "blob", f"HEAD:{relative}"],
        capture_output=True,
    )
    if proc.returncode != 0:
        return None
    return hashlib.sha256(proc.stdout).hexdigest()


def parse_write(spec: str) -> tuple[str, int | None]:
    """Split `path` or `path:budget`. Windows-style drive letters are not a case
    we support, so a lone trailing colon-integer is unambiguously a budget."""
    head, sep, tail = spec.rpartition(":")
    if sep and tail.isdigit():
        return head, int(tail)
    return spec, None


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Freeze the complement of a round's write scope."
    )
    ap.add_argument(
        "--root",
        required=True,
        help="absolute controller repository root; the round's own worktree is "
        "derived from it by `codex_dispatch.py worktree`",
    )
    ap.add_argument("--repo", required=True, help="owner/name")
    ap.add_argument(
        "--scope",
        required=True,
        action="append",
        help="repo-relative directory to freeze; repeatable",
    )
    ap.add_argument("--issue", help="issue number for a ticketed round")
    ap.add_argument("--run-id", help="unique run id for a one-shot round")
    ap.add_argument(
        "--intent",
        help="frozen one-line intent; required with --run-id, since a one-shot "
        "round has no ticket to carry it",
    )
    ap.add_argument("--inject", help="absolute path to the round's delta contract")
    ap.add_argument(
        "--design-input",
        action="append",
        default=[],
        help="repo-relative frozen design artifact; repeatable",
    )
    ap.add_argument(
        "--write",
        action="append",
        default=[],
        help="repo-relative writable path, optionally PATH:BUDGET; repeatable",
    )
    ap.add_argument(
        "--gate",
        help="the one command this round is judged by; required with --write. "
        "It is emitted into task_contract.gate_command and task_commands.allow, "
        "so the generated rule file authorizes the round's own gate",
    )
    ap.add_argument(
        "--allow",
        action="append",
        default=[],
        help="an additional exact command line the worker may run; repeatable. "
        "Each becomes an execpolicy prefix_rule with decision=allow",
    )
    ap.add_argument(
        "--allow-prefix",
        action="append",
        default=[],
        help="a command prefix the worker may run with any arguments; "
        "repeatable. Codex reads files by running commands, so a round that "
        "expects the worker to inspect anything needs read prefixes here "
        "(`--allow-prefix 'sed -n'`). Unlike --allow these are not audited "
        "byte-for-byte, so keep them read-only",
    )
    ap.add_argument(
        "--read-commands",
        action="store_true",
        help="add the default read prefixes (sed -n / cat / rg / ls / shasum -a 256 / "
        "git -c core.fsmonitor=false show / git -c core.fsmonitor=false log)",
    )
    ap.add_argument(
        "--writable-root",
        action="append",
        default=[],
        help="repo-relative directory the sandbox may write outside the "
        "worktree default; repeatable",
    )
    ap.add_argument("--model", default="gpt-5.6-sol")
    ap.add_argument(
        "--reasoning-effort",
        default="high",
        choices=("low", "medium", "high", "xhigh"),
    )
    ap.add_argument(
        "--sandbox",
        default="workspace-write",
        choices=("read-only", "workspace-write"),
        help="danger-full-access is deliberately not offered: a round that "
        "needs it is not a bounded round",
    )
    ap.add_argument("--timeout", default="45m")
    ap.add_argument("--out", required=True, help="absolute path for the profile")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    if not (root / ".git").exists():
        print(f"error: {root} is not a repository root", file=sys.stderr)
        return 2

    if bool(args.issue) == bool(args.run_id):
        print("error: pass exactly one of --issue or --run-id", file=sys.stderr)
        return 2
    if args.run_id and not (args.intent or "").strip():
        print("error: --run-id requires --intent", file=sys.stderr)
        return 2

    writes: list[str] = []
    budgets: dict[str, int] = {}
    for spec in args.write:
        path, budget = parse_write(spec)
        if any(ch in path for ch in "*?["):
            print(f"error: --write takes exact paths, not globs: {path}", file=sys.stderr)
            return 2
        writes.append(path)
        if budget is not None:
            budgets[path] = budget

    if writes and not args.design_input:
        print(
            "error: a bounded-write round needs at least one --design-input",
            file=sys.stderr,
        )
        return 2

    gate = (args.gate or "").strip()
    if writes and not gate:
        print(
            "error: a bounded-write round needs --gate: it is the one command "
            "the round is judged by, and the worker has to be allowed to run it",
            file=sys.stderr,
        )
        return 2

    if writes and args.sandbox == "read-only":
        print(
            "error: --sandbox read-only contradicts --write; the worker would "
            "be denied at the OS layer for doing exactly what it was sent to do",
            file=sys.stderr,
        )
        return 2

    writable = set(writes)
    protected: list[dict[str, str]] = []
    for scope in args.scope:
        if not (root / scope).is_dir():
            print(f"error: scope {scope} is not a directory", file=sys.stderr)
            return 2
        for rel, sha in sorted(head_digests(root, scope).items()):
            if rel in writable:
                continue
            # Repo-relative, not absolute: the round runs in a derived worktree
            # whose root differs from the controller root this was generated
            # against, and the dispatcher resolves relative paths against the
            # round's own root.
            protected.append({"path": rel, "sha256": sha})

    design_inputs = []
    for rel in args.design_input:
        sha = head_digest_file(root, rel)
        if sha is None:
            print(
                f"error: design input {rel} is not tracked at HEAD, so the "
                "worker's derived checkout will not contain it",
                file=sys.stderr,
            )
            return 2
        if (root / rel).is_file() and digest(root / rel) != sha:
            print(
                f"note: {rel} is modified in the controller's tree; the round "
                "freezes and the worker sees the HEAD version"
            )
        design_inputs.append({"path": rel, "sha256": sha})

    contract: dict[str, object] = {
        "kind": "implementation" if writes else "measurement",
        "design_inputs": design_inputs,
    }
    if args.issue:
        contract["session_policy"] = "ticketed"
        contract["issue"] = args.issue
    else:
        contract["session_policy"] = "one-shot"
        contract["run_id"] = args.run_id
        contract["intent"] = args.intent.strip()

    allow: list[str] = []
    if gate:
        contract["gate_command"] = gate
        allow.append(gate)
    for command in args.allow:
        if command.strip() and command.strip() not in allow:
            allow.append(command.strip())

    # The denials are emitted even though the rule file is generated from
    # scratch each round and an unlisted command is already unmatched. They are
    # here so the controller reads them in the profile and so the worker is told
    # in its prompt what it must not attempt: an unmatched command fails as a
    # tool error the worker may report as a product defect, while a `forbidden`
    # rule fails as a refusal it can recognize.
    allow_prefix: list[str] = []
    if args.read_commands:
        allow_prefix.extend(DEFAULT_READ_PREFIXES)
    for prefix in args.allow_prefix:
        if prefix.strip() and prefix.strip() not in allow_prefix:
            allow_prefix.append(prefix.strip())

    task_commands: dict[str, list[str]] = {
        "allow": allow,
        "allow_prefix": allow_prefix,
        "deny": list(DEFAULT_DENY),
    }

    slug = re.sub(r"[^a-z0-9]+", "-", str(root).lower()).strip("-")
    task_key = args.issue or args.run_id

    profile: dict[str, object] = {
        # `controller_root` is authored; `root` is a placeholder that
        # `codex_dispatch.py worktree` overwrites with the round's derived
        # checkout. Every other verb reads `root`.
        "controller_root": str(root),
        "root": str(root),
        "repo": args.repo,
        "state_dir": f"/tmp/codex-dispatch/{slug}/{task_key}",
        "mode": "bounded-write" if writes else "measure-only",
        "task_contract": contract,
        "model": args.model,
        "reasoning_effort": args.reasoning_effort,
        "sandbox_mode": args.sandbox,
        "timeout": args.timeout,
        "task_commands": task_commands,
        "extra_writable_roots": list(args.writable_root),
        "protected_artifacts": protected,
        "snapshot_paths": list(args.scope),
        "allowed_repo_writes": writes,
        "path_change_budgets": budgets,
    }
    if args.inject:
        profile["inject_prompt_file"] = args.inject

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(profile, indent=2) + "\n")

    missing = [w for w in writes if not (root / w).exists()]
    print(f"wrote {out}")
    print(f"mode:                {profile['mode']}")
    print(f"protected artifacts: {len(protected)}")
    print(f"writable paths:      {len(writes)} ({len(missing)} not yet on disk)")
    print(f"authorized commands: {len(allow)} exact, {len(allow_prefix)} prefix")
    if budgets:
        print(f"budgeted paths:      {len(budgets)}")
    if not allow:
        print(
            "NOTE: this worker has no shell. The dispatcher will ask it to "
            "describe its work rather than return a verdict."
        )
    print(
        "\nnext: derive the round's worktree before anything else --\n"
        f"  python3 {Path(__file__).resolve().parent / 'codex_dispatch.py'} "
        f"worktree {out} {task_key}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
