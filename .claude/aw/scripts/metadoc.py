#!/usr/bin/env python3
"""Refuse a META-doc run that reached outside the project's own documents.

`/aw-grill-me-to-meta` interviews a human and writes prose. Prose is exactly
what no exit code can judge, so this script judges the two things around it
that an exit code *can*: where the run wrote, and what the commit says about
it.

**Where it wrote.** The skill's `## Never` list already says "never write
outside the named project's documents". A `Never` list is a sentence an agent
reads once. `check` is the same claim as a working-tree measurement: the dirty
set against HEAD, every path of it, against one allowlist of four entries --
`README.md`, `STATUS.md`, `ROADMAP.md`, and everything under `docs/`. A run
that edited a `src/**` file, another project, or the repository root is refused
here whether or not anyone read the list.

Three of those four were deliberately *excluded* until 2026-08-27, and the
reversal is worth stating rather than quietly inheriting. The reasoning behind
the exclusion was real: the `docs/` sections stand on ids that live in
`STATUS.md` and `ROADMAP.md` and on capability names in the project `README.md`,
so a run that edits both is a run that moved the ground it is standing on in
the same breath. What made the exclusion affordable was a second skill,
`/aw-check-meta`, that a human remembered to run afterwards.

The exclusion is gone because the split was the wrong shape: a promise and the
STATUS row that measures it are one edit, and forcing them into two runs by two
skills meant the second one was the one that got skipped. What replaces it is
not trust. `P7` still resolves every id the sections claim, and `P10` still runs
the product-document contract -- both against the *edited* working tree, so
moving the ground is measured rather than assumed. The check that used to be a
separate skill is now a step in this one's landing sequence, where it cannot be
forgotten:

    metadoc.py check <project>     # this script: allowlist, shape, ids
    meta.py check <project>        # M1-M7 over the edited tree
    metadoc.py commit <project> --why <path>

**What the commit says.** The next skill in the ladder --
`/aw-grill-meta-to-wis`, which measures the promises against the tracker and
the codebase -- has to find these commits and know what each one touched. It
cannot get that from a hand-written subject line. So `commit` is the only
writer here: it re-runs every check, stages exactly the allowlist, and appends
a trailer block naming the project, every section this commit added, modified,
or removed, and how many of those are still unbound. That makes the history
searchable:

    git log --grep='^Meta-Project: apps/tape'

## What it cannot do

It cannot tell a good promise from a bad one. Every check here is structural:
a path is inside the allowlist or it is not, a bullet is present or it is not,
an id occurs in `STATUS.md` or it does not. Whether the Promise is worth
making, whether the `Open:` questions are the right ones, whether the Problem
is real -- none of that is measurable from the file, and this script does not
pretend otherwise. That judgement stays with the human the skill interviews.

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

# The four things a META-doc run may write, relative to the project root.
# Three files and one tree, and the near misses that stay outside are the ones
# that were never this skill's: `src/**`, `e2e/**`, `Cargo.toml`, another
# project's documents, and anything at the repository root.
#
# `docs/` rather than `docs/product/`: the sections are the project's product
# detail wherever they sit, and pinning one subdirectory would make the
# allowlist a statement about a filing convention rather than about ownership.
# `docs/product/` remains a perfectly ordinary `docs/**` path -- the seven area
# files already under it were not moved, because moving them would be a rename
# with no reader asking for it.
AREAS = "docs"
TOP = ("README.md", "STATUS.md", "ROADMAP.md")

# The index is a `README.md` beside the area files it indexes, so `P8` reads
# the one in each touched directory rather than a single hardcoded path. That
# is what lets `docs/` hold more than one family of areas without either
# family's index claiming the other's sections.
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
LEGACY_IID = re.compile(r"\s*\(#(\d+)\)$")
MILESTONE = re.compile(r"\s*\(Milestone[ \t]+#(\d+)\)$")
BINDING = re.compile(r"(?:\s*\(#\d+\)|\s*\(Milestone[ \t]+#\d+\))$")
BULLET = re.compile(r"^-[ \t]+(?P<key>Promise, for now|[A-Z][A-Za-z -]*?)"
                    r":[ \t]*(?P<rest>.*)$")
BACKTICKED = re.compile(r"`([^`]+)`")
TRAILER = re.compile(r"^[A-Za-z][A-Za-z-]*: .+$")
TRACKING_LINK = re.compile(r"Tracking:[ \t]*\[(?:Milestone[ \t]+#|#)\d+\]")


@dataclass(frozen=True)
class Finding:
    """One refusal, named by the check that made it."""

    rule: str
    path: str
    message: str

    def as_dict(self) -> dict:
        return {"rule": self.rule, "path": self.path, "message": self.message}


CHECKS = {
    "P1": "the project carries README.md, STATUS.md, ROADMAP.md and docs/",
    "P2": "every changed path is one of the project's four document paths",
    "P3": "one of those four actually changed",
    "P4": "no heading or Tracking: line gained a tracker binding",
    "P5": "every touched section carries its own kind's bullets, in order",
    "P6": "an Outcome: bullet keeps its Tracking: on the same line",
    "P7": "every STATUS row id and ROADMAP outcome id resolves",
    "P8": "each directory's index and its touched area files agree",
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
    """A section title with its current or legacy tracker binding removed."""
    return BINDING.sub("", title).strip()


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


def in_scope(project: str, path: str) -> bool:
    """Whether one repo-relative path is inside this project's allowlist."""
    return (path in {f"{project}/{name}" for name in TOP}
            or path.startswith(f"{project}/{AREAS}/"))


def is_area(repo: Path, project: str, path: str) -> bool:
    """Whether a path is an area file -- a `docs/**` section document.

    Three exclusions, and the third is the one that had to be added.

    A directory's own `README.md` is its index, not an area, and the three
    top-level documents are not under `docs/` at all. Both of those matter for
    the same reason: an index measured as an area has no owner bullet and would
    trip `P5` on every run, and a project README measured as one would trip it
    on every heading it owns.

    The third is that **a directory with no index holds no areas**. Until
    2026-08-27 the schema's scope was `docs/product/` and this could not
    arise; widening it to `docs/**` swept in 61 pre-existing reference
    documents across seven projects -- jet's `docs/bundler.md`, mamba's
    `docs/blockers/*`, lumen's `docs/protocol.md`, tape's own
    `docs/deployment-handoff.md` -- none of which is a promise. Measured as
    areas they made `wis.py`'s `G1` row permanently `?` UNMEASURED for every
    one of those projects, because a section with no owner bullet has no kind.

    So the index is the registry. `/aw-grill-me-to-meta` already writes one
    `README.md` per `docs/` directory and refuses a section indexed in the
    wrong directory's; this makes that file the thing that declares its
    directory to hold promises at all. A reference document needs no marker
    and no move -- it just sits in a directory that never got an index.

    The hazard the rule creates is the mirror of the one it fixes: deleting an
    index silently demotes every area beside it, and `P5` stops refusing them.
    That is why `collect()` reports the reference count beside the area count
    rather than dropping the files quietly -- the population line moves when an
    index goes.
    """
    if not (path.startswith(f"{project}/{AREAS}/") and path.endswith(".md")):
        return False
    if path.endswith(f"/{INDEX}"):
        return False
    return ((repo / path).parent / INDEX).is_file()


def area_population(repo: Path, project: str) -> tuple[int, int]:
    """How many `docs/**` files are areas, and how many are unindexed reference.

    Printed rather than returned to a caller that could ignore it: the second
    number is what moves when an index is deleted, and a demotion nobody sees
    is a rule that stopped applying without failing.
    """
    root = repo / project / AREAS
    if not root.is_dir():
        return 0, 0
    areas = reference = 0
    for path in sorted(root.rglob("*.md")):
        rel = path.relative_to(repo).as_posix()
        if rel.endswith(f"/{INDEX}"):
            continue
        if is_area(repo, project, rel):
            areas += 1
        else:
            reference += 1
    return areas, reference


def collect(repo: Path, project: str) -> tuple[list[Finding], dict]:
    """Every finding, and the population each check was measured over."""
    out: list[Finding] = []
    root = repo / project
    areas = f"{project}/{AREAS}"

    # -- P1: the ground the sections stand on ------------------------------
    for name in TOP:
        if not (root / name).is_file():
            out.append(Finding("P1", f"{project}/{name}", "missing; a promise is "
                               "owned by a STATUS row or a ROADMAP outcome and "
                               "names a README capability, and these files are "
                               "where those ids have to resolve"))
    if not (repo / areas).is_dir():
        out.append(Finding("P1", areas, "missing; run /aw-grill-me-to-meta in "
                           "create mode before this check can mean anything"))
        return out, {"project": project, "changed": [], "sections": 0,
                      "areas": 0, "reference": 0}

    # -- P2/P3: the write allowlist ----------------------------------------
    dirty = leg.dirty_set(repo)
    changed = [p for p in dirty if in_scope(project, p)]
    for path in dirty:
        if path not in changed:
            out.append(Finding("P2", path, "outside this project's documents; a "
                               f"META-doc run writes `{project}/README.md`, "
                               f"`{project}/STATUS.md`, `{project}/ROADMAP.md` "
                               f"and `{areas}/` and nowhere else"))
    if not changed:
        out.append(Finding("P3", project, "none of the four document paths "
                           "changed; there is no META-doc run to check or commit"))

    # Only area files are read for shape. The three top-level documents are in
    # the allowlist but have a contract of their own -- `P10` runs it, and
    # `meta.py check` runs the rest of it in the landing sequence -- so reading
    # them for `## <title>` bullets here would refuse every heading they own.
    #
    # Only files that still exist are read. A deleted area file is a change
    # this script has to account for -- P8 does, through the index rows that
    # still point at it -- but it has no body left to measure.
    touched = [p for p in changed if is_area(repo, project, p) and (repo / p).is_file()]
    texts = {p: (repo / p).read_text(encoding="utf-8") for p in touched}
    status = (root / "STATUS.md").read_text(encoding="utf-8") if (root / "STATUS.md").is_file() else ""
    roadmap = (root / "ROADMAP.md").read_text(encoding="utf-8") if (root / "ROADMAP.md").is_file() else ""

    counted = 0
    for path, text in sorted(texts.items()):
        parsed = sections(text)

        # -- P4: binding is the tracker grill's, not this one's ------------
        # Measured as a delta against HEAD rather than as an absolute. A file
        # that already carries a bound section is the normal case -- the whole
        # point of a brownfield META-doc run -- and refusing it outright would
        # make this check unusable on every project past its first epic.
        before = git_show(repo, path)
        gained_bound = ({bare(t) for t, _ in parsed if BINDING.search(t)}
                        - {bare(t) for t, _ in sections(before) if BINDING.search(t)})
        for title in sorted(gained_bound):
            out.append(Finding("P4", path, f"section `{title}` gained a tracker "
                               "binding; that is /aw-grill-meta-to-wis's write, "
                               "made in the same run that opens the milestone"))
        gained_links = (len(TRACKING_LINK.findall(text))
                        - len(TRACKING_LINK.findall(before)))
        if gained_links > 0:
            out.append(Finding("P4", path, f"{gained_links} `Tracking:` tracker "
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
                    # The one-line rule is not cosmetic: /aw-grill-meta-to-wis
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

    # -- P8: each directory's index and its area files agree ---------------
    # Per directory rather than per project. A single index would have to name
    # sections from families it does not own, which makes "this section has no
    # row" unfalsifiable -- there would always be some other index it could
    # have belonged to.
    #
    # The directories are taken from the *changed* paths, not from a walk of
    # `docs/`: an index whose own area files were untouched is not part of this
    # run, and reading it would report a defect nobody in this run introduced.
    for folder in sorted({p.rsplit("/", 1)[0] for p in changed
                          if p.startswith(f"{areas}/")}):
        index_path = f"{folder}/{INDEX}"
        if not (repo / index_path).is_file():
            out.append(Finding("P8", index_path, "missing; every area file is "
                               "reached through the section index beside it"))
            continue
        rows = index_rows((repo / index_path).read_text(encoding="utf-8"))
        for title, filename in rows:
            target = f"{folder}/{filename}"
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
            if path.rsplit("/", 1)[0] != folder:
                continue
            filename = path.rsplit("/", 1)[1]
            indexed = {t for t, f in rows if f == filename}
            for raw, _ in sections(text):
                title = bare(raw)
                if title != FOOTER and title not in indexed:
                    out.append(Finding("P8", path, f"section `{title}` has no row "
                                       f"in `{folder}/{INDEX}`"))

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
    areas, reference = area_population(repo, project)
    return out, {"project": project, "changed": changed,
                 "sections": counted, "areas": areas,
                 "reference": reference}


def section_modes(repo: Path, project: str, changed: list[str]) -> list[str]:
    """`<mode> <path>#<title>` for every section this run added or moved.

    Modes are derived by diffing HEAD against the working tree per file, not
    declared by the caller: a trailer the agent writes by hand is a trailer
    that can disagree with the diff it claims to describe.

    Scoped to the area files. The three top-level documents changed in the same
    run are reported by `Meta-Top:` instead: their headings are not sections in
    this schema's sense, and listing them here would put `## Capabilities` in a
    trailer that the next skill reads as a promise.
    """
    lines = []
    for path in sorted(changed):
        if not is_area(repo, project, path):
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


def unbound_count(repo: Path, project: str, changed: list[str]) -> int:
    """Touched sections still carrying no Milestone binding.

    What `/aw-grill-meta-to-wis` reads to know how much of this commit is still
    waiting for a work item -- and what its `unbound promise` gap row starts
    from.
    """
    total = 0
    for path in changed:
        if not is_area(repo, project, path) or not (repo / path).is_file():
            continue
        for raw, _ in sections((repo / path).read_text(encoding="utf-8")):
            if bare(raw) != FOOTER and not MILESTONE.search(raw):
                total += 1
    return total


def report(findings: list[Finding], population: dict, fmt: str,
           next_command: str) -> int:
    if fmt == "json":
        print(json.dumps({"population": population,
                          "checks": CHECKS,
                          "findings": [f.as_dict() for f in findings]}, indent=2))
        return 1 if findings else 0

    print(f"META-doc check: {population['project']}, "
          f"{len(population['changed'])} changed path(s), "
          f"{population['sections']} section(s) read")
    print(f"  docs/: {population.get('areas', 0)} indexed area file(s), "
          f"{population.get('reference', 0)} unindexed reference file(s) "
          f"not measured as promises")
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
    # The next command is `meta.py check`, not this script's own `commit`. The
    # landing sequence has three steps and this is the first: naming `commit`
    # here would print a command that skips the one the merged `/aw-check-meta`
    # skill used to be.
    return report(findings, population, args.format,
                  f'{launcher} ".claude/aw/scripts/meta.py" check {project}'
                  f' && {launcher} ".claude/aw/scripts/metadoc.py" commit '
                  f'{project} --why <path>')


def cmd_commit(args: argparse.Namespace) -> int:
    """Re-run every check, then write the one commit the run is allowed."""
    repo = leg.repo_root()
    project = resolve_project(repo, args.project)
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

    # `Meta-` and not `PRD-`. The prefix is renamed rather than kept because
    # the subject changed: these commits now carry README, STATUS and ROADMAP
    # edits too, and a `PRD-Section:` trailer on a commit that moved a STATUS
    # row would be describing something the name denies. Nothing in the history
    # is orphaned by the rename -- at the changeover
    # `git log --grep='^PRD-Project:'` matched zero commits.
    trailers = [f"Meta-Project: {project}"]
    for top in TOP:
        if f"{project}/{top}" in changed:
            trailers.append(f"Meta-Top: {top}")
    for path in sorted(changed):
        if path.endswith(f"/{INDEX}") and path.startswith(f"{project}/{AREAS}/"):
            trailers.append(f"Meta-Index: {path}")
    trailers += [f"Meta-Section: {line}"
                 for line in section_modes(repo, project, changed)]
    trailers.append(f"Meta-Unbound: {unbound_count(repo, project, changed)}")
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
    print(f"next.command: git log --grep='^Meta-Project: {project}'")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="metadoc.py",
                                     description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="verb", required=True)

    # Two verbs, and the split is the point. `check` reads and never writes, so
    # it can be run at any moment of a META-doc run without changing what the
    # run is. `commit` writes exactly one thing -- a commit -- and re-runs
    # `check` first, so the two cannot disagree about what landed.
    p = sub.add_parser("check", help="refuse a META-doc run that wrote outside "
                                     "the project's four document paths, or "
                                     "left a section unshaped")
    p.add_argument("project", help="`apps/<name>`, `libs/<name>`, or a bare name")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.set_defaults(func=cmd_check)

    p = sub.add_parser("commit",
                       help="check, then write the searchable META-doc commit")
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
