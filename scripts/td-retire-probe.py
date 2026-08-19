#!/usr/bin/env python3
"""Probe the tech-design retirement population.

usage: td-retire-probe.py <tree-prefix> [<tree-prefix> ...]
       td-retire-probe.py --expect '<line>' <tree-prefix> [<tree-prefix> ...]
       td-retire-probe.py --census

Prints  md=<n> lock=<n> py=<n> hdr=<n> files=<n> other=<n>  for the listed trees.

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
  py      Python TD files under any listed tree.  Held FIXED by every gate:
          they are a different artifact class, and the loudest way to fail
          this campaign is to delete a tech-design directory wholesale.
  hdr     REAL standalone header lines outside every tree that point into one
  files   distinct files holding at least one such line
  other   lines naming a listed tree that are NOT standalone headers -- prose
          mentions and code string literals.  Deliberately out of scope.
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
HEADER = re.compile(
    r"^(?:///|//!|//|#|<!--)\s*(?:SPEC-MANAGED:|SPEC-REF:|@spec)\s")
RAW = re.compile("r" + "#" + "+" + chr(34))
QUOTE = chr(34)
INCL = re.compile(r"include_(?:str|bytes|dir)!\s*\(\s*" + QUOTE + r"([^" + QUOTE + r"]+)" + QUOTE)


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
    key = "/".join(parts[:parts.index("tech-design") + 1])
    return key[2:] if key.startswith("./") else key


def _deleted(path):
    """True when this campaign would delete `path`, so embedding it is fatal."""
    return path.endswith(".md") or os.path.basename(path) == "td.lock"


def _hits(m, rd, want):
    """True when a matched reference names a wanted tree, literally or resolved.

    A reference is written either from the repository root or from the
    referring file.  Testing only the literal prefix loses the second form;
    testing only the resolved path loses the first, since `apps/x/tech-design`
    named inside `apps/x/src/` resolves to a path that does not exist.
    """
    literal = m.group(1)
    literal = literal[2:] if literal.startswith("./") else literal
    if want(literal):
        return True
    parts = os.path.normpath(os.path.join(rd, m.group(0))).split("/")
    return "tech-design" in parts and want(_tree(parts))


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
    md = lock = py = hdr = other = embed = 0
    files = set()
    for dp, dns, fns in os.walk(ROOT):
        dns[:] = [d for d in dns if d not in SKIP]
        rd = os.path.relpath(dp, ROOT)
        parts = rd.split("/")
        if "tech-design" in parts:
            tree = _tree(parts)
            if want(tree):
                md += sum(1 for f in fns if f.endswith(".md"))
                lock += sum(1 for f in fns if f == "td.lock")
                py += sum(1 for f in fns if f.endswith(".py"))
            continue
        for fn in fns:
            fp = os.path.join(dp, fn)
            text = read_text(fp)
            if text is None or "tech-design" not in text:
                continue
            rel = os.path.normpath(os.path.join(rd, fn))
            masked = literal_lines(text) if fn.endswith(".rs") else set()
            for im in INCL.finditer(text):
                tgt = os.path.normpath(os.path.join(rd, im.group(1)))
                parts = tgt.split("/")
                if ("tech-design" in parts and want(_tree(parts))
                        and _deleted(tgt)):
                    embed += 1
            for n, line in enumerate(text.splitlines(), 1):
                m = REF.search(line)
                if not m or not _hits(m, rd, want):
                    continue
                if HEADER.match(line.strip()) and n not in masked:
                    hdr += 1
                    files.add(rel)
                else:
                    other += 1
    return md, lock, py, hdr, files, other, embed


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
            if "tech-design" in parts:
                trees.add(_tree(parts))
        for t in sorted(trees):
            md, lock, py, hdr, files, other, embed = scan(
                lambda x, t=t: x == t or x.startswith(t + "/"))
            print("%-36s md=%-5d lock=%-3d py=%-5d hdr=%-5d files=%-5d other=%-4d embed=%d"
                  % (t, md, lock, py, hdr, len(files), other, embed))
        return 0
    for p in argv:
        if not _resolves(p):
            sys.exit(f"error: unknown tree prefix: {p}")
    want = lambda t: any(t == p or t.startswith(p.rstrip("/") + "/") for p in argv)
    md, lock, py, hdr, files, other, embed = scan(want)
    line = ("md=%d lock=%d py=%d hdr=%d files=%d other=%d embed=%d"
            % (md, lock, py, hdr, len(files), other, embed))
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
