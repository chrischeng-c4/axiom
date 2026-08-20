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
        --scope libs/service-auth \
        --issue 3368 \
        --design-input libs/service-auth/README.md \
        --write libs/service-auth/src/runner_protocol.rs:2 \
        --gate "cargo test -p service-auth"

Five inputs describe the round -- the scope, the ticket, the design inputs, the
write set, and the gate. Everything else is derived, and each flag stays
available to override:

    --root         the repository containing the current directory
    --repo         owner/name from `origin`
    --project-id   the one AGY Project registered for --root
    --out          {state_dir}/rounds/{task_key}.profile.json
    --inject       {state_dir}/injections/{task_key}.md, where `scaffold` writes
                   the round's delta contract

A typed input is not free, and the two that hurt are the two that do not fail
loudly. An omitted `--inject` dispatched the round with no delta contract at
all, and `lint` reported the injection green because it reads the file rather
than the wiring. A wrong `--project-id` names a real Project, passes every
check, and runs the round against the wrong work area.

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


def dispatcher():
    """The sibling script, imported once.

    Every derivation below reuses one it already owns. A second implementation
    of "where does a Project document live" or "which Project is this root" that
    disagreed by one normalization step would turn every generated profile into
    a refusal the controller cannot read.
    """
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import agy_dispatch

    return agy_dispatch


def project_protective_rules(project_id: str) -> dict[str, list[str]]:
    """The Project's live `deny` and `ask` rules, read through the dispatcher.

    `grant` refuses a declared surface that drops a guard the recorded baseline
    held, so these have to be read the way `grant` reads them.
    """
    agy_dispatch = dispatcher()
    project = json.loads(
        agy_dispatch.project_path_by_id(project_id).read_text()
    )
    surface = agy_dispatch.project_permission_surface(project)
    return {kind: list(surface[kind]) for kind in agy_dispatch.PROTECTIVE_KINDS}


def inherited_global_allow() -> list[str]:
    """The `allow` rules already in force on this machine, whoever the round is.

    `doctor` blocks a profile whose declared surface is narrower than the
    inherited one, and it is right to: an inherited `allow` widens the worker
    past what the round says it authorized, so the round would be measured
    against a surface that is not the one it ran under.

    The Project's `deny` rules were already carried for the mirror-image reason.
    Carrying `allow` too is what makes the declared surface equal the effective
    one. Derived rather than transcribed, because a transcription is a list that
    drifts silently: the surface it describes is edited on the machine, not in
    the profile.
    """
    return list(dispatcher().global_permission_surface()["allow"])


def git_line(cwd: Path, *args: str) -> str | None:
    result = subprocess.run(
        ["git", "-C", str(cwd), *args], capture_output=True, text=True, check=False
    )
    if result.returncode:
        return None
    return result.stdout.strip() or None


def derive_root(explicit: str | None) -> Path | None:
    """The controller checkout, from cwd when not named.

    A mistyped `--root` freezes the complement of a tree the round never runs
    on, and every path in the profile is then wrong together, which reads like
    one large deliberate scope rather than one typo.
    """
    if explicit:
        return Path(explicit).resolve()
    top = git_line(Path.cwd(), "rev-parse", "--show-toplevel")
    return Path(top).resolve() if top else None


def derive_repo(root: Path, explicit: str | None) -> str | None:
    """`owner/name` from origin, in either URL spelling git writes."""
    if explicit:
        return explicit
    url = git_line(root, "remote", "get-url", "origin")
    if not url:
        return None
    trimmed = url[:-4] if url.endswith(".git") else url
    # `git@host:owner/name` and `https://host/owner/name` differ only in the
    # separator before the owner, and both end in the two segments wanted.
    parts = trimmed.replace(":", "/").rstrip("/").split("/")
    if len(parts) < 2:
        return None
    return "/".join(parts[-2:])


def derive_project_id(root: Path, explicit: str | None) -> tuple[str | None, str]:
    """The registered Project for this root, or the reason there isn't one.

    This is the input that cannot fail loudly when hand-copied: a wrong id names
    a real Project, so it passes every check and the round runs against the
    wrong work area. Deriving it removes the only silent typo in the set.

    The lookup resolves now because `worktree` rebinds the Project to the
    round's checkout only later. It comes back empty in exactly one situation --
    a previous round was never discarded, so the binding still points at that
    round's worktree -- and saying so here is earlier than discovering it in
    `worktree`.
    """
    if explicit:
        return explicit, ""
    matches = dispatcher().project_ids_for_root(root)
    if len(matches) == 1:
        return matches[0], ""
    if not matches:
        return None, (
            f"no AGY Project is registered for {root}. A previous round that "
            "was never discarded leaves its Project bound to that round's "
            "worktree, which looks exactly like this; run `discard` for it, or "
            "pass --project-id."
        )
    return None, (
        f"{len(matches)} AGY Projects are registered for {root}: "
        + ", ".join(matches)
        + ". Pass --project-id to say which."
    )


def parse_write(spec: str) -> tuple[str, int | None, str | None]:
    """Split `path`, `path:budget`, or `path:budget:ranges`. Windows-style drive
    letters are not a case we support, so colon-separated trailing fields are
    unambiguously budget and ranges."""
    parts = spec.split(":")
    if len(parts) == 1:
        return parts[0], None, None
    elif len(parts) == 2:
        path, b_str = parts
        if b_str.isdigit():
            return path, int(b_str), None
        elif b_str.lower() in ("none", ""):
            return path, None, None
        else:
            return path, None, b_str
    else:
        path = parts[0]
        b_str = parts[1]
        r_str = ":".join(parts[2:])
        budget = int(b_str) if b_str.isdigit() else None
        ranges = r_str if r_str else None
        return path, budget, ranges


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Freeze the complement of a round's write scope."
    )
    ap.add_argument(
        "--root",
        help="absolute controller repository root; the round's own worktree is "
        "derived from it by `agy_dispatch.py worktree`. Defaults to the "
        "repository containing the current directory",
    )
    ap.add_argument(
        "--repo", help="owner/name; defaults to origin's owner/name"
    )
    ap.add_argument(
        "--project-id",
        help="registered AGY project id; defaults to the one Project "
        "registered for --root",
    )
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
    ap.add_argument(
        "--inject",
        help="absolute path to the round's delta contract; defaults to "
        "{state_dir}/injections/{task_key}.md, which is where `scaffold` "
        "writes it",
    )
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
    ap.add_argument("--model", default="gemini-3.7-flash-high")
    ap.add_argument("--timeout", default="45m")
    ap.add_argument(
        "--allow-shell",
        action="store_true",
        help="do not emit the no-shell contract; you must then fill in the "
        "exact task_commands yourself",
    )
    ap.add_argument(
        "--out",
        help="absolute path for the profile; defaults to "
        "{state_dir}/rounds/{task_key}.profile.json",
    )
    args = ap.parse_args()

    root = derive_root(args.root)
    if root is None:
        print(
            "error: --root was not given and the current directory is not "
            "inside a git repository",
            file=sys.stderr,
        )
        return 2
    if not (root / ".git").exists():
        print(f"error: {root} is not a repository root", file=sys.stderr)
        return 2

    if bool(args.issue) == bool(args.run_id):
        print("error: pass exactly one of --issue or --run-id", file=sys.stderr)
        return 2

    if args.run_id and not (args.intent or "").strip():
        print("error: --run-id requires --intent", file=sys.stderr)
        return 2

    repo = derive_repo(root, args.repo)
    if not repo:
        print(
            f"error: --repo was not given and {root} has no `origin` remote to "
            "read owner/name from",
            file=sys.stderr,
        )
        return 2

    project_id, problem = derive_project_id(root, args.project_id)
    if project_id is None:
        print(f"error: {problem}", file=sys.stderr)
        return 2

    writes: list[str] = []
    budgets: dict[str, int] = {}
    ranges: dict[str, str] = {}
    for spec in args.write:
        path, budget, rspec = parse_write(spec)
        if any(ch in path for ch in "*?["):
            print(
                f"error: --write takes exact paths, not globs: {path}", file=sys.stderr
            )
            return 2
        writes.append(path)
        if budget is not None:
            budgets[path] = budget
        if rspec is not None:
            ranges[path] = rspec
        else:
            if not (root / path).exists():
                ranges[path] = "new"
            else:
                ranges[path] = "any"

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
    inherited = project_protective_rules(project_id)
    inherited_allow = inherited_global_allow()
    project_permissions: dict[str, object] = {
        "allow": list(inherited_allow),
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
    elif not args.allow_shell and not project_permissions["allow"]:
        # A round that measures without running anything is legitimate; it just
        # has to say so, so that an unfilled profile stays distinguishable from
        # a deliberately silent one.
        #
        # Conditional on the surface actually being empty, because the inherited
        # rules can make it not be. `no_shell` beside a non-empty `allow` is a
        # profile that says the worker runs nothing while authorizing it to run
        # thirteen things, and it is read as an exemption: it turns off the
        # refusal that a round must authorize its own gate.
        project_permissions["no_shell"] = True

    profile: dict[str, object] = {
        # `controller_root` is authored; `root` is a placeholder that
        # `agy_dispatch.py worktree` overwrites with the round's derived
        # checkout. Every other verb reads `root`.
        "controller_root": str(root),
        "root": str(root),
        "repo": repo,
        "agy_project_id": project_id,
        "state_dir": f"/tmp/agy-dispatch/{project_id}",
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
        "path_line_ranges": ranges,
    }
    # Left unset, the round dispatches with no delta contract at all. `scaffold`
    # writes `injections/{task_key}.md` and `injection_path()` resolves that
    # default, but the prompt assembly reads `inject_prompt_file` directly, so a
    # profile that does not name one sends the worker the oracle and nothing
    # else. `scaffold` says so and `lint` does not, because `lint` reads the file
    # rather than the wiring -- which is how a fully green `lint` came to mean a
    # round that was never actually briefed.
    profile["inject_prompt_file"] = args.inject or str(
        Path(profile["state_dir"])
        / "injections"
        / f"{args.issue or args.run_id}.md"
    )
    if args.allow_shell:
        # Deliberately left for the controller to fill in: an auto-generated
        # allowlist would be a guess. A whole command line goes in `allow`, a
        # verb whose arguments are the worker's business goes in `allow_prefix`,
        # and an entry in the wrong list is the round's own defect: an exact
        # entry matches no real invocation, a prefix entry matches every one.
        project_permissions["allow"].append("REPLACE-WITH-EXACT-command(...)")
        task_commands["allow"].append("REPLACE-WITH-EXACT-COMMAND-LINE")
        task_commands["allow_prefix"].append("REPLACE-WITH-COMMAND-PREFIX-OR-DELETE")

    task_key = args.issue or args.run_id
    # `revise` already writes its successor to `{state_dir}/rounds/{key}.profile.json`,
    # so a generated profile that lands anywhere else is the odd one out, and a
    # mistyped `--out` writes a profile the dispatch verbs then cannot find.
    out = (
        Path(args.out)
        if args.out
        else Path(profile["state_dir"]) / "rounds" / f"{task_key}.profile.json"
    )
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(profile, indent=2) + "\n")

    missing = [w for w in writes if not (root / w).exists()]
    print(f"wrote {out}")
    print(f"mode:                {profile['mode']}")
    print(f"protected artifacts: {len(protected)}")
    print(
        "inherited guards:    "
        f"{len(inherited['deny'])} deny, {len(inherited['ask'])} ask "
        "(carried from the Project so `grant` does not revoke them)"
    )
    print(
        "inherited allow:     "
        f"{len(inherited_allow)} (carried from the global surface so the "
        "declared surface equals the effective one)"
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
