#!/usr/bin/env python3
"""Generate a dispatch profile by freezing the complement of the write scope.

The contract a bounded-write round needs is not "these files are protected" but
"everything except these files is protected". Enumerating the protected side by
hand is where profiles rot: a file added since the last round is silently
unprotected, and `verify` then cannot tell a stray write from an intended one.

This script inverts the job. You name the scope and the handful of paths the
round may write; every other regular file under the scope is frozen with its
current sha256. A path listed as writable does not have to exist yet, so a
greenfield round is expressed the same way as an edit round.

Usage
-----
    python3 make_profile.py \
        --root /abs/path/to/repo \
        --repo owner/name \
        --project-id <registered-agy-project-id> \
        --scope libs/service-auth \
        --issue 3368 \
        --inject /abs/path/to/delta-round.md \
        --design-input libs/service-auth/CAPABILITIES.md \
        --write libs/service-auth/external-contracts/tests/unit/test_runner_protocol.py:2 \
        --out /abs/path/to/profile.json

`--write PATH[:BUDGET]` appends an exact `allowed_repo_writes` entry and, when
BUDGET is given, a `path_change_budgets` ceiling on added+removed lines for that
path. Globs are rejected: `verify` compares exact paths, and a glob that matches
nothing looks identical to a glob that matches everything.

With no `--write`, the profile is `measure-only`. With at least one, it is
`bounded-write` and at least one `--design-input` is required.

`--root` is the *controller's* checkout. It is written to `controller_root`;
the round's own `root` is filled in by `agy_dispatch.py worktree`, which cuts a
branch from the controller's current `HEAD` and points the AGY Project at it.
Protected paths are therefore emitted repo-relative so the frozen complement
follows the round rather than the tree it was generated from.

`--gate` names the one command the round is judged by. It is required whenever
`--write` is passed, and it lands in three places at once — `task_contract`,
`task_commands.allow`, and `project_permissions.allow` — so the emitted profile
authorizes its own gate and `grant` has a real surface to install. Add the
round's build and format commands to both lists by hand afterwards.

Without `--gate` the profile declares a worker with no shell, marked
`project_permissions.no_shell: true` so that a deliberate silence stays
distinguishable from an unfilled profile. That is what a pure authoring round
wants: any test result or per-criterion verdict in the report is then fabricated
by construction. Pass `--allow-shell` when the round needs commands this script
cannot guess, and replace the placeholders in the emitted profile.
"""

from __future__ import annotations

import argparse
import hashlib
import json
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
        ["git", "-C", str(root), "ls-tree", "-r", "-z", "--name-only", "HEAD", "--", scope],
        capture_output=True,
        check=True,
    )
    names = [name for name in listing.stdout.decode().split("\0") if name]
    wanted = [
        name
        for name in names
        if not any(part in SKIP_DIRS for part in Path(name).parts)
    ]
    if not wanted:
        return {}
    batch = subprocess.run(
        ["git", "-C", str(root), "cat-file", "--batch"],
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


def project_protective_rules(project_id: str) -> dict[str, list[str]]:
    """The Project's live `deny` and `ask` rules, read through the dispatcher.

    Imported rather than reimplemented: the two scripts have to agree on where
    a Project document lives and how its grants are shaped, and `grant` refuses
    a declared surface that drops a guard the recorded baseline held. A second
    reader that disagreed by one normalization step would turn every generated
    profile into that refusal.
    """
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import agy_dispatch

    project = json.loads(
        agy_dispatch.project_path_by_id(project_id).read_text()
    )
    surface = agy_dispatch.project_permission_surface(project)
    return {kind: list(surface[kind]) for kind in agy_dispatch.PROTECTIVE_KINDS}


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
        "derived from it by `agy_dispatch.py worktree`",
    )
    ap.add_argument("--repo", required=True, help="owner/name")
    ap.add_argument("--project-id", required=True, help="registered AGY project id")
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
        "It is emitted into task_contract, task_commands.allow, and "
        "project_permissions.allow, so the profile authorizes its own gate "
        "without hand editing",
    )
    ap.add_argument("--model", default="gemini-3.6-flash-high")
    ap.add_argument("--timeout", default="45m")
    ap.add_argument(
        "--allow-shell",
        action="store_true",
        help="do not emit the no-shell contract; you must then fill in the "
        "exact task_commands yourself",
    )
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
            print(
                f"error: --write takes exact paths, not globs: {path}", file=sys.stderr
            )
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

    writable = set(writes)
    protected: list[dict[str, str]] = []
    for scope in args.scope:
        base = root / scope
        if not base.is_dir():
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
        p = root / rel
        if not p.is_file():
            print(f"error: design input {rel} does not exist", file=sys.stderr)
            return 2
        design_inputs.append({"path": rel, "sha256": digest(p)})

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

    # `require_empty_global` is deliberately absent. `doctor` already measures
    # the invariant that matters -- whether an inherited rule widens the worker
    # past the declared surface -- as `global_broadening_rules`, and treats the
    # flag as opt-in extra strictness on top. Emitting it as `true` made every
    # generated profile block on a harmless inherited deny, and a flag the
    # controller flips off each round teaches it to flip past the real finding
    # standing next to it. Add it by hand for a round that genuinely wants an
    # empty global scope.
    # `deny` and `ask` are inherited from the Project rather than emitted empty.
    # `grant` installs the declared surface verbatim, so an empty `deny` did not
    # mean "this round adds nothing" -- it meant "this round revokes every guard
    # the Project holds", and on the 3358-s1f round it revoked twenty of them
    # (`command(git commit)`, `command(git push)`, the recursive-delete rule, …)
    # leaving a bounded-write worker with no denial surface at all. The `allow`
    # side of the same defect was #3479; it announced itself within one round,
    # because a worker that can run nothing stops. This side announces nothing.
    inherited = project_protective_rules(args.project_id)
    project_permissions: dict[str, object] = {
        "allow": [],
        "deny": inherited["deny"],
        "ask": inherited["ask"],
    }
    # `allow_prefix` is emitted empty rather than omitted. A controller learns
    # the round's vocabulary by reading a generated profile, and the key it
    # cannot see is the key it writes a bare verb into `allow` instead of --
    # where the entry authorizes no real invocation exactly and the permission
    # layer beside it admits every invocation by prefix.
    task_commands: dict[str, list[str]] = {
        "allow": [],
        "allow_prefix": [],
        "deny": [],
    }

    # An empty `allow` is what `grant` installs verbatim, so emitting one meant
    # emitting a surface that revokes everything the Project holds and lets
    # every downstream check iterate an empty list and find nothing wrong. The
    # gate is the one command every round has, so it is the one command a
    # generated profile can authorize without guessing.
    if gate:
        contract["gate_command"] = gate
        task_commands["allow"].append(gate)
        project_permissions["allow"].append(f"command({gate})")
    elif not args.allow_shell:
        # A round that measures without running anything is legitimate; it just
        # has to say so, so that an unfilled profile stays distinguishable from
        # a deliberately silent one.
        project_permissions["no_shell"] = True

    profile: dict[str, object] = {
        # `controller_root` is authored; `root` is a placeholder that
        # `agy_dispatch.py worktree` overwrites with the round's derived
        # checkout. Every other verb reads `root`.
        "controller_root": str(root),
        "root": str(root),
        "repo": args.repo,
        "agy_project_id": args.project_id,
        "state_dir": f"/tmp/agy-dispatch/{args.project_id}",
        "mode": "bounded-write" if writes else "measure-only",
        "task_contract": contract,
        "model": args.model,
        "timeout": args.timeout,
        "project_permissions": project_permissions,
        "task_commands": task_commands,
        "protected_artifacts": protected,
        "snapshot_paths": list(args.scope),
        "allowed_repo_writes": writes,
        "path_change_budgets": budgets,
    }
    if args.inject:
        profile["inject_prompt_file"] = args.inject
    if args.allow_shell:
        # Deliberately left for the controller to fill in: an auto-generated
        # allowlist would be a guess. A whole command line goes in `allow`, a
        # verb whose arguments are the worker's business goes in `allow_prefix`,
        # and an entry in the wrong list is the round's own defect: an exact
        # entry matches no real invocation, a prefix entry matches every one.
        project_permissions["allow"].append("REPLACE-WITH-EXACT-command(...)")
        task_commands["allow"].append("REPLACE-WITH-EXACT-COMMAND-LINE")
        task_commands["allow_prefix"].append("REPLACE-WITH-COMMAND-PREFIX-OR-DELETE")

    out = Path(args.out)
    out.write_text(json.dumps(profile, indent=2) + "\n")

    missing = [w for w in writes if not (root / w).exists()]
    task_key = args.issue or args.run_id
    print(f"wrote {out}")
    print(f"mode:                {profile['mode']}")
    print(f"protected artifacts: {len(protected)}")
    print(
        "inherited guards:    "
        f"{len(inherited['deny'])} deny, {len(inherited['ask'])} ask "
        "(carried from the Project so `grant` does not revoke them)"
    )
    print(f"writable paths:      {len(writes)} ({len(missing)} not yet on disk)")
    if budgets:
        print(f"budgeted paths:      {len(budgets)}")
    if args.allow_shell:
        print("NOTE: --allow-shell left placeholder command entries; fill them in.")
    print(
        "\nnext: derive the round's worktree before anything else --\n"
        f"  python3 {Path(__file__).resolve().parent / 'agy_dispatch.py'} "
        f"worktree {out} {task_key}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
