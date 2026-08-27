#!/usr/bin/env python3
"""Refuse a PRD run that reached outside `docs/product/`, then commit it.

`/aw-grill-me-to-prd` interviews a human and writes prose. Prose is exactly
what no exit code can judge, so this script judges the two things around it
that an exit code *can*: where the run wrote, and what the commit says about
it.

**Where it wrote.** The skill's `## Never` list already says "never write
outside `docs/product/` of the named project". A `Never` list is a sentence an
agent reads once. `check` is the same claim as a working-tree measurement: the
dirty set against HEAD, every path of it, against one allowlist. A PRD run that
edited `STATUS.md`, `ROADMAP.md`, the project README, or a `src/**` file is
refused here whether or not anyone read the list.

**What the commit says.** The next skill in the ladder --
`/aw-grill-prd-to-wi`, which turns a written promise into an epic or a change
-- has to find the PRD commits and know what each one touched. It cannot get
that from a hand-written subject line. So `commit` is the only writer here: it
re-runs every check, stages exactly the allowlist, and appends a trailer block
naming the project, every section this commit added, modified, or removed, and
how many of those are still unbound. That makes the history searchable:

    git log --grep='^PRD-Project: apps/tape'

## What it cannot do

It cannot tell a good promise from a bad one. Every check here is structural:
a path is inside a directory or it is not, a bullet is present or it is not, an
id occurs in `STATUS.md` or it does not. Whether the Promise is worth making,
whether the `Open:` questions are the right ones, whether the Problem is real
-- none of that is measurable from the file, and this script does not pretend
otherwise. That judgement stays with the human the skill interviews.

It also cannot prove the run was *authored* by the skill rather than typed by
hand. The allowlist refuses the writes the skill forbids; it says nothing about
who made the writes it permits.

It never writes a document. `commit` writes a commit; nothing here edits a
Markdown file, and a check that could repair what it measures would be a check
that can hide what it measures.
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

# `leg.py` already owns the git plumbing every phase script uses -- the
# fsmonitor-disabling prefix, the outermost-`aw.toml` root, and the
# rename-aware dirty set. Loaded rather than copied: a second implementation of
# `dirty_set` that disagreed about renames would disagree silently, and this
# script's whole job is to be exact about which paths changed.
sys.path.insert(0, str(Path(__file__).resolve().parent))
import leg  # noqa: E402

# The one directory a PRD run may write. Not a prefix list: the point of the
# check is that there is exactly one, and every near miss -- `STATUS.md`,
# `ROADMAP.md`, the project `README.md`, `docs/technical/`, `src/**` -- sits
# outside it. The project README was considered and left out: the areas stand
# on its capability ids, so a run that edits both is a run that moved the
# ground it is standing on in the same breath.
PRODUCT = "docs/product"
INDEX = "README.md"
FOOTER = "Non-goals in this area"

# One section, two shapes, and the owner bullet is what says which. A future
# promise carries `Open:` -- the decisions its epic will have to settle. A
# shipped one cannot: shipping settled them. What it carries instead, when it
# has one, is `Limits today:` -- what the promise does *not* do yet, which is a
# different claim from an unanswered question and is where the next section's
# `Problem:` comes from.
#
# Both shapes are measured rather than declared. This started as a single
# seven-bullet rule read off the skill's prose, and the first run against a
# real PRD refused eleven of tape's twenty-three sections -- every shipped one.
# The corpus carries exactly four bullet sequences: the twelve `Outcome:`
# sections are `FUTURE`, and the eleven `Status rows:` sections are `SHIPPED`,
# six of them with `Limits today:` and five without. A gate whose declaration
# is narrower than the corpus it must accept is a gate that refuses the prior
# art it was written from.
FUTURE = ("Problem", "Who", "Promise", "Non-goals", "Open", "Neighbours")
SHIPPED = ("Problem", "Who", "Promise", "Limits today", "Non-goals",
           "Neighbours")

# `Limits today:` is the one bullet a shape may omit. It is not optional in the
# sense of unimportant -- five shipped sections have no limits left worth
# naming, and inventing one to satisfy a gate is exactly the false answer this
# script exists to refuse.
OPTIONAL = ("Limits today",)

# The owner bullet, and the shape it selects. A section carries exactly one:
# a shipped promise names the STATUS rows that measure it, a future promise
# names the one ROADMAP outcome that will.
OWNERS = {"Status rows": SHIPPED, "Outcome": FUTURE}

# Every bullet either shape knows about. A key in here but not in *this*
# section's shape is a bullet borrowed from the other kind -- an `Open:` on a
# shipped promise, a `Limits today:` on one that has not shipped -- which is a
# section that has not decided which kind it is.
KNOWN = frozenset(FUTURE) | frozenset(SHIPPED)

# `Promise, for now:` is the shipped-and-leaving form: the section records a
# surface that is public today and going away, so the promise is scoped to the
# present tense rather than withdrawn. Enumerated rather than parsed as a
# grammar -- a general `<Key>, <words>:` rule would start reading ordinary
# prose bullets as schema, and `- Non-goals: Google-signed OIDC tokens, ...`
# is one comma away from being misread.
QUALIFIED = {"Promise, for now": "Promise"}

# The product-document contract, which owns the shape of STATUS, ROADMAP and
# the READMEs these sections point at. It is not re-implemented here; it is
# run. It reads TOML, `tomllib` is 3.11+, and `python3` is 3.9 on at least one
# machine this runs on -- so it gets the pinned launcher regardless of what
# interpreter this script was started under.
VALIDATOR = "scripts/meta/project_docs_contract.py"
PINNED = ("uv", "run", "--python", "3.13", "--no-project")

SECTION = re.compile(r"^##[ \t]+(?P<title>.+?)[ \t]*$", re.M)
IID = re.compile(r"\s*\(#(\d+)\)$")
BULLET = re.compile(r"^-[ \t]+(?P<key>Promise, for now|[A-Z][A-Za-z -]*?)"
                    r":[ \t]*(?P<rest>.*)$")
BACKTICKED = re.compile(r"`([^`]+)`")
TRAILER = re.compile(r"^[A-Za-z][A-Za-z-]*: .+$")
TRACKING_LINK = re.compile(r"Tracking:[ \t]*\[#\d+\]")


@dataclass(frozen=True)
class Finding:
    """One refusal, named by the check that made it."""

    rule: str
    path: str
    message: str

    def as_dict(self) -> dict:
        return {"rule": self.rule, "path": self.path, "message": self.message}


CHECKS = {
    "P1": "the project carries STATUS.md, ROADMAP.md and docs/product/",
    "P2": "every changed path is under <project>/docs/product/",
    "P3": "something under docs/product/ actually changed",
    "P4": "no heading or Tracking: line gained an issue number",
    "P5": "every touched section carries its own kind's bullets, in order",
    "P6": "an Outcome: bullet keeps its Tracking: on the same line",
    "P7": "every STATUS row id and ROADMAP outcome id resolves",
    "P8": "the section index and the touched area files agree",
    "P9": "every touched area file ends with its non-goals",
    "P10": "the product-document contract still passes",
}


def usage_error(message: str) -> None:
    raise SystemExit(f"error: {message}")


def resolve_project(repo: Path, given: str) -> str:
    """A repo-relative `apps/<name>` or `libs/<name>` from what a human typed.

    A bare name is accepted because that is what the skill's own prose uses,
    but it is resolved against the two roots rather than guessed: a name that
    exists under both is refused instead of silently picking one.
    """
    given = given.strip().strip("/")
    if "/" in given:
        if (repo / given).is_dir():
            return given
        usage_error(f"no such project directory: {given}")
    found = [f"{root}/{given}" for root in ("apps", "libs")
             if (repo / root / given).is_dir()]
    if not found:
        usage_error(f"no `apps/{given}` or `libs/{given}` in {repo}")
    if len(found) > 1:
        usage_error(f"`{given}` is ambiguous: {', '.join(found)}; name the root")
    return found[0]


def sections(text: str) -> list[tuple[str, str]]:
    """Every `## ` section of a document as `(raw title, body)`.

    The body runs to the next `## ` or to the end. Nothing above the first
    heading is a section -- that is the file's own H1 and the paragraph naming
    its README capability ids.
    """
    marks = list(SECTION.finditer(text))
    out = []
    for i, m in enumerate(marks):
        end = marks[i + 1].start() if i + 1 < len(marks) else len(text)
        out.append((m.group("title"), text[m.end():end]))
    return out


def bare(title: str) -> str:
    """A section title with its ` (#<iid>)` binding removed."""
    return IID.sub("", title).strip()


def bullets(body: str) -> list[tuple[str, str]]:
    """The section's top-level bullets, each joined across its soft wraps.

    Joined because the parts that matter are routinely wrapped: `Status rows:`
    runs to three lines in tape's own PRD, and reading only the first line
    would find a third of its ids.
    """
    out: list[tuple[str, str]] = []
    for line in body.splitlines():
        m = BULLET.match(line)
        if m:
            key = m.group("key")
            out.append((QUALIFIED.get(key, key), m.group("rest")))
        elif out and line.startswith(("  ", "\t")) and line.strip():
            key, rest = out[-1]
            out[-1] = (key, f"{rest} {line.strip()}")
        elif not line.strip():
            continue
    return out


def index_rows(text: str) -> list[tuple[str, str]]:
    """`(section title, file)` for every row of `## Section index`."""
    rows = []
    inside = False
    for line in text.splitlines():
        if line.startswith("## "):
            inside = line[3:].strip() == "Section index"
            continue
        if not inside or not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 2 or set(cells[0]) <= set("- ") or cells[0] == "Section":
            continue
        rows.append((cells[0], cells[1]))
    return rows


def git_show(repo: Path, rel: str) -> str:
    """The file as HEAD has it, or empty if HEAD does not have it."""
    proc = subprocess.run([*leg.GIT, "show", f"HEAD:{rel}"],
                          cwd=repo, capture_output=True, text=True)
    return proc.stdout if proc.returncode == 0 else ""


def pinned_launcher() -> list[str]:
    """A 3.11+ interpreter for the validator, resolved once per process."""
    found = subprocess.run(["uv", "python", "find", "3.13"],
                           capture_output=True, text=True)
    if found.returncode == 0 and found.stdout.strip():
        return [found.stdout.strip()]
    return list(PINNED)


def collect(repo: Path, project: str) -> tuple[list[Finding], dict]:
    """Every finding, and the population each check was measured over."""
    out: list[Finding] = []
    root = repo / project
    product = f"{project}/{PRODUCT}"

    # -- P1: the ground the sections stand on ------------------------------
    for name in ("STATUS.md", "ROADMAP.md"):
        if not (root / name).is_file():
            out.append(Finding("P1", f"{project}/{name}", "missing; a promise is "
                               "owned by a STATUS row or a ROADMAP outcome, and "
                               "this file is where the id has to resolve"))
    if not (repo / product).is_dir():
        out.append(Finding("P1", product, "missing; run /aw-grill-me-to-prd in "
                           "create mode before this check can mean anything"))
        return out, {"project": project, "changed": [], "sections": 0}

    # -- P2/P3: the write allowlist ----------------------------------------
    dirty = leg.dirty_set(repo)
    changed = [p for p in dirty if p.startswith(f"{product}/")]
    for path in dirty:
        if path not in changed:
            out.append(Finding("P2", path, "outside "
                               f"`{product}/`; a PRD run writes nowhere else"))
    if not changed:
        out.append(Finding("P3", product, "nothing changed here; there is no "
                           "PRD run to check or commit"))

    # Only files that still exist are read. A deleted area file is a change
    # this script has to account for -- P8 does, through the index rows that
    # still point at it -- but it has no body left to measure.
    touched = [p for p in changed
               if (repo / p).is_file() and p != f"{product}/{INDEX}"]
    texts = {p: (repo / p).read_text(encoding="utf-8") for p in touched}
    status = (root / "STATUS.md").read_text(encoding="utf-8") if (root / "STATUS.md").is_file() else ""
    roadmap = (root / "ROADMAP.md").read_text(encoding="utf-8") if (root / "ROADMAP.md").is_file() else ""

    counted = 0
    for path, text in sorted(texts.items()):
        parsed = sections(text)

        # -- P4: binding is the epic grill's, not this one's ---------------
        # Measured as a delta against HEAD rather than as an absolute. A file
        # that already carries a bound section is the normal case -- the whole
        # point of a brownfield PRD run -- and refusing it outright would make
        # this check unusable on every project past its first epic.
        before = git_show(repo, path)
        gained_bound = ({bare(t) for t, _ in parsed if IID.search(t)}
                        - {bare(t) for t, _ in sections(before) if IID.search(t)})
        for title in sorted(gained_bound):
            out.append(Finding("P4", path, f"section `{title}` gained a `(#<iid>)` "
                               "binding; that is /aw-grill-me-to-epic's write, "
                               "made in the same run that opens the epic"))
        gained_links = (len(TRACKING_LINK.findall(text))
                        - len(TRACKING_LINK.findall(before)))
        if gained_links > 0:
            out.append(Finding("P4", path, f"{gained_links} `Tracking: [#<iid>]` "
                               "link(s) appeared; a section this skill writes "
                               "ends `Tracking: not assigned.`"))

        # -- P5/P6/P7: the section shape -----------------------------------
        for raw, body in parsed:
            title = bare(raw)
            if title == FOOTER:
                continue
            counted += 1
            keys = [k for k, _ in bullets(body)]
            owners = [k for k in keys if k in OWNERS]
            if len(owners) != 1:
                # No owner, or two, means there is no shape to measure the rest
                # against -- so this is the whole finding for the section, and
                # reporting the missing bullets of a shape it may not have is
                # noise about a section whose kind is the actual defect.
                out.append(Finding("P5", path, f"section `{title}` carries "
                                   f"{len(owners)} owner bullet(s) "
                                   f"({', '.join(owners) or 'none'}); it takes "
                                   "exactly one of `Status rows:` or `Outcome:`"))
                continue
            shape = OWNERS[owners[0]]
            missing = [b for b in shape if b not in keys and b not in OPTIONAL]
            if missing:
                out.append(Finding("P5", path, f"section `{title}` is missing "
                                   f"bullet(s) {', '.join(missing)}"))
            foreign = [k for k in keys if k in KNOWN and k not in shape]
            if foreign:
                out.append(Finding("P5", path, f"section `{title}` ends "
                                   f"`{owners[0]}:` but carries "
                                   f"{', '.join(foreign)}, which belongs to the "
                                   "other kind of section"))
            ordered = [k for k in keys if k in shape]
            if ordered != [b for b in shape if b in ordered]:
                out.append(Finding("P5", path, f"section `{title}` writes its "
                                   f"bullets out of order: {', '.join(ordered)}"))
            for key, rest in bullets(body):
                if key == "Outcome":
                    # The one-line rule is not cosmetic: /aw-grill-me-to-epic
                    # finds the line with `grep` when it binds the section, and
                    # a soft wrap hides it from the bind.
                    raw_line = next((line for line in body.splitlines()
                                     if line.startswith("- Outcome:")), "")
                    if "Tracking:" not in raw_line:
                        out.append(Finding("P6", path, f"section `{title}` wraps "
                                           "its `Outcome:` bullet before "
                                           "`Tracking:`; the bind greps one line"))
                    ids = BACKTICKED.findall(rest)
                    for oid in ids[:1]:
                        if f"`{oid}`" not in roadmap and f": `{oid}`" not in roadmap \
                                and oid not in roadmap:
                            out.append(Finding("P7", path, f"section `{title}` "
                                               f"claims ROADMAP outcome `{oid}`, "
                                               "which does not occur in "
                                               f"{project}/ROADMAP.md"))
                if key == "Status rows":
                    for sid in BACKTICKED.findall(rest):
                        if sid not in status:
                            out.append(Finding("P7", path, f"section `{title}` "
                                               f"claims STATUS row `{sid}`, which "
                                               "does not occur in "
                                               f"{project}/STATUS.md"))

        # -- P9: the footer ------------------------------------------------
        if not parsed or bare(parsed[-1][0]) != FOOTER:
            out.append(Finding("P9", path, f"does not end with `## {FOOTER}`; "
                               "non-goals are not sections, they are the file's "
                               "last heading"))

    # -- P8: the index and the area files agree ----------------------------
    index_path = f"{product}/{INDEX}"
    if not (repo / index_path).is_file():
        if changed:
            out.append(Finding("P8", index_path, "missing; every area file is "
                               "reached through the section index"))
    else:
        rows = index_rows((repo / index_path).read_text(encoding="utf-8"))
        for title, filename in rows:
            target = f"{product}/{filename}"
            if not (repo / target).is_file():
                out.append(Finding("P8", index_path, f"row `{title}` points at "
                                   f"`{filename}`, which is not on disk; a "
                                   "deleted area leaves the index in the same "
                                   "edit"))
            elif target in texts:
                titles = {bare(t) for t, _ in sections(texts[target])}
                if title not in titles:
                    out.append(Finding("P8", index_path, f"row `{title}` names no "
                                       f"section in `{filename}`"))
        for path, text in sorted(texts.items()):
            filename = path.rsplit("/", 1)[1]
            indexed = {t for t, f in rows if f == filename}
            for raw, _ in sections(text):
                title = bare(raw)
                if title != FOOTER and title not in indexed:
                    out.append(Finding("P8", path, f"section `{title}` has no row "
                                       "in the section index"))

    # -- P10: the contract that owns the documents around these ------------
    validator = repo / VALIDATOR
    if not validator.is_file():
        out.append(Finding("P10", VALIDATOR, "missing; the product-document "
                           "contract is what /project-readme-check runs"))
    else:
        proc = subprocess.run([*pinned_launcher(), str(validator), "check",
                               project, "--format", "json"],
                              cwd=repo, capture_output=True, text=True)
        try:
            reports = json.loads(proc.stdout)["reports"]
            ok = bool(reports) and all(r.get("ok") for r in reports)
        except (ValueError, KeyError, TypeError):
            ok = False
            reports = []
        if not ok:
            detail = "; ".join(str(f) for r in reports for f in r.get("findings", []))
            out.append(Finding("P10", project, "the product-document contract "
                               "refuses this project: "
                               + (detail or proc.stderr.strip() or "no report")))

    out.sort(key=lambda f: (f.rule, f.path, f.message))
    return out, {"project": project, "changed": changed, "sections": counted}


def section_modes(repo: Path, changed: list[str], product: str) -> list[str]:
    """`<mode> <path>#<title>` for every section this run added or moved.

    Modes are derived by diffing HEAD against the working tree per file, not
    declared by the caller: a trailer the agent writes by hand is a trailer
    that can disagree with the diff it claims to describe.
    """
    lines = []
    for path in sorted(changed):
        if path.endswith(f"/{INDEX}"):
            continue
        before = {bare(t): b for t, b in sections(git_show(repo, path))}
        after = ({bare(t): b for t, b in
                  sections((repo / path).read_text(encoding="utf-8"))}
                 if (repo / path).is_file() else {})
        for title in sorted(set(after) - set(before)):
            lines.append(f"added {path}#{title}")
        for title in sorted(set(before) - set(after)):
            lines.append(f"removed {path}#{title}")
        for title in sorted(set(before) & set(after)):
            if before[title] != after[title]:
                lines.append(f"modified {path}#{title}")
    return lines


def unbound_count(repo: Path, changed: list[str]) -> int:
    """Touched sections still carrying no `(#<iid>)`.

    What `/aw-grill-prd-to-wi` reads to know how much of this commit is still
    waiting for a work item.
    """
    total = 0
    for path in changed:
        if path.endswith(f"/{INDEX}") or not (repo / path).is_file():
            continue
        for raw, _ in sections((repo / path).read_text(encoding="utf-8")):
            if bare(raw) != FOOTER and not IID.search(raw):
                total += 1
    return total


def report(findings: list[Finding], population: dict, fmt: str,
           next_command: str) -> int:
    if fmt == "json":
        print(json.dumps({"population": population,
                          "checks": CHECKS,
                          "findings": [f.as_dict() for f in findings]}, indent=2))
        return 1 if findings else 0

    print(f"PRD check: {population['project']}, "
          f"{len(population['changed'])} changed path(s), "
          f"{population['sections']} section(s) read")
    counts = {rule: sum(1 for f in findings if f.rule == rule) for rule in CHECKS}
    for rule, why in CHECKS.items():
        print(f"  {rule:<4} {counts[rule]:>3}  {why}")

    current = ""
    for finding in findings:
        if finding.path != current:
            current = finding.path
            print(f"\n{current}")
        print(f"  {finding.rule:<4} {finding.message}")

    if not findings:
        print("\n=> CLEAN")
        print(f"next.command: {next_command}")
        return 0
    print(f"\n=> {len(findings)} finding(s) in "
          f"{len({f.path for f in findings})} path(s)")
    print("next.command: fix the paths above, then re-run this verb")
    return 1


def cmd_check(args: argparse.Namespace) -> int:
    repo = leg.repo_root()
    project = resolve_project(repo, args.project)
    findings, population = collect(repo, project)
    launcher = " ".join(PINNED)
    return report(findings, population, args.format,
                  f'{launcher} ".claude/aw/scripts/prd.py" commit {project} '
                  '--why <path>')


def cmd_commit(args: argparse.Namespace) -> int:
    """Re-run every check, then write the one commit the run is allowed."""
    repo = leg.repo_root()
    project = resolve_project(repo, args.project)
    product = f"{project}/{PRODUCT}"
    name = project.rsplit("/", 1)[1]

    why = Path(args.why)
    if not why.is_file():
        usage_error(f"no such --why file: {why}")
    lines = why.read_text(encoding="utf-8").strip().splitlines()
    if not lines:
        usage_error(f"--why file is empty: {why}")
    subject, body = lines[0].strip(), [line.rstrip() for line in lines[1:]]
    # The subject is the human's sentence, not a generated one -- a generated
    # subject would say what the trailers already say and nothing a reader
    # cannot get from the diff. What is checked is that it is addressed to this
    # project and short enough to survive `git log --oneline`.
    if not subject.startswith(f"docs({name}): "):
        usage_error(f"--why subject must start with `docs({name}): `; got {subject!r}")
    if len(subject) > 72:
        usage_error(f"--why subject is {len(subject)} chars; keep it under 72")

    findings, population = collect(repo, project)
    if findings:
        report(findings, population, "text", "")
        print("\nrefusing to commit: fix the findings above")
        return 1

    changed = population["changed"]
    # Trailers the human already wrote (a `Co-Authored-By:` line, typically)
    # are carried through and re-emitted after the generated block, because git
    # reads trailers only as one contiguous run at the end of the message.
    carried = []
    while body and TRAILER.match(body[-1]):
        carried.insert(0, body.pop())
    while body and not body[-1].strip():
        body.pop()

    trailers = [f"PRD-Project: {project}"]
    if f"{product}/{INDEX}" in changed:
        trailers.append("PRD-Index: modified")
    trailers += [f"PRD-Section: {line}"
                 for line in section_modes(repo, changed, product)]
    trailers.append(f"PRD-Unbound: {unbound_count(repo, changed)}")
    trailers += carried

    message = "\n".join([subject, "", *body, "", *trailers]).replace("\n\n\n", "\n\n")

    print(f"paths: {' '.join(changed)}")
    print("---")
    print(message)
    print("---")
    if args.dry_run:
        print("=> DRY RUN; nothing staged, nothing committed")
        return 0

    add = subprocess.run([*leg.GIT, "add", "--", *changed],
                         cwd=repo, capture_output=True, text=True)
    if add.returncode != 0:
        raise SystemExit(add.stderr.strip() or "git add failed")
    done = subprocess.run([*leg.GIT, "commit", "-F", "-"], cwd=repo,
                          input=message, capture_output=True, text=True)
    if done.returncode != 0:
        raise SystemExit(done.stderr.strip() or done.stdout.strip() or "git commit failed")
    sha = subprocess.run([*leg.GIT, "rev-parse", "--short", "HEAD"],
                         cwd=repo, capture_output=True, text=True).stdout.strip()
    print(f"=> committed {sha}")
    print(f"next.command: git log --grep='^PRD-Project: {project}'")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="prd.py",
                                     description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="verb", required=True)

    # Two verbs, and the split is the point. `check` reads and never writes, so
    # it can be run at any moment of a PRD run without changing what the run
    # is. `commit` writes exactly one thing -- a commit -- and re-runs `check`
    # first, so the two cannot disagree about what landed.
    p = sub.add_parser("check", help="refuse a PRD run that wrote outside "
                                     "docs/product/ or left a section unshaped")
    p.add_argument("project", help="`apps/<name>`, `libs/<name>`, or a bare name")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.set_defaults(func=cmd_check)

    p = sub.add_parser("commit", help="check, then write the searchable PRD commit")
    p.add_argument("project", help="`apps/<name>`, `libs/<name>`, or a bare name")
    p.add_argument("--why", required=True, metavar="PATH",
                   help="file whose first line is the subject `docs(<name>): ...` "
                        "and whose rest is the body")
    p.add_argument("--dry-run", action="store_true",
                   help="print the message and the pathspec, commit nothing")
    p.set_defaults(func=cmd_commit)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
