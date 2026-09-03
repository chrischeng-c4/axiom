#!/usr/bin/env python3
"""Issue work-item engine: everything that does not know which type it serves.

Split out of `epic.py`, which is now a read-compatible legacy facade.
`milestone.py` owns release epics, versions, membership, and order.
The split follows one line: a name belongs here if adding a second work-item
type would otherwise copy it. That is 60% of the original file -- checkout
resolution, the section-schema machinery, the `gh` layer, staging paths, and
seven of the eleven verbs -- and copying any of it is how two types that are
supposed to share a tracker start disagreeing about it.

`_repo_root()` is the sharpest instance. It shipped broken once (walking up
from `__file__` only, which finds no `aw.toml` when the plugin is installed
outside every checkout) and now has a dedicated gate on it,
`verification/probe_offtree_root.py`. A second copy would be a second thing that
gate does not watch.

What is *not* here: any section, label, or rule that names an issue type. Those
live with their issue facade. `epic.py` retains the retired issue-epic schema
for reads. Its CLI refuses create, update, and close.

The generic verbs read `args.wi_type`, which the facade's `main()` binds before
dispatch. They take it from `args` rather than as a parameter so that argparse
`set_defaults(func=...)` stays a plain function reference on both sides.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from collections.abc import Callable
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import quote

import wi_types


def _checkout_boundary(start: Path) -> Path | None:
    """The git checkout `start` stands in, or `None` if it stands in none.

    Asked of git rather than inferred from the directory layout, because a
    linked worktree's root carries a `.git` *file* and a submodule's carries
    one too; only git knows which tree a path belongs to.
    """
    proc = subprocess.run(
        ("git", "-c", "core.fsmonitor=false", "rev-parse", "--show-toplevel"),
        capture_output=True, text=True,
        cwd=start if start.is_dir() else start.parent,
    )
    if proc.returncode != 0 or not proc.stdout.strip():
        return None
    return Path(proc.stdout.strip()).resolve()


def outermost_aw_toml(start: Path) -> Path | None:
    """The outermost `aw.toml` at or above `start`, without leaving its checkout.

    Two rules, and each one exists because the other alone was wrong.

    *Outermost*, not nearest: this checkout carries 37 `aw.toml` files -- one
    per project under `apps/` and `libs/` -- and exactly one, the repository
    root's, holds the `[agentic_workflow.issue_platform]` section
    `default_repo()` reads. Stopping at the nearest marker means running from
    `apps/<name>/` resolves to a project `aw.toml` with no tracker in it, and
    stages bodies under `apps/<name>/.aw/` besides.

    *Without leaving its checkout*, which the unbounded walk did not do. A git
    worktree may sit inside another checkout -- Claude Code puts its own under
    `.claude/worktrees/<name>/`, which is where an agent session stands -- and
    both trees carry a root `aw.toml`. The unbounded walk took the enclosing
    checkout's, so every phase script run from an agent worktree measured, and
    would have written to, the *other* tree: `metadoc.py check` read a dirty
    set belonging to a different session, and `meta.py`, which resolves the
    root with `git rev-parse --show-toplevel`, disagreed with it inside a
    single landing sequence.

    The boundary is dropped when git reports a root that is not on the walk --
    an unreachable git, a path reached through a symlink -- because a boundary
    that is not an ancestor cannot truncate the chain, and guessing there is
    how this resolves to nothing at all.
    """
    start = start.resolve()
    chain = [start, *start.parents]
    boundary = _checkout_boundary(start)
    if boundary is not None and boundary in chain:
        chain = chain[: chain.index(boundary) + 1]
    found = [c for c in chain if (c / "aw.toml").is_file()]
    return found[-1] if found else None


def _repo_root() -> Path:
    """Find the checkout that owns `aw.toml`, searching cwd before `__file__`.

    The marker file identifies the checkout; a fixed parent count would break
    the moment the script is relocated or mirrored to another tree.

    cwd is searched first because this script has been distributed outside
    every checkout -- it was a Claude Code plugin under `~/.claude/plugins/`
    until 2026-08-21 -- and walking up from `__file__` alone finds no
    `aw.toml` at all there, so the script dies before doing anything. The
    `__file__` leg stays as the fallback for a copy that does live inside a
    checkout, and it stays second on purpose: when both resolve, the tree the
    caller is standing in is the one they meant.

    `outermost_aw_toml()` carries the two rules the walk itself obeys.
    """
    starts = (Path.cwd().resolve(), Path(__file__).resolve().parent)
    for start in starts:
        found = outermost_aw_toml(start)
        if found is not None:
            return found
    raise SystemExit(
        f"error: no `aw.toml` found above the working directory ({starts[0]}) "
        f"or the script ({starts[1]}); run this from inside an axiom checkout"
    )


REPO_ROOT = _repo_root()
AW_TOML = REPO_ROOT / "aw.toml"

PRIORITIES = ("p0", "p1", "p2", "p3", "p4", "p5")

# Where work-item bodies are staged, relative to the checkout root. `.aw/` is
# gitignored (`.gitignore:3`), so a staged body never shows up as untracked
# residue in the tree it was authored against.
#
# The extra `workitems/` level is load-bearing rather than decorative: `.aw/`
# already carries a dozen legacy namespaces from an earlier AW, one of which is
# `.aw/changes/<change_id>/` -- a per-change directory that means something
# entirely different from a `type=change` work item. Nesting under `workitems/`
# is what keeps `workitems/changes/` from being read as that.
WORKITEMS_DIR_REL = ".aw/workitems"

# The closed work-item type enum. argparse consumes it as `choices`, so a
# plural typo like `--type changes` goes red at parse time instead of quietly
# creating a directory nothing ever reads.
# `epic` remains only so legacy staged bodies and read probes still resolve.
# New delivery writes use one registry delivery type. `spike` and `report`
# remain intake types. Releases use `milestone.py` and are not issue work-item
# types.
WORK_ITEM_TYPES = (*wi_types.DELIVERY_TYPES, *wi_types.INTAKE_TYPES)
LEGACY_WORK_ITEM_TYPES = ("epic",)


# --------------------------------------------------------------------------
# Section schema
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Section:
    """One required H2 in a work-item body.

    `subsections` are H3s that must appear under it. `rules` are (predicate,
    message) pairs run against the section's own text, so a rule can never
    accidentally read a neighbouring section's content.

    That isolation is deliberate and is kept: a check that genuinely spans two
    sections goes in `WorkItemType.cross_rules`, where the sections it reads
    are declared rather than reached for.

    `template` is the starter content `skeleton()` writes under the section,
    and `tight` drops the blank line between the guidance comment and it. Both
    are data on the section because the alternative -- `skeleton()` testing
    headings by name -- puts one type's section names inside the generic
    builder, which is exactly the coupling this module exists to remove.
    """

    heading: str
    guidance: str
    subsections: tuple[str, ...] = ()
    rules: tuple[tuple, ...] = ()
    template: tuple[str, ...] = ()
    tight: bool = False


@dataclass(frozen=True)
class WorkItemType:
    """One work-item type on the axis. Adding a type is adding an entry here.

    `prog` is the script name this type is reached through, and it appears
    verbatim in refusal messages -- a message telling the caller to rerun
    `epic.py` when they ran something else is a wrong instruction, not a
    cosmetic difference. It carries no default on purpose: a default would be
    one type's name sitting in the engine, which is the exact thing this
    module exists to not contain, and `check_engine_split.py` refuses it.

    `phase_label` does default, because `phase:created` is where every type
    starts -- it names a lifecycle position, not a type.

    `validate` and `skeleton_text` exist because not every type's schema is
    owned here. A type whose rules this plugin invented supplies `sections` and
    lets the declarative walk below enforce them. A type whose rules are owned
    by the `aw` crate supplies a port of that owner instead, and a copy of the
    empty body that owner hands out -- encoding those rules a third time, as
    section data, would produce a reading that matches neither the owner's
    structure nor its output, and could not be compared to either.

    Which route a type takes is the type's business. The engine only has to
    not care, which is why both hooks are plain callables and neither names a
    type.
    """

    name: str
    type_label: str
    prog: str
    sections: tuple[Section, ...]
    cross_rules: tuple = ()
    phase_label: str = "phase:created"
    validate: Callable[[str], list[str]] | None = None
    skeleton_text: str | None = None


# --------------------------------------------------------------------------
# Body parsing and validation
# --------------------------------------------------------------------------


def row_cells(line: str) -> list[str]:
    """One pipe row's cells, honouring markdown's `\\|` escape.

    A gate command inside a table cell spells its shell pipe `\\|`; a naive
    `split("|")` breaks that cell in two and every column after it reads its
    neighbour's value. Split on unescaped pipes only, then unescape, so the
    cell comes back carrying the `|` the author wrote.
    """
    stripped = line.strip().strip("|")
    return [c.strip().replace("\\|", "|") for c in re.split(r"(?<!\\)\|", stripped)]


def split_sections(body: str) -> dict[str, str]:
    """Map each H2 heading to its own text, excluding following H2s."""
    sections: dict[str, str] = {}
    current: str | None = None
    buffer: list[str] = []
    for line in body.splitlines():
        match = re.match(r"^##\s+(.+?)\s*$", line)
        if match and not line.startswith("###"):
            if current is not None:
                sections[current] = "\n".join(buffer).strip()
            current = match.group(1).strip()
            buffer = []
            continue
        if current is not None:
            buffer.append(line)
    if current is not None:
        sections[current] = "\n".join(buffer).strip()
    return sections


def validate_body(body: str, wi_type: WorkItemType) -> list[str]:
    """Return every reason this body is not a valid work-item of `wi_type`."""
    if wi_type.validate is not None:
        return wi_type.validate(body)

    errors: list[str] = []
    if not body.strip():
        return [f"body is empty; a {wi_type.name} needs every section in "
                f"`{wi_type.prog} skeleton`"]

    found = split_sections(body)
    for section in wi_type.sections:
        text = found.get(section.heading)
        if text is None:
            errors.append(f"`## {section.heading}` is missing -- {section.guidance}")
            continue
        if not text.strip():
            errors.append(f"`## {section.heading}` is empty -- {section.guidance}")
            continue
        for sub in section.subsections:
            if not re.search(rf"^###\s+{re.escape(sub)}\s*$", text, re.M):
                errors.append(f"`## {section.heading}` is missing its `### {sub}` subsection")
        for predicate, message in section.rules:
            if not predicate(text):
                errors.append(f"`## {section.heading}` {message}")

    for cross_rule in wi_type.cross_rules:
        errors.extend(cross_rule(found))

    known = {section.heading for section in wi_type.sections}
    # Unknown H2s are reported, not refused: real bodies carry optional
    # sections, such as an epic's `## Child Work Items`.
    for heading in found:
        if heading not in known:
            errors.append(f"note: `## {heading}` is not a schema section (allowed, not required)")
    return errors


def skeleton(wi_type: WorkItemType) -> str:
    if wi_type.skeleton_text is not None:
        return wi_type.skeleton_text

    lines: list[str] = []
    for section in wi_type.sections:
        lines.append(f"## {section.heading}")
        lines.append("")
        lines.append(f"<!-- {section.guidance} -->")
        if not section.tight:
            lines.append("")
        for sub in section.subsections:
            lines.append(f"### {sub}")
            lines.append("")
            lines.append("")
        lines.extend(section.template)
    return "\n".join(lines).rstrip() + "\n"


# --------------------------------------------------------------------------
# GitHub access
# --------------------------------------------------------------------------


class GhError(RuntimeError):
    pass


def default_repo() -> str:
    """Read the issue platform out of the repository's own aw.toml."""
    if not AW_TOML.is_file():
        raise GhError(f"{AW_TOML} not found; pass --repo explicitly")
    text = AW_TOML.read_text(encoding="utf-8")
    block = re.search(r"\[agentic_workflow\.issue_platform\](.*?)(?=\n\[|\Z)", text, re.S)
    if block:
        found = re.search(r'^\s*repo\s*=\s*"([^"]+)"', block.group(1), re.M)
        if found:
            return found.group(1)
    raise GhError("no [agentic_workflow.issue_platform] repo in aw.toml; pass --repo")


def gh(*args: str, check: bool = True) -> str:
    completed = subprocess.run(
        ["gh", *args], capture_output=True, text=True, check=False, cwd=REPO_ROOT
    )
    if check and completed.returncode != 0:
        raise GhError(f"gh {' '.join(args)} failed:\n{completed.stderr.strip()}")
    return completed.stdout


def fetch_issue(iid: str, repo: str) -> dict:
    raw = gh(
        "issue",
        "view",
        str(iid),
        "--repo",
        repo,
        "--json",
        "number,title,body,state,labels,url,milestone,updatedAt",
    )
    issue = json.loads(raw)
    issue["labels"] = [label["name"] for label in issue.get("labels", [])]
    issue["updated_at"] = issue.get("updatedAt")
    milestone = issue.get("milestone")
    if milestone:
        issue["milestone"] = {
            "number": milestone.get("number"),
            "title": milestone.get("title"),
            "state": milestone.get("state"),
            "url": milestone.get("url"),
        }
    return issue


def fetch_issues_by_label(label: str, repo: str) -> list[dict]:
    """Every issue carrying `label`, open or closed.

    What the label *means* is the caller's business: this returns the set, and
    reading `epic:<iid>` as an ownership claim is the epic type's rule, not
    a property of labels in general. REST pagination is required here because
    the former `gh issue list --limit 200` silently truncated G2 coverage.
    """
    raw = gh(
        "api",
        "--paginate",
        "--slurp",
        f"repos/{repo}/issues?state=all&labels={quote(label, safe='')}&per_page=100",
    )
    loaded = json.loads(raw)
    rows = ([row for page in loaded for row in page]
            if loaded and isinstance(loaded[0], list) else loaded)
    issues: list[dict] = []
    for issue in rows:
        if "pull_request" in issue:
            continue
        milestone = issue.get("milestone")
        issues.append({
            "number": issue["number"],
            "title": issue["title"],
            "state": issue["state"].upper(),
            "labels": [lbl["name"] for lbl in issue.get("labels", [])],
            "url": issue["html_url"],
            "milestone": None if not milestone else {
                "number": milestone.get("number"),
                "title": milestone.get("title"),
                "state": milestone.get("state"),
                "url": milestone.get("html_url") or milestone.get("url"),
            },
        })
    return issues


def require_type(issue: dict, verb: str, wi_type: WorkItemType) -> None:
    if wi_type.type_label in issue["labels"]:
        return
    actual = [lbl for lbl in issue["labels"] if lbl.startswith("type:")] or ["<untyped>"]
    raise GhError(
        f"work-item #{issue['number']} has type {actual[0]}; `{wi_type.prog} {verb}` accepts only "
        f"{wi_type.type_label}. The work-item type enum converges by spawn-and-link, never by "
        "changing a type in place."
    )


def require_delivery_type(issue: dict, verb: str) -> str:
    """Return the one live delivery type for an issue.

    The generic engine is the lowest common consumer of the registry.  It
    deliberately does not select a schema; the change facade maps this result
    to the shared GHAN schema after this guard succeeds.
    """
    try:
        return wi_types.delivery_type(issue.get("labels", []), subject=f"work-item #{issue['number']}")
    except wi_types.TypeError as exc:
        raise GhError(f"`change.py {verb}` refuses: {exc}") from exc


def reject_type_label_mutation(add: list[str] | None, remove: list[str] | None) -> None:
    """Normal update never changes the immutable executable type."""
    proposed = [
        label.strip()
        for value in (add or []) + (remove or [])
        for label in value.split(",")
        if label.strip().startswith(wi_types.TYPE_PREFIX)
    ]
    if proposed:
        raise GhError(
            "normal update cannot add, remove, or replace `type:*` labels; "
            "use `change.py retype` only before delivery starts"
        )


def replace_issue_labels(iid: str, repo: str, labels: list[str], dry_run: bool) -> str:
    """Set the complete issue label set with one tracker operation."""
    argv_prefix = ["api", "--method", "PUT", f"repos/{repo}/issues/{iid}/labels"]
    if dry_run:
        print("[dry-run] gh " + " ".join(argv_prefix) + " <complete-label-set>")
        return ""
    with tempfile.TemporaryDirectory() as scratch:
        payload = Path(scratch) / "labels.json"
        payload.write_text(json.dumps({"labels": labels}), encoding="utf-8")
        return gh(*argv_prefix, "--input", str(payload))


def refs_commits(iid: str) -> list[str]:
    """Commit ids whose own trailer records delivery evidence for this issue."""
    completed = subprocess.run(
        ["git", "-c", "core.fsmonitor=false", "log", "--format=%H",
         "--extended-regexp", f"--grep=^Refs #{iid}$"],
        capture_output=True, text=True, check=False, cwd=REPO_ROOT,
    )
    if completed.returncode != 0:
        raise GhError(f"cannot inspect delivery commits for #{iid}: {completed.stderr.strip()}")
    return [line for line in completed.stdout.splitlines() if line]


def commit_message(commit: str) -> str:
    """The exact message of one recorded delivery commit."""
    completed = subprocess.run(
        ["git", "-c", "core.fsmonitor=false", "show", "-s", "--format=%B", commit],
        capture_output=True, text=True, check=False, cwd=REPO_ROOT,
    )
    if completed.returncode != 0:
        raise GhError(
            f"cannot read delivery commit {commit}: {completed.stderr.strip()}"
        )
    return completed.stdout


def is_ancestor(commit: str, head: str = "HEAD") -> bool:
    """Whether `commit` is reachable from `head` in the local object graph.

    `merge-base --is-ancestor` answers rc 0/1 only when both sides name real
    local objects; anything else -- `commit` missing from the local store,
    `head` unresolvable, git itself broken -- exits with something else and no
    documented way to tell those apart from its own rc alone. The two cases
    resolve oppositely, though, once `head` is checked directly: if
    `head^{commit}` resolves locally, `head` is a real local commit and the
    only thing that could have made the pair fail is `commit` not existing in
    this store -- and an object this store does not have cannot be an
    ancestor of anything in it, since every ancestor of a local commit is
    itself local. That case is a routine False, not a refusal: it is exactly
    the shape of evidence a squash-merge landing proof exists to accept
    through the GitHub API instead. If `head` does not resolve either, nothing
    downstream of this call can be trusted, and the failure is raised rather
    than guessed at.
    """
    completed = subprocess.run(
        ["git", "-c", "core.fsmonitor=false", "merge-base", "--is-ancestor", commit, head],
        capture_output=True, text=True, check=False, cwd=REPO_ROOT,
    )
    if completed.returncode == 0:
        return True
    if completed.returncode == 1:
        return False
    head_resolved = subprocess.run(
        ["git", "-c", "core.fsmonitor=false", "rev-parse", "--verify", "--quiet",
         f"{head}^{{commit}}"],
        capture_output=True, text=True, check=False, cwd=REPO_ROOT,
    )
    if head_resolved.returncode == 0:
        return False
    raise GhError(
        f"cannot test whether {commit} is an ancestor of {head}: {completed.stderr.strip()}"
    )


def commit_message_via_api(commit: str, repo: str) -> str:
    """The exact message of `commit`, read from GitHub rather than the local tree.

    A commit whose object never reaches this checkout -- swept off a squashed
    branch before it was ever fetched here -- still has a message the tracker
    kept. `payload["sha"]` is checked against the request because GitHub
    resolves an abbreviated id to whichever full one it currently means; an
    exact readback is the only way to know the message answers the commit
    that was asked for.
    """
    raw = gh("api", f"repos/{repo}/commits/{commit}")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GhError(f"commits/{commit} on {repo} did not return JSON") from exc
    if payload.get("sha") != commit:
        raise GhError(
            f"commits/{commit} on {repo} resolved to a different sha ({payload.get('sha')!r})"
        )
    message = payload.get("commit", {}).get("message")
    if not isinstance(message, str) or not message:
        raise GhError(f"commits/{commit} on {repo} carries no commit message")
    return message


def default_branch(repo: str) -> str:
    """The branch a squash-merge landing proof must land on."""
    raw = gh("api", f"repos/{repo}")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GhError(f"repos/{repo} did not return JSON") from exc
    branch = payload.get("default_branch")
    if not branch:
        raise GhError(f"repos/{repo} carries no default_branch")
    return branch


def pulls_for_commit(commit: str, repo: str) -> list[dict]:
    """Every pull request GitHub associates with `commit` on `repo`."""
    raw = gh("api", f"repos/{repo}/commits/{commit}/pulls")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GhError(f"commits/{commit}/pulls on {repo} did not return JSON") from exc
    if not isinstance(payload, list):
        raise GhError(f"commits/{commit}/pulls on {repo} did not return a list")
    return payload


def pull_request(number: int, repo: str) -> dict:
    """The full pull request record for `number` on `repo`."""
    raw = gh("api", f"repos/{repo}/pulls/{number}")
    try:
        payload = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GhError(f"pulls/{number} on {repo} did not return JSON") from exc
    if not isinstance(payload, dict):
        raise GhError(f"pulls/{number} on {repo} did not return an object")
    return payload


def pull_request_commit_shas(number: int, repo: str) -> list[str]:
    """Every commit sha GitHub lists under pull request `number`, paginated.

    Mirrors `fetch_issues_by_label`'s page-flattening: `--paginate --slurp`
    hands back one list per page rather than one flat list once a PR carries
    more commits than a single page holds -- this landing proof's own PR
    carried 111.
    """
    raw = gh(
        "api", "--paginate", "--slurp",
        f"repos/{repo}/pulls/{number}/commits?per_page=100",
    )
    try:
        loaded = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise GhError(f"pulls/{number}/commits on {repo} did not return JSON") from exc
    rows = ([row for page in loaded for row in page]
            if loaded and isinstance(loaded[0], list) else loaded)
    shas: list[str] = []
    for row in rows:
        sha = row.get("sha") if isinstance(row, dict) else None
        if not sha:
            raise GhError(f"pulls/{number}/commits on {repo} returned a commit with no sha")
        shas.append(sha)
    return shas


@dataclass(frozen=True)
class LandingProof:
    """One commit's message, proven readable, and how it was proven to land."""

    commit: str
    message: str
    route: str


def landing_proof(commit: str, repo: str, head: str = "HEAD") -> LandingProof:
    """Prove `commit` landed on `head`'s history and return its exact message.

    Two routes, and only two. Route A is `is_ancestor`: `commit` sits directly
    in `head`'s ancestry, which is the common case for a checkout that has not
    been rebased since the evidence commit was made. Route B exists because a
    GitHub squash merge rewrites the PR's commits into one new commit on
    `head` and leaves the originals reachable from nothing -- `is_ancestor`
    then answers False for evidence that is genuinely landed. Route B proves
    the same fact a different way: exactly one pull request merged `commit`
    into `repo`'s default branch, that PR really is merged, `commit` is really
    among its commits, and the PR's own merge commit *is* an ancestor of
    `head`.

    What Route B never does is read the squash commit's message. That message
    is the concatenation of every commit message in the PR -- GitHub's own
    squash template -- so it can contain a `Refs #<iid>` line and a digest
    trailer that belong to a wholly different leg, or to no real evidence at
    all. The message this function returns always comes from `commit` itself,
    fetched locally or through `commit_message_via_api`; the merge commit is
    consulted only for its sha, never for its prose.
    """
    if not re.fullmatch(r"[0-9a-f]{40,64}", commit):
        raise GhError(f"{commit!r} is not a full lowercase-hex commit id")

    reasons: list[str] = []
    message: str | None = None
    try:
        message = commit_message(commit)
    except GhError as local_error:
        reasons.append(f"local read failed: {local_error}")
        try:
            message = commit_message_via_api(commit, repo)
        except GhError as api_error:
            reasons.append(f"API read failed: {api_error}")
    if message is None:
        raise GhError(f"cannot read the message of {commit}: " + "; ".join(reasons))

    if is_ancestor(commit, head):
        return LandingProof(commit, message, "ancestry")

    branch = default_branch(repo)
    raw_pulls = pulls_for_commit(commit, repo)
    candidates = [
        pull for pull in raw_pulls
        if pull.get("merged_at")
        and (pull.get("base") or {}).get("ref") == branch
        and ((pull.get("base") or {}).get("repo") or {}).get("full_name") == repo
    ]
    if len(candidates) != 1:
        raise GhError(
            f"{commit} is not an ancestor of {head}, and GitHub returns "
            f"{len(candidates)} of {len(raw_pulls)} pull request(s) merged into "
            f"{repo}:{branch} for it; landing proof needs exactly one"
        )
    number = candidates[0].get("number")
    if not isinstance(number, int):
        raise GhError(f"{commit}'s candidate pull request carries no integer number")

    pr = pull_request(number, repo)
    if pr.get("merged") is not True:
        raise GhError(f"PR #{number} on {repo} is not merged")
    if (pr.get("base") or {}).get("ref") != branch:
        raise GhError(f"PR #{number} on {repo} does not target {branch}")
    if ((pr.get("base") or {}).get("repo") or {}).get("full_name") != repo:
        raise GhError(f"PR #{number} does not target {repo}")
    merge = pr.get("merge_commit_sha")
    if not isinstance(merge, str) or not re.fullmatch(r"[0-9a-f]{40,64}", merge):
        raise GhError(f"PR #{number} on {repo} carries no valid merge_commit_sha")

    if commit not in pull_request_commit_shas(number, repo):
        raise GhError(f"PR #{number} commits do not include {commit}")
    if not is_ancestor(merge, head):
        raise GhError(f"merge commit {merge} of PR #{number} is not an ancestor of {head}")

    return LandingProof(commit, message, f"squash-merged PR #{number} ({merge})")


def project_label(project: str) -> str:
    """Accept a bare project name or an already-qualified label."""
    if ":" in project:
        return project
    if (REPO_ROOT / "apps" / project).is_dir():
        return f"app:{project}"
    if (REPO_ROOT / "libs" / project).is_dir():
        return f"lib:{project}"
    return f"project:{project}"


def run_or_show(argv: list[str], dry_run: bool) -> str:
    printable = " ".join(["gh", *argv])
    if dry_run:
        print(f"[dry-run] {printable}")
        return ""
    return gh(*argv)


# --------------------------------------------------------------------------
# Staging paths
# --------------------------------------------------------------------------


def staging_dir(wi_type: str, create: bool = True) -> Path:
    """The directory bodies of `wi_type` are staged in.

    One owner for the location. Directories are created lazily, so a type
    nobody has staged for leaves no empty directory behind.
    """
    # Delivery phases consume a staged body before they can read tracker state.
    # They need one stable location because the phase scripts receive an iid,
    # not a type selector. Intake and legacy facades retain their own folders.
    leaf = "deliveries" if wi_type in wi_types.DELIVERY_TYPES else f"{wi_type}s"
    directory = REPO_ROOT / WORKITEMS_DIR_REL / leaf
    if create:
        directory.mkdir(parents=True, exist_ok=True)
    return directory


def in_staging_tree(path: Path) -> bool:
    """Whether `path` sits under any type's staging directory."""
    root = (REPO_ROOT / WORKITEMS_DIR_REL).resolve()
    try:
        path.resolve().relative_to(root)
    except ValueError:
        return False
    return True


def issue_number_from_create_output(out: str) -> str | None:
    """Pull the new issue number out of what `gh issue create` printed.

    `gh` prints the created issue's URL and nothing else, so the trailing path
    segment is the number. Returning None rather than raising keeps a `gh`
    output change from turning a landed write into a failure -- the write
    already happened by the time this is called, and the only thing lost is
    the rename.
    """
    found = re.search(r"/issues/(\d+)\s*$", out.strip())
    return found.group(1) if found else None


def rename_to_iid(path: Path, iid: str) -> tuple[bool, str]:
    """Rename a staged body to `<iid>.md` beside itself.

    Returns (renamed, message). Refuses anything outside the staging tree: a
    `--body-file` may point at tracked source, and this script must never
    rename a file the caller did not stage.
    """
    if not in_staging_tree(path):
        return False, (
            f"{path} is outside {WORKITEMS_DIR_REL}/, so it was left alone -- "
            "only staged bodies are renamed"
        )
    target = path.with_name(f"{iid}.md")
    if target == path:
        return False, f"{path} is already named for #{iid}"
    if target.exists():
        return False, f"refusing to overwrite the existing {target}"
    path.rename(target)
    return True, str(target)


# --------------------------------------------------------------------------
# Type-independent verbs
#
# Each reads `args.wi_type`, bound by the facade's `main()` before dispatch.
# --------------------------------------------------------------------------


def cmd_skeleton(args) -> int:
    sys.stdout.write(skeleton(args.wi_type))
    return 0


def cmd_bodydir(args) -> int:
    """Print the absolute directory bodies of a type are staged in, creating it.

    A body file is a transient input to one tracker write, so it belongs in
    the checkout's gitignored runtime area rather than in tracked source. The
    directory is repo-anchored but `--body-file` resolves against the *current*
    directory, so the caller needs the absolute path -- printing it here keeps
    one owner for the location instead of asking every caller to rebuild it.
    """
    print(staging_dir(args.type))
    return 0


def cmd_fetch(args) -> int:
    """Write the tracker's current body to `<staging>/<type>s/<iid>.md`.

    The start of the update path, and it overwrites unconditionally. A local
    body carries no authority: if it differs from the tracker, the only
    reading is that it is stale, and editing a stale copy is how a body
    written elsewhere gets silently reverted.

    Deliberately not gated on `validate_body`. Fetching exists precisely to
    pull a malformed work-item down and fix it -- validating here would lock
    exactly the bodies that need repair out of the tool that repairs them, and
    that population is not hypothetical: six live epics fail the epic schema.
    """
    issue = fetch_issue(args.iid, args.repo)
    require_type(issue, "fetch", args.wi_type)
    path = staging_dir(args.wi_type.name) / f"{issue['number']}.md"
    existed = path.exists()
    path.write_text(issue.get("body") or "", encoding="utf-8")
    print(path)
    if existed:
        print(f"(overwrote the previous local copy of #{issue['number']})", file=sys.stderr)
    return 0


def cmd_adopt(args) -> int:
    """Rename a staged body to `<iid>.md` once the tracker has assigned one.

    `create` does this for itself. This verb exists for work items opened
    through `gh issue create` directly rather than through this script -- the
    child work items `/aw-grill-milestone-to-issue` opens -- where nothing else is
    positioned to complete the rename.
    """
    path = Path(args.path)
    if not path.is_file():
        raise GhError(f"{path} does not exist")
    if not in_staging_tree(path):
        print(
            f"error: {path} is outside {WORKITEMS_DIR_REL}/; `adopt` renames staged "
            "bodies only",
            file=sys.stderr,
        )
        return 1
    renamed, message = rename_to_iid(path, str(args.iid))
    if renamed:
        print(message)
        return 0
    # Already correctly named is the idempotent case, not a failure; a
    # collision with a different file is.
    if message.endswith(f"already named for #{args.iid}"):
        print(message)
        return 0
    print(f"error: {message}", file=sys.stderr)
    return 1


def cmd_validate(args) -> int:
    if args.body_file:
        body = Path(args.body_file).read_text(encoding="utf-8")
        subject = args.body_file
    elif args.iid:
        issue = fetch_issue(args.iid, args.repo)
        require_type(issue, "validate", args.wi_type)
        body = issue.get("body") or ""
        subject = f"#{issue['number']}"
    else:
        raise GhError("validate needs an <iid> or --body-file")

    errors = validate_body(body, args.wi_type)
    hard = [e for e in errors if not e.startswith("note:")]
    notes = [e for e in errors if e.startswith("note:")]
    if args.json:
        print(json.dumps({"subject": subject, "valid": not hard, "errors": hard, "notes": notes},
                         indent=2))
    else:
        for note in notes:
            print(f"  {note}")
        if hard:
            print(f"{subject}: INVALID ({len(hard)} error(s))")
            for error in hard:
                print(f"  - {error}")
        else:
            print(f"{subject}: valid")
    return 1 if hard else 0


def cmd_create(args) -> int:
    body = Path(args.body_file).read_text(encoding="utf-8")
    errors = [e for e in validate_body(body, args.wi_type) if not e.startswith("note:")]
    if errors:
        print(f"refusing to create: the body fails {len(errors)} schema rule(s)")
        for error in errors:
            print(f"  - {error}")
        return 1

    labels = [args.wi_type.type_label, args.wi_type.phase_label, f"priority:{args.priority}"]
    if args.project:
        labels.append(project_label(args.project))
    # Whatever else the facade decided this write carries. Ownership links are
    # the case that exists today, and what a given label *means* is the
    # facade's business -- the engine only has to put it on the write.
    labels += getattr(args, "extra_labels", None) or []
    argv = [
        "issue", "create",
        "--repo", args.repo,
        "--title", args.title,
        "--body-file", str(Path(args.body_file).resolve()),
    ]
    for label in labels:
        argv += ["--label", label]
    milestone_title = getattr(args, "milestone_title", None)
    if milestone_title:
        argv += ["--milestone", milestone_title]

    out = run_or_show(argv, args.dry_run)
    if args.dry_run:
        return 0
    print(out.strip())

    # The staged body was named for a slug because the number did not exist
    # yet. It does now, and `gh` already handed it back in `out`, so no second
    # round-trip is needed. This runs only past the dry-run return above: a
    # dry run that renamed would leave an `<id>.md` naming an issue nobody
    # opened.
    number = issue_number_from_create_output(out)
    args.created_iid = number
    if number:
        renamed, message = rename_to_iid(Path(args.body_file), number)
        print(f"staged body -> {message}" if renamed else f"staged body: {message}")
    return 0


def cmd_update(args) -> int:
    issue = fetch_issue(args.iid, args.repo)
    require_type(issue, "update", args.wi_type)

    if args.body_file:
        body = Path(args.body_file).read_text(encoding="utf-8")
        errors = [e for e in validate_body(body, args.wi_type) if not e.startswith("note:")]
        if errors:
            print(f"refusing to update: the body fails {len(errors)} schema rule(s)")
            for error in errors:
                print(f"  - {error}")
            return 1

    argv = ["issue", "edit", str(args.iid), "--repo", args.repo]
    if args.title:
        argv += ["--title", args.title]
    if args.body_file:
        argv += ["--body-file", str(Path(args.body_file).resolve())]
    for label in args.add_label or []:
        argv += ["--add-label", label]
    for label in args.remove_label or []:
        argv += ["--remove-label", label]
    milestone_title = getattr(args, "milestone_title", None)
    if milestone_title:
        argv += ["--milestone", milestone_title]
    if getattr(args, "remove_milestone", False):
        argv += ["--remove-milestone"]
    if len(argv) == 5:
        raise GhError("update needs at least one of --body-file, --title, --add-label, "
                      "--remove-label, --milestone, or --remove-milestone")

    out = run_or_show(argv, args.dry_run)
    if not args.dry_run:
        print(out.strip() or f"updated #{args.iid}")
    return 0


# The lifecycle block, and why it is shaped the way it is.
#
# A work item's body is authored prose under four H2s, and that set is closed:
# `validate_body` refuses any other H2 by name. So the legs cannot record
# themselves as a `## Lifecycle` section -- doing that would make the *next*
# leg's work-item precondition fail on the body its own predecessor wrote.
# Measured, not assumed: an appended `## Lifecycle` is INVALID, an appended
# HTML comment and an appended horizontal rule are both valid.
#
# What lands instead is a block fenced by two HTML comments, appended after the
# last authored section. The fence is what makes the write an *upsert*: a
# re-run of a leg, or a later leg, rewrites the block rather than stacking a
# second copy under it. Without a fence the only available operation is append,
# and append plus retry is how a body accumulates three contradictory records
# of the same leg.
#
# The rule is inside the fence and readable on the tracker rather than only
# here, because the next person to see it will be reading a GitHub issue, not
# this file.
LIFECYCLE_BEGIN = "<!-- aw:lifecycle:begin -->"
LIFECYCLE_END = "<!-- aw:lifecycle:end -->"
LIFECYCLE_NOTE = ("*Written by `aw` as each leg lands. Not authored content, and not "
                  "reviewed: edit the sections above, never this block.*")
# The ladder whose legs may appear as rows, in the order they are rendered.
#
# This read `("ec", "td", "cb")` for one commit past the changeover that deleted
# those three scripts, which made every phase's closing step unreachable: every
# phase script ends by printing `change.py lifecycle ... --leg <PHASE>`, and
# `change.py` takes its `--leg` choices from here, so each printed a command
# this parser exits 2 on. Eighteen gates were green over it, because no gate
# compared a printed command with the parser receiving it.
# `check_next_command.py` is now that comparison.
#
# The retired names are dropped rather than kept alongside. Keeping them would
# preserve rows in bodies that predate the changeover, and the population of
# those was measured before deciding: zero issues on the tracker carry the
# lifecycle marker at all, because the verb that writes it has never once
# succeeded. There is no history here to lose.
#
# `unit` and `logic` were folded into one `impl` leg on 2026-08-27, for the same
# reason and with the same measurement: nothing on the tracker carried either
# row. The two were one phase in Rust -- a colocated test and the code under it
# are the same tree, edited together -- and what the split bought (a named red
# measured before the green) is bought instead by `impl.py`'s `red` verb, which
# records the failing names mid-phase.
LEGS = ("e2e", "impl", "maint")


def lifecycle_rows(body: str) -> dict:
    """The legs already recorded in `body`, keyed by leg name."""
    start = body.find(LIFECYCLE_BEGIN)
    end = body.find(LIFECYCLE_END)
    if start == -1 or end == -1:
        return {}
    rows = {}
    for line in body[start:end].splitlines():
        cells = row_cells(line)
        if len(cells) == 3 and cells[0] in LEGS:
            rows[cells[0]] = cells
    return rows


def lifecycle_errors(body: str, required: tuple[str, ...]) -> list[str]:
    """Reject malformed lifecycle evidence before it can authorize closure."""
    start = body.find(LIFECYCLE_BEGIN)
    end = body.find(LIFECYCLE_END)
    if start == -1 and end == -1:
        return [f"missing {leg} lifecycle evidence" for leg in required]
    if start == -1 or end == -1 or end < start:
        return ["lifecycle marker block is malformed"]
    seen: set[str] = set()
    errors: list[str] = []
    for line in body[start:end].splitlines():
        if not line.strip().startswith("|"):
            continue
        cells = row_cells(line)
        if cells in (["leg", "commit", "digest"], ["---", "---", "---"]):
            continue
        if len(cells) != 3 or cells[0] not in LEGS:
            errors.append("lifecycle table has an invalid row")
            continue
        if cells[0] not in required:
            errors.append(
                f"lifecycle table records {cells[0]} outside this issue's required flow"
            )
        if cells[0] in seen:
            errors.append(f"lifecycle table records {cells[0]} more than once")
        seen.add(cells[0])
        if not re.fullmatch(r"`[0-9a-f]{40,64}`", cells[1]):
            errors.append(f"lifecycle {cells[0]} has an invalid full commit id")
        if not re.fullmatch(r"`[0-9a-f]{64}`", cells[2]):
            errors.append(f"lifecycle {cells[0]} has a missing or invalid digest")
    errors.extend(f"missing {leg} lifecycle evidence" for leg in required if leg not in seen)
    return errors


def has_lifecycle_evidence(body: str) -> bool:
    """Any lifecycle marker blocks the pre-delivery retype escape hatch."""
    return LIFECYCLE_BEGIN in body or LIFECYCLE_END in body


def lifecycle_upsert(body: str, leg: str, commit: str, digest: str) -> str:
    """`body` with `leg`'s row set to `commit`/`digest`, block created if absent.

    Ordering is by `LEGS`, not by arrival, so the block reads as the lifecycle
    rather than as a log. A leg that lands twice occupies one row either way.
    """
    rows = lifecycle_rows(body)
    rows[leg] = [leg, f"`{commit}`", f"`{digest}`" if digest else "--"]

    # "digest" rather than "change digest": this is the type-agnostic engine,
    # and `check_engine_split.py` reads a work-item type name in a literal here
    # as the engine having learned about one specific type. It is right to --
    # the column is the same column whatever type the item is.
    table = ["| leg | commit | digest |", "|---|---|---|"]
    table += [f"| {' | '.join(rows[name])} |" for name in LEGS if name in rows]
    block = "\n".join([LIFECYCLE_BEGIN, "", "---", "", LIFECYCLE_NOTE, "",
                       *table, "", LIFECYCLE_END])

    start = body.find(LIFECYCLE_BEGIN)
    if start == -1:
        return body.rstrip("\n") + "\n\n" + block + "\n"
    end = body.find(LIFECYCLE_END) + len(LIFECYCLE_END)
    return body[:start] + block + body[end:]


def cmd_lifecycle(args) -> int:
    """Record a landed leg in the work item's body.

    The body is fetched live rather than read from the staged copy. A leg lands
    at the end of a session that started by staging the body, and between those
    two moments the tracker is the only thing that knows whether someone else
    wrote to it. Rewriting a stale local copy back over the issue would silently
    revert them.

    The result is validated *before* it is pushed, and that check is the whole
    safety argument for writing to the body at all: it is the same function the
    next leg's precondition runs, so a body this verb would refuse can never
    reach the tracker.
    """
    issue = fetch_issue(args.iid, args.repo)
    require_type(issue, "lifecycle", args.wi_type)

    def hard(text: str) -> list[str]:
        return [e for e in validate_body(text, args.wi_type) if not e.startswith("note:")]

    original = issue["body"] or ""
    body = lifecycle_upsert(original, args.leg, args.commit, args.digest)

    # Two refusals, and they are kept apart because they have different fixes
    # and only one of them is this verb's fault. A body that was already
    # malformed cannot be recorded onto, but saying "the result fails" there
    # would point at the append -- sending whoever reads it to look for a bug
    # in a block that is doing exactly what it should.
    before, after = hard(original), hard(body)
    if before:
        print(f"refusing to record {args.leg}: #{args.iid} already fails "
              f"{len(before)} schema rule(s) before this leg touches it")
        for error in before:
            print(f"  - {error}")
        print("fix the work item first; this verb does not repair a body")
        return 1
    if after:
        print(f"refusing to record {args.leg}: the append would break a body that "
              f"validates today, on {len(after)} rule(s)")
        for error in after:
            print(f"  - {error}")
        return 1

    # Through a file rather than `--body-file -` or `--body`: the body is
    # markdown with newlines, backticks and pipes in it, and every route that
    # puts it on a command line is a quoting bug waiting for the first work
    # item whose prose contains the wrong character.
    with tempfile.TemporaryDirectory() as scratch:
        staged = Path(scratch) / f"{args.iid}.md"
        staged.write_text(body, encoding="utf-8")
        argv = ["issue", "edit", str(args.iid), "--repo", args.repo,
                "--body-file", str(staged)]
        out = run_or_show(argv, args.dry_run)
    if not args.dry_run:
        print(out.strip() or f"recorded {args.leg} on #{args.iid}: {args.commit}")
    return 0


def dispatch(args, wi_type: WorkItemType | None, local_verbs: tuple[str, ...]) -> int:
    """Resolve the tracker repo, bind the type, and run the parsed verb.

    `local_verbs` are the verbs that never reach the tracker, so a missing
    `[agentic_workflow.issue_platform]` must not stop them: they are pure
    local output, a local rename, or have a file mode.
    """
    if wi_type is not None:
        args.wi_type = wi_type
    if not getattr(args, "repo", None):
        try:
            args.repo = default_repo()
        except GhError as error:
            if args.verb not in local_verbs:
                print(f"error: {error}", file=sys.stderr)
                return 2
            args.repo = None
    try:
        return args.func(args)
    except GhError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
