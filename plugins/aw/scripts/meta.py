#!/usr/bin/env python3
"""Refuse a META-doc that has rotted.

Three files carry everything this repository states about itself in prose:
`CLAUDE.md` is what an agent loads at launch, `README.md` is what a thing
promises, and `CONTRIBUTING.md` is how to change it. Until this script, nothing
read any of them.

Something used to. `aw meta` spliced generated content between
`<!-- aw:meta:...:start -->` and `:end`, and that verb is gone with the crate
that carried it -- leaving the markers on disk, 132 of them across 65 files,
each one telling a reader that a producer regenerates what sits inside it. A
marker whose producer does not exist is worse than plain prose, because plain
prose does not claim to be maintained.

That is the class this refuses: a doc fact whose owner is gone. The rule it
serves is the repository's own -- a fact carries a generator, a validator, or
an explicit policy-only marker, and a fact with none of the three rots
silently. This is the validator for the part of that which resolves against the
filesystem: a marker, a command, a path, a required heading. Every rule here
can be decided by looking, which is why there is no model in the loop and no
`AskUserQuestion` in the skill that calls it.

What it cannot do
-----------------
It cannot tell whether a promise is true, and it cannot run a gate command to
find out whether the promise under it is verified. Of the 725 backticked spans
in these files, 605 resolve to no path in the checkout and 333 begin `cargo `;
deciding whether one of those names a real test target means resolving crate
and target names through `cargo metadata`, which is a different and much larger
job. A `## Capabilities` entry whose gate command nobody runs is caught here by
a reader, not by a check, and the root `CONTRIBUTING.md` says so.

It writes nothing. That is the whole of its difference from the `aw meta` it
replaces, and it is why the verb is not reused: a checker that can also repair
is a checker whose green means "it agreed with itself".
"""
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

# `git` here disables fsmonitor for the reason `leg.py` gives: a stalled
# fsmonitor daemon blocks every command that reads the index, indefinitely and
# with no error, and `ls-files` reads the index.
GIT = ("git", "-c", "core.fsmonitor=false")

# The three META-docs, and there is no fourth. `<project>/CAPABILITIES.md` was
# the fourth until 2026-08-17, deleted for precisely the defect this refuses:
# 62 files, 57 of them the same empty template, and no mechanical reader
# anywhere in the repository.
#
# `AGENTS.md` is deliberately outside the population. It is not documentation --
# `codex exec` loads it as its instructions, so it is a production input with a
# reader of its own, and a rule that edits it changes what a reviewer is told.
# It carries 0 markers and 0 `aw <verb>` spans today, so admitting it would
# change no result while making a review input answerable to a doc rule.
META_DOCS = ("CLAUDE.md", "README.md", "CONTRIBUTING.md")

RULES = {
    "M1": "an `aw:meta:*` marker whose producer does not exist",
    "M2": "an `aw <verb>` command naming a CLI that was deleted",
    "M3": "a relative link whose target is not in the checkout",
    "M4": "a project README missing a required section",
}

# --------------------------------------------------------------------------
# M1 -- the markers, and who is supposed to write them
# --------------------------------------------------------------------------
MARKER = re.compile(r"<!--\s*aw:meta:([a-z0-9:_-]+):(start|end)\s*-->")

# Empty, and the emptiness is the measurement rather than a stub. Every marker
# in the repository is orphaned *by derivation* from this table -- nothing here
# hardcodes "all 132 are dead". The day something regenerates one of these
# blocks again, its name goes here and the rule stops firing on it; until then
# there is no producer, so there is no entry.
PRODUCERS: dict[str, str] = {}

# --------------------------------------------------------------------------
# M2 -- commands for a binary that no longer exists
# --------------------------------------------------------------------------
# The same detector `check_plugin.py` runs over the skill bodies, deliberately
# byte-identical: two populations, one defect. `check_meta_flow.py` asserts the
# two patterns have not drifted, because nothing else compares them.
AW_INVOCATION = re.compile(r"`aw\s+[a-z]")
AW_SPAN = re.compile(r"`(aw\s+[a-z][^`]*)`")

# The backtick is what makes a command a command in prose -- and it is exactly
# what a fenced block does not have. `apps/jet/README.md:63-64` is a ```bash
# block holding two bare `aw` lines, which is the most copy-and-run shape in the
# repository and the one the pattern above cannot see. So a second pattern, and
# it is applied only inside a fence: outside one, a line beginning "aw " is an
# English sentence far more often than a command.
AW_BARE = re.compile(r"^\s*aw\s+[a-z]")

# Prose that *names* the dead verb in order to say it is dead. Deleting these
# would delete the record of the deletion, which is the fact a reader most
# needs when they find `aw wi ...` in an old transcript and it fails.
#
# An exemption is a claim, so it is verified like one: each fragment below must
# match some line in that file, and one that stops matching is itself a finding.
# Without that, an exemption outlives the sentence it was written for and
# silently blinds the rule to whatever replaces it.
#
# Two near misses are deliberately *not* here.
# `plugins/aw/verification/README.md:315` says the change schema is "enforced by
# `aw wi validate`" in the present tense -- that is the rot, not a record of it.
# `apps/mamba/CONTRIBUTING.md:58` was a live instruction in a checklist table
# telling a human to run `aw wi create`; exempting it would have left a reader
# following a command that cannot run. It now names `/aw:wi-change-grill` and
# `change.py create`, which is the shape a forward-looking instruction has to
# take -- unlike a past-tense evidence row, which must not be repointed at a
# live command, because that fabricates a measurement nobody took.
DEAD_COMMAND_EXEMPT: dict[str, tuple[str, ...]] = {
    "CLAUDE.md": (
        'a stray `aw wi …` now fails with "command not found"',
    ),
    "CONTRIBUTING.md": (
        "It was `aw review --project <project>`, spliced",
        "and `aw meta init` / `sync` / `check`",
    ),
    "plugins/aw/verification/README.md": (
        "`aw wi validate` enforced it, and",
    ),
}

# --------------------------------------------------------------------------
# M3 -- links
# --------------------------------------------------------------------------
LINK = re.compile(r"\[[^\]]*\]\(\s*([^)\s]+?)\s*(?:\s+\"[^\"]*\")?\)")
SCHEME = re.compile(r"^[a-zA-Z][a-zA-Z0-9+.-]*:")

# An inline code span is the same exemption a fence is, at a smaller scale: no
# renderer turns `[a](b)` inside backticks into a link, so neither does this.
# The delimiter is a *run* of backticks closed by a run of the same length,
# which is how a document quotes backticks -- ```` ```bash ```` two lines up
# from here is one span, not three.
CODE_SPAN = re.compile(r"(`+)(?:(?!\1).)*?\1")

# --------------------------------------------------------------------------
# M4 -- what a project README has to say
# --------------------------------------------------------------------------
# A project is a directory holding both a `README.md` and a `CONTRIBUTING.md`:
# the promise and the procedure, which is what makes it a project rather than a
# subdirectory that happens to be documented. Derived, never listed -- a new
# project joins the population by existing.
#
# The repository root is excluded by that same derivation being wrong for it:
# the root `README.md` is the inventory (`## Projects`, `## Shared Libraries`,
# `## Install`), not a promise about one thing, and requiring `## Brief` of it
# would be requiring a section with nothing to put in it.
REQUIRED_H2 = ("## Brief", "## Capabilities")


class Finding:
    __slots__ = ("rule", "path", "line", "message")

    def __init__(self, rule: str, path: str, line: int, message: str) -> None:
        self.rule = rule
        self.path = path
        self.line = line
        self.message = message

    def as_dict(self) -> dict:
        return {"rule": self.rule, "path": self.path,
                "line": self.line, "message": self.message}


def usage_error(message: str):
    """Refuse the invocation, distinguishably from refusing the docs.

    Exit 2, never 1. A caller that cannot tell "three files have rotted" from
    "you passed a rule that does not exist" will read the second as the first,
    and a typo in `--rule` becomes a finding count it then tries to fix.
    """
    print(f"error: {message}", file=sys.stderr)
    raise SystemExit(2)


def repo_root() -> Path:
    """The checkout git says we are standing in.

    Not the outermost `aw.toml`, which is how the work-item scripts resolve it.
    The population here *is* `git ls-files`, so the root has to be the root that
    listing is relative to, or the two disagree and every path is measured
    against the wrong directory.
    """
    proc = subprocess.run([*GIT, "rev-parse", "--show-toplevel"],
                          capture_output=True, text=True)
    if proc.returncode != 0 or not proc.stdout.strip():
        usage_error("not inside a git checkout: "
                    f"{proc.stderr.strip() or 'git rev-parse failed'}")
    return Path(proc.stdout.strip()).resolve()


def ships_this_script(repo: Path) -> bool:
    """Whether the target repository is the one carrying `DEAD_COMMAND_EXEMPT`.

    That table quotes specific sentences from specific files. Against any other
    checkout the quotes are simply absent, and reporting them as stale would
    make the script report four findings about its own data rather than about
    the docs it was pointed at -- which is what the fixture in
    `check_meta_flow.py` measured the first time it ran.

    The stale-exemption rule itself is not weakened: the live run is a run over
    this checkout, so it is exactly the case where the rule applies.
    """
    try:
        Path(__file__).resolve().relative_to(repo.resolve())
    except ValueError:
        return False
    return True


def tracked_docs(repo: Path) -> list[str]:
    """Every tracked META-doc, repo-relative and sorted.

    Tracked, not globbed: a scratch `README.md` in an ignored build directory is
    not documentation this repository ships, and walking the tree would sweep in
    `target/`, `node_modules/`, and every fixture corpus under `projects/`.
    """
    proc = subprocess.run([*GIT, "ls-files", "-z"], cwd=repo,
                          capture_output=True, text=True)
    if proc.returncode != 0:
        usage_error(f"git ls-files failed: {proc.stderr.strip()}")
    return sorted(rel for rel in proc.stdout.split("\0")
                  if rel and rel.rsplit("/", 1)[-1] in META_DOCS)


def m1_orphan_markers(rel: str, text: str, out: list[Finding]) -> None:
    open_at: dict[str, int] = {}
    for lineno, line in enumerate(text.splitlines(), 1):
        for name, side in MARKER.findall(line):
            if side == "start":
                if name in open_at:
                    out.append(Finding("M1", rel, lineno,
                                       f"`aw:meta:{name}` opens again while the block "
                                       f"opened at line {open_at[name]} is still open"))
                open_at[name] = lineno
                continue
            start = open_at.pop(name, None)
            if start is None:
                out.append(Finding("M1", rel, lineno,
                                   f"`aw:meta:{name}:end` closes a block that never opened"))
            elif name not in PRODUCERS:
                out.append(Finding("M1", rel, start,
                                   f"`aw:meta:{name}` (lines {start}-{lineno}) names a "
                                   f"producer that does not exist"))
    for name, start in sorted(open_at.items(), key=lambda kv: kv[1]):
        out.append(Finding("M1", rel, start,
                           f"`aw:meta:{name}:start` is never closed"))


def m2_dead_commands(rel: str, text: str, out: list[Finding]) -> None:
    exempt = DEAD_COMMAND_EXEMPT.get(rel, ())
    for lineno, line, in_fence in walk(text):
        if not (AW_INVOCATION.search(line)
                or (in_fence and AW_BARE.match(line))):
            continue
        if any(fragment in line for fragment in exempt):
            continue
        spans = sorted(set(AW_SPAN.findall(line)))
        named = ", ".join(f"`{s}`" for s in spans) if spans else line.strip()
        out.append(Finding("M2", rel, lineno,
                           f"{named} -- there is no `aw` binary; the crate that "
                           f"carried it was deleted"))


def m2_stale_exemptions(selected: set[str], texts: dict[str, str],
                        whole_repo: bool, out: list[Finding]) -> None:
    for rel, fragments in sorted(DEAD_COMMAND_EXEMPT.items()):
        if rel not in selected:
            # Under `--path` the file was simply not asked for. Over the whole
            # repository it is an exemption for a file that no longer exists.
            if whole_repo:
                out.append(Finding("M2", rel, 0,
                                   "exemption names a file that is not a tracked META-doc"))
            continue
        for fragment in fragments:
            if fragment not in texts[rel]:
                out.append(Finding("M2", rel, 0,
                                   f"stale exemption: no line contains {fragment!r}"))


def walk(text: str):
    """Every line, with whether it sits inside a fenced code block.

    The two rules that need this need it in opposite directions, which is why
    the fence state is reported rather than filtered. M3 wants prose only: a
    fence is where Python lives, and `_Box[int](42)` in the CPython seed corpus
    is markdown link syntax to any regex -- two such lines were the whole of
    M3's false positives before this existed. M2 wants the fence *most*,
    because that is where a command sits unbackticked and ready to be copied.
    """
    fence = ""
    for lineno, line in enumerate(text.splitlines(), 1):
        marker = line.lstrip()[:3]
        if marker in ("```", "~~~"):
            fence = marker if not fence else ("" if marker == fence else fence)
            # The fence line itself belongs to neither side: ```` ```bash ````
            # is not prose, and it is not a command either.
            continue
        yield lineno, line, bool(fence)


def m3_broken_links(repo: Path, rel: str, text: str, out: list[Finding]) -> None:
    base = (repo / rel).parent
    for lineno, line, in_fence in walk(text):
        if in_fence:
            continue
        # Blanked to the same width rather than deleted, so a span never fuses
        # the text on either side of it into link syntax that was in neither.
        line = CODE_SPAN.sub(lambda m: " " * len(m.group(0)), line)
        for raw in LINK.findall(line):
            if not raw or raw[0] in "#/" or SCHEME.match(raw):
                continue
            # A placeholder is not a link. `[body](<path>)` and `{project}` are
            # instructions to substitute something, and resolving them would
            # report every template in the repository as broken.
            if any(c in raw for c in "<>{}"):
                continue
            target = raw.split("#", 1)[0].split("?", 1)[0]
            if not target or (base / target).exists():
                continue
            out.append(Finding("M3", rel, lineno, f"link target does not exist: {raw}"))


def m4_readme_shape(repo: Path, all_docs: list[str], selected: set[str],
                    texts: dict[str, str], out: list[Finding]) -> list[str]:
    """Report project READMEs missing a required section; return the population.

    The population is derived from `all_docs` rather than from the selection, so
    `--path apps/keep/README.md` still knows that `apps/keep/` is a project.
    """
    by_dir: dict[str, set[str]] = {}
    for rel in all_docs:
        head, _, name = rel.rpartition("/")
        by_dir.setdefault(head, set()).add(name)

    population = []
    for head, names in sorted(by_dir.items()):
        if not head or not {"README.md", "CONTRIBUTING.md"} <= names:
            continue
        rel = f"{head}/README.md"
        population.append(rel)
        if rel not in selected:
            continue
        text = texts.get(rel)
        if text is None:
            text = (repo / rel).read_text(encoding="utf-8")
        missing = [h for h in REQUIRED_H2
                   if not re.search(rf"^{re.escape(h)}\s*$", text, re.M)]
        if missing:
            out.append(Finding("M4", rel, 1,
                               "project README has no " + " and no ".join(missing)))
    return population


def collect(repo: Path, rules: tuple[str, ...],
            paths: tuple[str, ...]) -> tuple[list[Finding], dict]:
    all_docs = tracked_docs(repo)
    if not all_docs:
        usage_error(f"no tracked META-doc under {repo}")

    if paths:
        wanted = [p.rstrip("/") for p in paths]
        selected = [rel for rel in all_docs
                    if any(rel == w or rel.startswith(f"{w}/") for w in wanted)]
        if not selected:
            # Not "clean". A selection that matched nothing has measured
            # nothing, and reporting that as green is the exact shape of a
            # false green: a mistyped path would certify the repository.
            usage_error("--path selected no tracked META-doc: " + ", ".join(wanted))
    else:
        selected = list(all_docs)

    texts = {rel: (repo / rel).read_text(encoding="utf-8") for rel in selected}
    chosen = set(selected)
    out: list[Finding] = []

    for rel in selected:
        if "M1" in rules:
            m1_orphan_markers(rel, texts[rel], out)
        if "M2" in rules:
            m2_dead_commands(rel, texts[rel], out)
        if "M3" in rules:
            m3_broken_links(repo, rel, texts[rel], out)
    if "M2" in rules and ships_this_script(repo):
        m2_stale_exemptions(chosen, texts, not paths, out)
    projects = m4_readme_shape(repo, all_docs, chosen if "M4" in rules else set(),
                               texts, out)

    out.sort(key=lambda f: (f.path, f.line, f.rule))
    population = {
        "tracked_meta_docs": len(all_docs),
        "scanned": len(selected),
        "project_readmes": len(projects),
        "rules": list(rules),
    }
    return out, population


def cmd_check(args: argparse.Namespace) -> int:
    repo = Path(args.repo).resolve() if args.repo else repo_root()
    rules = tuple(args.rule) if args.rule else tuple(RULES)
    unknown = [r for r in rules if r not in RULES]
    if unknown:
        usage_error(f"unknown rule(s) {', '.join(unknown)}; "
                    f"known: {', '.join(RULES)}")

    findings, population = collect(repo, rules, tuple(args.path))
    counts = {rule: sum(1 for f in findings if f.rule == rule) for rule in rules}

    if args.format == "json":
        print(json.dumps({"population": population,
                          "counts": counts,
                          "findings": [f.as_dict() for f in findings]}, indent=2))
        return 1 if findings else 0

    print(f"META-doc check: {population['scanned']} of "
          f"{population['tracked_meta_docs']} tracked META-docs, "
          f"{population['project_readmes']} project READMEs")
    for rule in rules:
        print(f"  {rule}  {counts[rule]:>4}  {RULES[rule]}")

    current = ""
    for finding in findings:
        if finding.path != current:
            current = finding.path
            print(f"\n{current}")
        where = f":{finding.line}" if finding.line else ""
        print(f"  {finding.rule} {where:<6} {finding.message}")

    if not findings:
        print("\n=> CLEAN")
        return 0
    files = len({f.path for f in findings})
    print(f"\n=> {len(findings)} finding(s) in {files} file(s)")
    print("next.command: fix the files above, then re-run this verb")
    return 1


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="meta.py",
                                     description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="verb", required=True)

    # One verb, and the singleton is the point. A second verb here would be a
    # verb that writes, and what this replaces was deleted for writing.
    p = sub.add_parser("check", help="refuse a META-doc whose facts have rotted")
    p.add_argument("--repo", help="checkout to measure; defaults to the one git reports")
    p.add_argument("--rule", action="append", default=[], metavar="ID",
                   help=f"restrict to one rule; repeatable ({', '.join(RULES)})")
    p.add_argument("--path", action="append", default=[], metavar="PATH",
                   help="restrict to a repo-relative file or directory; repeatable")
    p.add_argument("--format", choices=("text", "json"), default="text")
    p.set_defaults(func=cmd_check)

    args = parser.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
