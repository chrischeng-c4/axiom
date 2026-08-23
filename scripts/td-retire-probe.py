#!/usr/bin/env python3
"""Probe the tech-design retirement population.

usage: td-retire-probe.py <tree-prefix> [<tree-prefix> ...]
       td-retire-probe.py --expect '<line>' <tree-prefix> [<tree-prefix> ...]
       td-retire-probe.py --census

Prints one `<column>=<n>` pair per member of `COLUMNS`, in that order, for the
listed trees.  Nothing hand-writes that line: the format string, the census
row, and both gates' all-zeros expectation are all built from `COLUMNS`, because
the two defects that column list has already produced (#3746, #3762) were both a
literal that stopped matching the columns beside it.

`--expect` compares that whole printed line against its argument byte for byte
and exits 1 on any difference, printing both.  Without it the probe always
exits 0, which makes it a report rather than a gate: a round could delete
nothing, print a line nobody read, and be recorded as passing.  The comparison
is exact and covers every column, including the ones a given round holds fixed
-- a change that reaches its own target by also dropping a held-fixed column is
the failure the gate exists to catch, and a per-column ceiling would let it
through.

  md      regular *.md files under any listed tree
  lock    td.lock files under any listed tree
  py      Python TD files under any listed tree.  Held FIXED by the Markdown
          children of #3694 -- for them a Python TD project is a different
          artifact class, and the loudest way to fail that campaign is to
          delete a tech-design directory wholesale.  DRIVEN TO ZERO by the
          whole-tree retirement of the fifteen lumen-scope projects, where
          deleting the directory wholesale is the decision.  Which of the two
          a given round is doing is not inferable from this column: it is
          declared by `td-retire-gate.py`'s `--whole-tree`, and each mode
          refuses the other's target.
  tdrest  files under any listed tech-design tree that are none of the above --
          `pyproject.toml`, `uv.lock`, egg-info, a `.gitkeep`.  Counted by
          nothing before, so "the tree is gone" was unmeasurable: the three
          columns above cover only three filename shapes, and #3737 is one
          `.gitkeep` keeping a retired tree in the census.
  ec      Python external-contract files under any listed tree.
  ecrest  non-Python external-contract files under any listed tree, excluding
          generated evidence.
  ecev    non-Python files under a listed tree's `evidence/`.  `ec` counts only
          Python and `ecrest` excludes evidence outright, so 251 files in the
          lumen scope alone belonged to no column at all.
  ecdirs  external-contracts directories the walk reached, excluding evidence.
  ecscan  external-contracts directories whose files reached the reference scan,
          excluding evidence.
  hdr     REAL standalone header lines outside every tree that point into one
  files   distinct files holding at least one such line
  echdr   REAL standalone header lines that point into a listed
          external-contracts tree, counted only outside every tree.  `REF`
          matches `tech-design/**.md` and nothing else, so these 237 lines --
          104 of them in `apps/lumen/tests/` alone -- were invisible to `hdr`
          and to `other` alike, and a round that stripped one would be reported
          by `deletions` as having deleted a line the rule does not select.
  ecfiles distinct files holding at least one `echdr` line
  tdref   contract bindings (td_ref in .toml) pointing to a listed tree
  other   lines naming a listed tree that are NOT standalone headers or td_ref
          bindings -- prose mentions and code string literals.  Deliberately
          out of scope.
  embed   compile-time embeds (`include_str!` and friends) whose argument
          RESOLVES to a file this campaign would DELETE (`*.md`, `td.lock`)
          under a listed tree.  Must be 0 before that tree's Markdown may be
          deleted: such a file is a product input, not documentation.
          Resolution is relative to the embedding file, because the instances
          in this repository are written `include_str!("../tech-design/...")`
          and no string comparison against a tree name would find them.
          Embeds of a `.py` under a tree do not count: `py` is held fixed, so
          the file is never deleted and the embed is not a hazard.  Counting
          them would put `embed=0` permanently out of reach for
          `apps/agentic-workflow`, whose `llm.rs` compiles in two of them.

A header-shaped line that sits inside a Rust string literal is source code, not
a comment: `apps/agentic-workflow` embeds 33 of them as codegen fixtures.  Such
lines are excluded from `hdr` and counted in `other`, because deleting one
edits the crate's behaviour rather than its documentation.

Four comment syntaxes count, not three.  The generated `llms.txt` files carry
their header as `<!-- SPEC-MANAGED: ... -->`, in a file type no earlier version
of this probe scanned; 30 such lines were structurally invisible, one per tree,
so every tree-keyed change reported `hdr=0` while its project root still
pointed at the tree it had just deleted.

A reference is matched against a tree TWICE: once as the literal prefix it
carries, and, failing that, once resolved against the referring file's own
directory.  Only the first test existed at first, so `../tech-design/x.md`
matched no tree and fell out of `hdr` AND `other` alike -- seven such lines,
none of them header-shaped.

`include_` is matched against the whole file, not line by line.  Four of the
five compile-time embeds in this repository put the macro and its string
argument on separate lines, so a per-line scan reported `embed=0` for a tree
that still compiles a `.md` in, which is the single thing that column exists
to prevent.
"""
from __future__ import annotations

import os
import re
import subprocess
import sys

ROOT = os.environ.get(
    "TD_PROBE_ROOT",
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
)
SKIP = {".git", "target", "node_modules", "__pycache__", ".venv",
        "site-packages", "dist", "build"}
REF = re.compile(r"((?:[\w./\-]*?)tech-design)/[\w./\-]+\.md")
# The external-contracts counterpart of REF.  Not restricted to one extension:
# what a header names under an EC tree is a `.py` case, a `.md` brief, a `.json`
# review or a `.lock`, and the retirement deletes all of them.
ECREF = re.compile(r"((?:[\w./\-]*?)external-contracts)/[\w./\-]+\.\w+")
COLUMNS = ("md", "lock", "py", "tdrest", "ec", "ecrest", "ecev", "ecdirs",
           "ecscan", "hdr", "files", "echdr", "ecfiles", "tdref", "other",
           "embed")
LINE_FMT = " ".join(c + "=%d" for c in COLUMNS)
CENSUS_FMT = "%-36s " + " ".join(
    c + "=%-" + str(max(2, 6 - len(c))) + "d" for c in COLUMNS)
HEADER = re.compile(
    r"^(?:///|//!|//|#|<!--)\s*(?:SPEC-MANAGED:|SPEC-REF:|@spec)\s")
RAW = re.compile("r" + "#" + "+" + chr(34))
QUOTE = chr(34)
INCL = re.compile(r"include_(?:str|bytes|dir)!\s*\(\s*" + QUOTE + r"([^" + QUOTE + r"]+)" + QUOTE)
TDREF = re.compile(r"^td_ref\s*=")


def read_text(path):
    """Return file contents as text if path exists, can be read, and survives a UTF-8 round trip."""
    try:
        with open(path, "rb") as f:
            raw = f.read()
        text = raw.decode("utf-8")
        if text.encode("utf-8") != raw:
            return None
        return text
    except (OSError, UnicodeDecodeError):
        return None


def _tree(parts):
    """Tree key for a directory split, preserving a leading dot (`.aw`)."""
    idx = parts.index("external-contracts") if "external-contracts" in parts else parts.index("tech-design")
    key = "/".join(parts[:idx + 1])
    return key[2:] if key.startswith("./") else key


def _deleted(path):
    """True when this campaign would delete `path`, so embedding it is fatal."""
    return path.endswith(".md") or os.path.basename(path) == "td.lock"


def _hits(m, rd, want, marker="tech-design"):
    """True when a matched reference names a wanted tree, literally or resolved.

    A reference is written either from the repository root or from the
    referring file.  Testing only the literal prefix loses the second form;
    testing only the resolved path loses the first, since `apps/x/tech-design`
    named inside `apps/x/src/` resolves to a path that does not exist.

    `marker` selects which family the resolved half is tested against, so the
    same two-sided rule serves `REF` and `ECREF` instead of being written twice
    and drifting.
    """
    literal = m.group(1)
    literal = literal[2:] if literal.startswith("./") else literal
    if want(literal):
        return True
    parts = os.path.normpath(os.path.join(rd, m.group(0))).split("/")
    return marker in parts and want(_tree(parts))


def selects(line_stripped, rd, want):
    """True when the retirement rule deletes this line for the wanted trees.

    One definition, shared by the probe's own count, `td-retire-gate.py`'s
    `deletions` row and `td-retire-apply.py`'s rewrite, so that the gate keeps
    measuring the rule rather than restating it.
    """
    if not HEADER.match(line_stripped):
        return False
    m = REF.search(line_stripped)
    if m and _hits(m, rd, want):
        return True
    m = ECREF.search(line_stripped)
    return bool(m and _hits(m, rd, want, "external-contracts"))


def literal_lines(text):
    """1-indexed line numbers whose content lies inside a Rust string literal."""
    out = set()
    for m in RAW.finditer(text):
        n = m.group(0).count("#")
        end = text.find(QUOTE + "#" * n, m.end())
        if end < 0:
            continue
        first = text.count("\n", 0, m.end()) + 1
        last = text.count("\n", 0, end) + 1
        out.update(range(first, last + 1))
    lines = text.splitlines()
    for i, line in enumerate(lines, 1):
        prev = lines[i - 2].rstrip() if i > 1 else ""
        if prev.endswith("\\") or "\\n" in line or line.rstrip().endswith(QUOTE):
            out.add(i)
    return out


def scan(want):
    n_ = dict.fromkeys(COLUMNS, 0)
    files, ecfiles = set(), set()
    for dp, dns, fns in os.walk(ROOT):
        dns[:] = [d for d in dns if d not in SKIP]
        rd = os.path.relpath(dp, ROOT)
        parts = rd.split("/")
        if "tech-design" in parts:
            tree = _tree(parts)
            if want(tree):
                n_["md"] += sum(1 for f in fns if f.endswith(".md"))
                n_["lock"] += sum(1 for f in fns if f == "td.lock")
                n_["py"] += sum(1 for f in fns if f.endswith(".py"))
                n_["tdrest"] += sum(
                    1 for f in fns
                    if not f.endswith((".md", ".py")) and f != "td.lock")
            continue
        in_ec = "external-contracts" in parts
        if in_ec:
            tree = _tree(parts)
            if want(tree):
                n_["ec"] += sum(1 for f in fns if f.endswith(".py"))
                if "evidence" in parts:
                    n_["ecev"] += sum(1 for f in fns if not f.endswith(".py"))
                else:
                    n_["ecrest"] += sum(1 for f in fns if not f.endswith(".py"))
        if in_ec and "evidence" not in parts:
            n_["ecdirs"] += 1
        for fn in fns:
            if in_ec and "evidence" not in parts and fn == fns[0]:
                n_["ecscan"] += 1
            fp = os.path.join(dp, fn)
            text = read_text(fp)
            if text is None or not (
                    "tech-design" in text or "external-contracts" in text):
                continue
            rel = os.path.normpath(os.path.join(rd, fn))
            masked = literal_lines(text) if fn.endswith(".rs") else set()
            for im in INCL.finditer(text):
                tgt = os.path.normpath(os.path.join(rd, im.group(1)))
                tparts = tgt.split("/")
                if ("tech-design" in tparts and want(_tree(tparts))
                        and _deleted(tgt)):
                    n_["embed"] += 1
            for n, line in enumerate(text.splitlines(), 1):
                stripped = line.strip()
                is_hdr = bool(HEADER.match(stripped)) and n not in masked
                em = ECREF.search(line)
                if (em and not in_ec and is_hdr
                        and _hits(em, rd, want, "external-contracts")):
                    n_["echdr"] += 1
                    ecfiles.add(rel)
                m = REF.search(line)
                if not m or not _hits(m, rd, want):
                    continue
                if is_hdr:
                    n_["hdr"] += 1
                    files.add(rel)
                elif fn.endswith(".toml") and TDREF.match(stripped):
                    n_["tdref"] += 1
                else:
                    n_["other"] += 1
    n_["files"] = len(files)
    n_["ecfiles"] = len(ecfiles)
    return n_


def _resolves(prefix):
    """True when prefix matches a path on disk or a tracked path in the repository."""
    if not prefix:
        return False
    if os.path.exists(os.path.join(ROOT, prefix)):
        return True
    try:
        res = subprocess.run(
            ["git", "ls-files", "--", prefix],
            cwd=ROOT,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        return res.returncode == 0 and bool(res.stdout.strip())
    except (OSError, subprocess.SubprocessError):
        return False


def main(argv):
    expect = None
    if argv[:1] == ["--expect"]:
        if len(argv) < 3:
            sys.exit("error: --expect takes a line and at least one tree prefix")
        expect, argv = argv[1], argv[2:]
    if argv == ["--census"]:
        trees = set()
        for dp, dns, _ in os.walk(ROOT):
            dns[:] = [d for d in dns if d not in SKIP]
            parts = os.path.relpath(dp, ROOT).split("/")
            if "tech-design" in parts or "external-contracts" in parts:
                trees.add(_tree(parts))
        for t in sorted(trees):
            n_ = scan(lambda x, t=t: x == t or x.startswith(t + "/"))
            print(CENSUS_FMT % ((t,) + tuple(n_[c] for c in COLUMNS)))
        return 0
    for p in argv:
        if not _resolves(p):
            sys.exit(f"error: unknown tree prefix: {p}")
    want = lambda t: any(t == p or t.startswith(p.rstrip("/") + "/") for p in argv)
    n_ = scan(want)
    line = LINE_FMT % tuple(n_[c] for c in COLUMNS)
    print(line)
    if expect is not None and line != expect.strip():
        print("expected: " + expect.strip())
        print("actual:   " + line)
        return 1
    return 0


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1:]))
