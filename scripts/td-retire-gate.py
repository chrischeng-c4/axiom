#!/usr/bin/env python3
"""Judge one tree's retirement round: the counts, the residue, and the diff.

`td-retire-probe.py` answers "is the tree gone and are its pointers gone".
That is necessary and not sufficient, because it says nothing about *how* a
round reached zero. A round can reach `hdr=0` by deleting a whole file, by
reflowing a comment block, or by deleting a header line that pointed somewhere
else entirely, and the probe's columns are blind to all three.

So this gate reads the round's own diff as well as its counts, and it does so
against the same predicate the probe implements — imported from it rather than
restated, because a second copy of the rule is the copy that drifts. Rows:

    counts     the tree and its pointers are gone
    residue    nothing is left on disk under the prefix except `.py`
    additions  the diff adds no line anywhere
    deletions  every deleted line outside the tree is one the rule selects
    build      each named cargo package still compiles

`counts` is what keeps the two diff rows from passing vacuously: an empty diff
satisfies both of them and fails `counts` immediately, because the tree is
still there.

`counts` has two accepting answers, and needs both. Before the round commits,
the tree directory is gone from disk but the repository still tracks it, so the
prefix resolves and the probe prints the all-zero line. After the round
commits, the repository no longer tracks it either, so the probe refuses the
prefix outright. Accepting only the first answer would make this gate go red on
the exact state it had just approved — which is how a campaign learns to stop
re-running its own gate after integration. What both answers share, and what a
misspelled prefix would not, is the precondition: the prefix must name
something the repository tracked at `--base`.

Usage:

    python3 scripts/td-retire-gate.py --prefix libs/x/tech-design \\
        --base <sha> --package cclab-x

`--prefix` repeats. A child of the campaign may retire several trees at once,
and every row then takes the union: a file left under any prefix is residue, a
deletion is rule-selected if it names any prefix. Naming fewer prefixes than
the round touches does not make the gate laxer — it makes it stricter, because
the deletions aimed at the unnamed trees stop being rule-selected.

`--head <rev>` points the two diff rows at a committed state instead of the
working tree, for re-reading a landed round's diff in isolation. It does not
move `counts`, `residue`, or `build`, which always read the working tree.
"""
from __future__ import annotations

import importlib.util
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
ALL_ZEROS = "md=0 lock=0 py=0 hdr=0 files=0 other=0 embed=0"

# The columns this campaign drives to zero. `py`, `other`, and `embed` are
# held-fixed columns whose correct end value is whatever the tree started with:
# 18 of the 33 trees are also Python TD projects the campaign never retires, and
# a tree can be named by lines the rule deliberately does not select. A gate
# that demanded zero everywhere would be unwinnable for those rounds, and one
# that let the caller name any expectation at all could be handed the base
# measurement and accept a round that did nothing.
DRIVEN_TO_ZERO = ("md", "lock", "hdr", "files")
HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+")

_spec = importlib.util.spec_from_file_location(
    "td_retire_probe", os.path.join(HERE, "td-retire-probe.py"))
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)


def run(args, **kw):
    done = subprocess.run(args, cwd=ROOT, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True, **kw)
    return done.returncode, done.stdout


def under(path, prefixes):
    return any(path == p or path.startswith(p.rstrip("/") + "/") for p in prefixes)


def check_expect(expect):
    """Refuse an expectation that would let the round leave the corpus behind."""
    seen = dict(part.split("=", 1) for part in expect.split())
    missing = [c for c in DRIVEN_TO_ZERO if seen.get(c) != "0"]
    if missing:
        raise SystemExit(
            f"error: --expect must drive {', '.join(DRIVEN_TO_ZERO)} to zero; "
            f"{', '.join(missing)} is not 0 in {expect!r}")
    return expect


def row_counts(prefixes, base, expect):
    tracked = 0
    for prefix in prefixes:
        code, out = run(["git", "-c", "core.fsmonitor=false", "ls-tree", "-r",
                         "--name-only", base, "--", prefix])
        if code != 0 or not out.strip():
            return False, (f"{prefix} named nothing tracked at {base}; a gate "
                           f"whose prefix is a typo passes every row below it")
        tracked += len(out.strip().splitlines())

    code, out = run(["python3", "scripts/td-retire-probe.py",
                     "--expect", expect] + list(prefixes))
    lines = [l for l in out.splitlines() if l.strip()]
    first = lines[0] if lines else "<none>"
    if code == 0 and lines[:1] == [expect]:
        return True, f"tracked_at_base={tracked} counted, matched {expect!r}"
    if code != 0 and not any(l.startswith("md=") for l in lines) and any(
            p in out for p in prefixes):
        return True, f"tracked_at_base={tracked} refused post-commit: {first!r}"
    return False, f"tracked_at_base={tracked} exit={code} first_line={first!r}"


def row_residue(prefixes, base):
    """Under the prefixes: the Markdown corpus is gone and nothing else is.

    The deletable set is exactly what the probe calls `md` plus `lock` — every
    `*.md` and the `td.lock` beside them. Everything else under a tree stays,
    and that is not a formality: eighteen of the repository's thirty-three
    tech-design trees are also Python tech-design projects rooted at the same
    directory, so the tree holds a `pyproject.toml`, a `uv.lock`, a `src/` and
    a `tests/` that this campaign does not retire.

    The survival half is the one thing no other row can see. `deletions` skips
    everything under a prefix by construction, precisely so that the corpus's
    own removal is not reported as one offence per file — which means a round
    that took a `src/service_auth/__init__.py` with it would otherwise leave
    every row green.
    """
    left, missing = [], []
    for prefix in prefixes:
        for dp, dns, fns in os.walk(os.path.join(ROOT, prefix)):
            dns[:] = [d for d in dns if d not in probe.SKIP]
            for fn in fns:
                if fn.endswith(".md") or fn == "td.lock":
                    left.append(os.path.relpath(os.path.join(dp, fn), ROOT))
        code, out = run(["git", "-c", "core.fsmonitor=false", "ls-tree", "-r",
                         "--name-only", base, "--", prefix])
        if code == 0:
            for rel in out.splitlines():
                if rel.endswith(".md") or os.path.basename(rel) == "td.lock":
                    continue
                if not os.path.exists(os.path.join(ROOT, rel)):
                    missing.append(rel)
    ok = not left and not missing
    detail = f"corpus_left={len(left)} non_corpus_deleted={len(missing)}"
    if left:
        detail += f" e.g. {left[0]}"
    if missing:
        detail += f" e.g. deleted {missing[0]}"
    return ok, detail


DIFF_GIT = re.compile(r'^diff --git "?a/(.+?)"? "?b/')


def parse_diff(base, head=None):
    """(added_lines, [(path, old_line_number, text)]) for the round's diff.

    The path comes from the `diff --git` header rather than `+++ b/`, because a
    deleted file's `+++` line is `/dev/null` and every one of its removed lines
    would otherwise be attributed to whichever file was parsed before it.
    """
    args = ["git", "-c", "core.fsmonitor=false", "diff", "--unified=0",
            "--no-color", base] + ([head] if head else []) + ["--"]
    code, out = run(args)
    if code != 0:
        raise SystemExit(f"error: git diff against {base} failed:\n{out}")
    added, removed, path, old = 0, [], None, 0
    for line in out.splitlines():
        m = DIFF_GIT.match(line)
        if m:
            path, old = m.group(1), 0
            continue
        if line.startswith(("--- ", "+++ ")):
            continue
        m = HUNK.match(line)
        if m:
            old = int(m.group(1))
            continue
        if line.startswith("+"):
            added += 1
        elif line.startswith("-"):
            removed.append((path, old, line[1:]))
            old += 1
    return added, removed


def row_additions(added):
    return added == 0, f"added_lines={added}"


def row_deletions(removed, prefixes, base):
    """Every deleted line outside the tree is one the probe's rule selects.

    Three ways a deletion can be wrong and still reach `hdr=0`: the line is not
    a header at all, the line is a header naming a *different* tree, or the
    line only looks like a header because it sits inside a Rust string literal
    — which is source code, and which the probe deliberately does not count.
    """
    masked_cache: dict[str, set[int]] = {}
    bad = []
    for path, n, text in removed:
        if path is None or under(path, prefixes):
            continue
        stripped = text.strip()
        if not probe.HEADER.match(stripped):
            bad.append((path, n, "not a header line", stripped))
            continue
        m = probe.REF.search(text)
        rd = os.path.dirname(path)
        if not m or not probe._hits(m, rd, lambda t: under(t, prefixes)):
            bad.append((path, n, "header names another tree", stripped))
            continue
        if path.endswith(".rs"):
            if path not in masked_cache:
                code, blob = run(["git", "-c", "core.fsmonitor=false", "show",
                                  f"{base}:{path}"])
                masked_cache[path] = probe.literal_lines(blob) if code == 0 else set()
            if n in masked_cache[path]:
                bad.append((path, n, "line lives inside a string literal", stripped))
    detail = f"offending_deletions={len(bad)}"
    if bad:
        p, n, why, text = bad[0]
        detail += f" e.g. {p}:{n} {why}: {text[:60]!r}"
    return not bad, detail


def row_build(package):
    code, out = run(["cargo", "build", "-p", package])
    tail = out.strip().splitlines()[-1] if out.strip() else "<no output>"
    return code == 0, f"exit={code} {tail[:80]!r}"


def main(argv):
    base = head = None
    expect = ALL_ZEROS
    prefixes, packages = [], []
    it = iter(argv)
    for arg in it:
        if arg == "--prefix":
            prefixes.append(next(it, None))
        elif arg == "--expect":
            expect = check_expect(next(it, None))
        elif arg == "--base":
            base = next(it, None)
        elif arg == "--head":
            head = next(it, None)
        elif arg == "--package":
            packages.append(next(it, None))
        else:
            sys.exit(f"error: unknown argument: {arg}")
    if not prefixes or not base:
        sys.exit("error: --prefix and --base are both required")

    rows = [("counts   the trees and their pointers are gone",
              row_counts(prefixes, base, expect)),
            ("residue  the corpus is gone and nothing else is",
             row_residue(prefixes, base))]
    added, removed = parse_diff(base, head)
    rows.append(("additions  the round adds no line", row_additions(added)))
    rows.append(("deletions  every deletion outside the tree is rule-selected",
                 row_deletions(removed, prefixes, base)))
    for package in packages:
        rows.append((f"build    {package} still compiles", row_build(package)))

    failed = 0
    for i, (label, (ok, detail)) in enumerate(rows, 1):
        failed += not ok
        print(f"{'PASS' if ok else 'FAIL'}  {i} {label}: {detail}")
    print(f"rows={len(rows)} failed={failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
