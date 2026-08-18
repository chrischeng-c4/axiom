#!/usr/bin/env python3
"""Probe the tech-design retirement population.

usage: td-retire-probe.py <tree-prefix> [<tree-prefix> ...]
       td-retire-probe.py --census

Prints  md=<n> lock=<n> py=<n> hdr=<n> files=<n> other=<n>  for the listed trees.

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
          RESOLVES to a file under a listed tree.  Must be 0 before that
          tree's Markdown may be deleted: such a file is a product input,
          not documentation.  Resolution is relative to the embedding file,
          because the one real instance in this repository is written
          `include_str!("../tech-design/...")` and no string comparison
          against a tree name would find it.

A header-shaped line that sits inside a Rust string literal is source code, not
a comment: `apps/agentic-workflow` embeds 33 of them as codegen fixtures.  Such
lines are excluded from `hdr` and counted in `other`, because deleting one
edits the crate's behaviour rather than its documentation.
"""
from __future__ import annotations

import os
import re
import sys

ROOT = os.environ.get(
    "TD_PROBE_ROOT",
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
)
SKIP = {".git", "target", "node_modules", "__pycache__", ".venv",
        "site-packages", "dist", "build"}
EXT = {".rs", ".py", ".toml", ".sh", ".yaml", ".yml"}
REF = re.compile(r"((?:[\w./\-]*?)tech-design)/[\w./\-]+\.md")
HEADER = re.compile(r"^(?:///|//!|//|#)\s*(?:SPEC-MANAGED:|SPEC-REF:|@spec)\s")
RAW = re.compile("r" + "#" + "+" + chr(34))
QUOTE = chr(34)
INCL = re.compile(r"include_(?:str|bytes|dir)!\s*\(\s*" + QUOTE + r"([^" + QUOTE + r"]+)" + QUOTE)


def _tree(parts):
    """Tree key for a directory split, preserving a leading dot (`.aw`)."""
    key = "/".join(parts[:parts.index("tech-design") + 1])
    return key[2:] if key.startswith("./") else key


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
            if os.path.splitext(fn)[1] not in EXT:
                continue
            try:
                text = open(os.path.join(dp, fn), errors="replace").read()
            except OSError:
                continue
            if "tech-design" not in text:
                continue
            rel = os.path.normpath(os.path.join(rd, fn))
            masked = literal_lines(text) if fn.endswith(".rs") else set()
            for n, line in enumerate(text.splitlines(), 1):
                for im in INCL.finditer(line):
                    tgt = os.path.normpath(os.path.join(rd, im.group(1)))
                    parts = tgt.split("/")
                    if "tech-design" in parts and want(_tree(parts)):
                        embed += 1
                m = REF.search(line)
                if not m or not want((lambda g: g[2:] if g.startswith("./") else g)(m.group(1))):
                    continue
                if HEADER.match(line.strip()) and n not in masked:
                    hdr += 1
                    files.add(rel)
                else:
                    other += 1
    return md, lock, py, hdr, files, other, embed


def main(argv):
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
        return
    want = lambda t: any(t == p or t.startswith(p.rstrip("/") + "/") for p in argv)
    md, lock, py, hdr, files, other, embed = scan(want)
    print("md=%d lock=%d py=%d hdr=%d files=%d other=%d embed=%d"
          % (md, lock, py, hdr, len(files), other, embed))


if __name__ == "__main__":
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    main(sys.argv[1:])
