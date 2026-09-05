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
import tomllib
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
# `codex exec` loads it as its instructions, so it is codex's instruction file,
# with a reader of its own, and a rule that edits it changes what codex is told.
# It carries 0 markers and 0 `aw <verb>` spans today, so admitting it would
# change no result while making an instruction file answerable to a doc rule.
META_DOCS = ("CLAUDE.md", "README.md", "CONTRIBUTING.md")

RULES = {
    "M1": "an `aw:meta:*` marker whose producer does not exist",
    "M2": "an `aw <verb>` command naming a CLI that was deleted",
    "M3": "a relative link whose target is not in the checkout",
    "M4": "a project README missing a required section",
    "M5": "a capability gate whose test-name filter exits 0 when it matches nothing",
    "M6": "a capability gate naming a cargo package or test target that does not exist",
    "M7": "a capability field asserting a state nothing measures",
}

# M5, M6, and M7 read a project README's `## Capabilities` section, so they are
# the first three rules whose defect is a *claim* rather than a broken
# reference. M1-M4 ask "does this resolve?"; these ask "could this promise ever
# have been refused?" -- which is the question `CONTRIBUTING.md` line 1637
# leaves to a reader ("Nothing validates this shape") and the one that let 62
# `CAPABILITIES.md` files reach the identical empty template before anybody
# noticed.
#
# They are scoped to project READMEs and not to the other two META-docs on
# purpose. A `cargo test` line in the root `CONTRIBUTING.md` is an *example of a
# command's shape*, quoted to teach the convention, and a rule that read it as a
# gate would report the documentation of the rule as a violation of it. The
# promise-to-gate binding this measures exists in exactly one place, which
# `CONTRIBUTING.md` line 1625 names: a capability heading in a project README.

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
# M2 -- `aw` invocations that name no live CLI group
# --------------------------------------------------------------------------
# The same detector `check_plugin.py` runs over the skill bodies, deliberately
# byte-identical: two populations, one defect. `check_meta_flow.py` asserts
# neither the pattern nor the group registry has drifted, because nothing else
# compares them.
#
# The `aw` name is live again since 2026-09-02: `apps/aw` exposes this engine
# as typer groups, launched as `uv run --project apps/aw aw <group> ...`. So
# the rule no longer refuses every `aw` span -- a doc naming a live group is
# naming the CLI that exists. What stays refused is a span whose first token
# after `aw` is outside the group registry, which is exactly the shape of a
# retired verb (`aw wi`, `aw epic`, `aw review`) or a typo: unrunnable in the
# same way the deleted binary's verbs were.
AW_GROUPS = (
    "change", "milestone", "e2e", "impl", "maint",
    "wis", "meta", "metadoc", "release-plan", "version",
)
AW_INVOCATION = re.compile(r"(?:`|\buv run --project apps/aw )aw\s+([a-z0-9-]+)")
AW_SPAN = re.compile(r"`((?:uv run --project apps/aw )?aw\s+[a-z][^`]*)`")

# The backtick is what makes a command a command in prose -- and it is exactly
# what a fenced block does not have. `apps/jet/README.md:63-64` is a ```bash
# block holding two bare `aw` lines, which is the most copy-and-run shape in the
# repository and the one the pattern above cannot see. So a second pattern, and
# it is applied only inside a fence: outside one, a line beginning "aw " is an
# English sentence far more often than a command.
AW_BARE = re.compile(r"^\s*aw\s+([a-z0-9-]+)")

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
# `.claude/aw/verification/README.md:315` says the change schema is "enforced by
# `aw wi validate`" in the present tense -- that is the rot, not a record of it.
# `apps/mamba/CONTRIBUTING.md:90` was a live instruction in a checklist table
# telling a human to run `aw wi create`; exempting it would have left a reader
# following a command that cannot run. It now names an approved
# `aw-grill-release apply` and `change.py create`, which is the shape a
# forward-looking instruction has to take -- unlike a past-tense evidence row,
# which must not be repointed at a live command, because that fabricates a
# measurement nobody took.
DEAD_COMMAND_EXEMPT: dict[str, tuple[str, ...]] = {
    "CLAUDE.md": (
        'a stray `aw wi …` now fails with "command not found"',
    ),
    "CONTRIBUTING.md": (
        "It was `aw review --project <project>`, spliced",
        "and `aw meta init` / `sync` / `check`",
    ),
    ".claude/aw/verification/README.md": (
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

# --------------------------------------------------------------------------
# M5 -- a gate that cannot go red
# --------------------------------------------------------------------------
# `cargo test -p <crate> <filter>` exits **0** when `<filter>` selects no test.
# Not 1, not a warning -- libtest runs zero tests and reports success, so the
# gate is green in exactly the two cases a gate exists to tell apart: the
# behaviour holds, and the test that measured it was renamed out from under the
# README. Measured in this repository: `apps/lumen/README.md` gates that no
# longer matched anything kept printing `test result: ok. 0 passed`.
#
# `--test <target>` is the opposite direction and is why M6 exists beside this
# one: a target that is gone makes cargo *fail*, loudly, with "no test target
# named ...". So the repair for an M5 is to name the target, never to correct
# the filter -- a corrected filter is one rename away from being wrong again and
# silent about it.
#
# The parse deliberately understands which flags take a value. `--test peer_mtls`
# and `--features operator` put a bare word on the line that is not a filter, and
# reading them as one would report every correctly-targeted gate in the tree.
# The span ends at the first character that ends a command, and every one of
# them was put here by a false positive this rule reported before it was:
# `#` began a trailing shell comment (`cargo test -p arena  # spec/compare units`,
# whose comment words parsed as six filters), `"` closed the JSON string a
# command was quoted inside (`"command": "cargo test -p cap",`, which made the
# package `cap",`), and `|` closes a markdown table cell. Truncating early is
# the safe direction: it can only cost a finding, never invent one.
CARGO_TEST = re.compile(r"cargo\s+test\b([^`\n#\"'|;&]*)")
CARGO_VALUE_FLAGS = frozenset((
    "-p", "--package", "--exclude", "--features", "-F", "--test", "--bin",
    "--example", "--bench", "--manifest-path", "--target", "--profile",
    "--target-dir", "-j", "--jobs",
))

# --------------------------------------------------------------------------
# M6 -- a gate naming something that is not there
# --------------------------------------------------------------------------
# The package and the test target are the two halves of a gate that cargo
# resolves *before* running anything, so both are checkable without a build --
# which matters, because resolving them the way cargo does means `cargo
# metadata` and a compile-shaped dependency this script does not have.
#
# `autotests = false` is read rather than assumed. A crate that declares it has
# no targets except the `[[test]]` stanzas it lists, which is the arrangement
# `CLAUDE.md` requires under "Test Layout" and the reason a new `e2e/*.rs` with
# no stanza silently never runs. A crate *without* it still autodiscovers
# `tests/*.rs`, so both sets are admitted or this rule would report every
# ordinary crate in the repository.
CARGO_PKG_FLAGS = ("-p", "--package")

# --------------------------------------------------------------------------
# M7 -- a field that grades itself
# --------------------------------------------------------------------------
# `Status: verified`, `Maturity: smoke`, `Production: ready`. Nothing reads
# these. No script parses them, no gate compares them to a test result, and
# nothing goes red when the code behind one stops being any of those things --
# they are the state of the world on the day somebody typed them.
#
# That makes them the exact defect `CLAUDE.md`'s "Authoring" section
# refuses: a section that no consumer refuses degenerates into a title echo.
# A capability's refusable content is
# its promise, its named surface, and its gate command -- the three things
# `CONTRIBUTING.md` line 1625 asks for. These fields sit beside those three
# looking like more of them.
#
# They are residue, not authorship: the verb that emitted them was deleted with
# its crate, which is the same event M1 exists for. M1 cannot see these because
# the generator wrote them without a marker pair around them.
#
# `Root WI`, `Promise`, `Surfaces`, and `Gate` are deliberately absent from the
# list. All four are required or useful content, and three of them are named
# verbatim in the required shape.
SELF_GRADED_FIELD = re.compile(
    r"^\s*(?:[-*]\s+)?(Status|Maturity|Production|Feature Class|"
    r"Required Verification)\s*:", re.M)

# The same five fields in their other shape. The generator emitted a "Capability
# Index" table whose columns are `Impl | Verification | Maturity | Production`,
# and a rule that only read the line form would pass a file holding twenty rows
# of it. Both `Maturity` and `Production` are required in the header so that an
# ordinary table that happens to have a column called "Status" is not swept in.
INDEX_TABLE_HEADER = re.compile(r"^\|.*\bMaturity\b.*\bProduction\b.*\|\s*$", re.M)


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
        groups = AW_INVOCATION.findall(line)
        if in_fence:
            bare = AW_BARE.match(line)
            if bare:
                groups.append(bare.group(1))
        dead = sorted({group for group in groups if group not in AW_GROUPS})
        if not dead:
            continue
        if any(fragment in line for fragment in exempt):
            continue
        spans = sorted(set(AW_SPAN.findall(line)))
        named = ", ".join(f"`{s}`" for s in spans) if spans else line.strip()
        out.append(Finding("M2", rel, lineno,
                           f"{named} -- `aw {dead[0]}` names no live `aw` CLI "
                           f"group; the retired binary's verbs left with the "
                           f"deleted crate"))


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


def cargo_index(repo: Path) -> dict[str, set[str]]:
    """Package name -> the test targets `cargo test -p <name> --test <t>` accepts.

    Tracked manifests only, for the reason `tracked_docs` gives: a `Cargo.toml`
    under `target/` belongs to a build, not to this repository.

    A package that is present with no test targets maps to an empty set, which
    is distinct from a package that is absent and is why the value is a set
    rather than a truthy list -- M6 has to tell "you named a target this crate
    does not declare" from "you named a crate that does not exist", and those
    are different sentences to a reader holding a failing command.
    """
    proc = subprocess.run([*GIT, "ls-files", "-z", "--", "*Cargo.toml"], cwd=repo,
                          capture_output=True, text=True)
    if proc.returncode != 0:
        usage_error(f"git ls-files failed: {proc.stderr.strip()}")

    index: dict[str, set[str]] = {}
    for rel in proc.stdout.split("\0"):
        if not rel:
            continue
        try:
            manifest = tomllib.loads((repo / rel).read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError):
            # A manifest this script cannot parse is not a finding about a
            # document. Skipping it costs M6 one crate; reporting it would put
            # a Rust defect in a META-doc report.
            continue
        package = manifest.get("package")
        if not isinstance(package, dict) or not isinstance(package.get("name"), str):
            continue

        targets = {t.get("name") for t in manifest.get("test", [])
                   if isinstance(t, dict) and isinstance(t.get("name"), str)}
        if package.get("autotests") is not False:
            targets |= {p.stem for p in (repo / rel).parent.glob("tests/*.rs")}
        index[package["name"]] = targets
    return index


def cargo_test_spans(text: str):
    """(line, packages, targets, filters) for every `cargo test` in the text.

    One parser for M5 and M6 so the two rules can never disagree about which
    word on a line is a filter and which is the value of the flag before it.
    """
    for lineno, line in enumerate(text.splitlines(), 1):
        for match in CARGO_TEST.finditer(line):
            packages: list[str] = []
            targets: list[str] = []
            filters: list[str] = []
            pending = ""
            for token in match.group(1).split():
                if pending:
                    if pending in CARGO_PKG_FLAGS:
                        packages.append(token)
                    elif pending == "--test":
                        targets.append(token)
                    pending = ""
                    continue
                if token in CARGO_VALUE_FLAGS:
                    pending = token
                    continue
                if token.startswith("-"):
                    # `--lib`, `--all-targets`, `--`, `--nocapture`: no value,
                    # and none of them is a filter.
                    continue
                filters.append(token)
            yield lineno, packages, targets, filters


def m5_vacuous_gates(rel: str, text: str, out: list[Finding]) -> None:
    for lineno, _packages, _targets, filters in cargo_test_spans(text):
        for name in filters:
            out.append(Finding("M5", rel, lineno,
                               f"`{name}` is a test-name filter -- cargo exits 0 when "
                               f"it matches nothing, so this gate cannot go red; name "
                               f"the target with `--test <target>` or `--lib` instead"))


def m6_absent_gate_targets(rel: str, text: str, index: dict[str, set[str]],
                           out: list[Finding]) -> None:
    for lineno, packages, targets, _filters in cargo_test_spans(text):
        known = [p for p in packages if p in index]
        for package in packages:
            if package not in index:
                out.append(Finding("M6", rel, lineno,
                                   f"`-p {package}` names no package in this checkout"))
        for target in targets:
            if not known:
                # `--test` with no resolvable package is already reported above,
                # or the command names no package at all and cargo would pick
                # the manifest it is standing in -- which this script cannot
                # know. Either way the target is unanswerable, not absent.
                continue
            if any(target in index[p] for p in known):
                continue
            out.append(Finding("M6", rel, lineno,
                               f"`--test {target}` names no test target declared by "
                               + " or ".join(f"`{p}`" for p in known)))


def m7_self_graded_fields(rel: str, text: str, out: list[Finding]) -> None:
    lines = text.splitlines()
    for lineno, line in enumerate(lines, 1):
        field = SELF_GRADED_FIELD.match(line)
        if field:
            out.append(Finding("M7", rel, lineno,
                               f"`{field.group(1)}:` grades this capability, and nothing "
                               f"measures it -- the refusable content is the promise, "
                               f"the named surface, and the gate command"))
        if INDEX_TABLE_HEADER.match(line):
            out.append(Finding("M7", rel, lineno,
                               "a Capability Index table grades every row it holds "
                               "(`Maturity`, `Production`), and nothing measures those "
                               "columns"))


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

    # M5-M7 run over the project READMEs and nothing else, so they read the
    # population M4 derived rather than deriving a second one. Two derivations
    # of "what is a project" would be two answers the day one of them changed.
    capability_rules = {"M5", "M6", "M7"} & set(rules)
    if capability_rules:
        index = cargo_index(repo) if "M6" in capability_rules else {}
        for rel in projects:
            if rel not in chosen:
                continue
            if "M5" in capability_rules:
                m5_vacuous_gates(rel, texts[rel], out)
            if "M6" in capability_rules:
                m6_absent_gate_targets(rel, texts[rel], index, out)
            if "M7" in capability_rules:
                m7_self_graded_fields(rel, texts[rel], out)

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
