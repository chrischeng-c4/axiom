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
               -- or, under `--whole-tree`, nothing at all
    additions  the diff adds no line anywhere
    deletions  every deleted line outside the tree is one the rule selects
    knowledge  every `src/**/*.rs` that lost a pointer carries a `//!` block
    references every reference from the project's own source roots is gone
               (`--whole-tree` only)
    build      each named cargo package still compiles, with `--features`

The gate has two modes and they refuse each other. Its default is the Markdown
retirement of #3694: the corpus goes, the Python TD project rooted at the same
directory stays. `--whole-tree` is the decision taken for the fifteen
lumen-scope projects on 2026-08-19: the directory goes, Python included. The
mode is declared, never inferred, because the failure it guards against is
silent -- a whole-tree round run in the default mode reaches `md=0 lock=0 hdr=0
files=0` with the `.py`, the `pyproject.toml` and 251 files of generated
evidence still on disk, and every row of this gate is green.

`additions` is why a round has to be split across commits. It requires the diff
to add no line anywhere, so filling a `//!` module doc, moving `tests/` into
`e2e/`, or adding a `[[test]]` stanza cannot share a commit with the deletion
that this gate judges: one commit that both adds and deletes makes `additions`
unwinnable, and relaxing it there would forfeit the only row that can see a
round quietly writing something new while it deletes.

`counts` is what keeps the two diff rows from passing vacuously: an empty diff
satisfies both of them and fails `counts` immediately, because the tree is
still there.

Every row reads the whole diff against `--base`, not the round's own paths, so a
round has to be measured from an otherwise-clean tree. An unrelated modified
file elsewhere in the repository is added lines and unselected deletions, and
`additions` and `deletions` go red for a reason that has nothing to do with the
tree being judged. That is the strictness the rows are for; it is not something
to narrow to a path filter, because the filter would be the thing that let a
round edit source it was not supposed to touch.

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

    python3 scripts/td-retire-gate.py --whole-tree \\
        --prefix libs/x/tech-design --prefix libs/x/external-contracts \\
        --base <sha> --package cclab-x --features operator

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

import hashlib
import importlib.util
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
HUNK = re.compile(r"^@@ -(\d+)(?:,(\d+))? \+")

_spec = importlib.util.spec_from_file_location(
    "td_retire_probe", os.path.join(HERE, "td-retire-probe.py"))
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)

# Built from the probe's own column list, never restated: #3746 is this literal
# frozen at seven columns while the probe grew to twelve, which no tree could
# then match, and the first completed retirement would have been the first run
# to find out.
COUNT_LINE = re.compile("^" + " ".join(c + r"=(\d+)" for c in probe.COLUMNS)
                        + "$")

# The columns the MARKDOWN campaign drives to zero. `py`, `tdrest`, `ec`,
# `ecrest`, `ecev`, `other` and `embed` are held-fixed columns whose correct end
# value is whatever the tree started with: 18 of the 33 trees are also Python TD
# projects that campaign never retires, and a tree can be named by lines the
# rule deliberately does not select. A gate that demanded zero everywhere would
# be unwinnable for those rounds, and one that let the caller name any
# expectation at all could be handed the base measurement and accept a round
# that did nothing.
DRIVEN_TO_ZERO = ("md", "lock", "hdr", "files")

# The columns the WHOLE-TREE retirement of the fifteen lumen-scope projects
# drives to zero: every population, because the decision is that the directory
# goes. Only three columns are not in it, and each for a stated reason --
# `ecdirs` and `ecscan` count what the walk reached rather than what survives,
# and `other` is prose and string literals the deletion rule does not select,
# whose surviving count `--expect` pins line by line instead.
#
# The two modes refuse each other. A whole-tree round run without
# `--whole-tree` reaches `hdr=0 files=0 md=0 lock=0` while leaving the Python,
# the `pyproject.toml` and 251 evidence files on disk, and every row is green.
# A Markdown round run WITH it cannot pass at all.
WHOLE_TREE_ZEROS = tuple(
    c for c in probe.COLUMNS
    if c not in ("ecdirs", "ecscan", "other"))


def run(args, **kw):
    done = subprocess.run(args, cwd=ROOT, stdout=subprocess.PIPE,
                          stderr=subprocess.STDOUT, text=True, **kw)
    return done.returncode, done.stdout


def under(path, prefixes):
    return any(path == p or path.startswith(p.rstrip("/") + "/") for p in prefixes)


def check_expect(expect, whole_tree):
    """Refuse a hand-written `--expect` that would let the corpus stay behind.

    `--expect` is optional and pins the probe's whole line, liveness columns
    included. Nothing needs it: `row_counts` reads the columns it cares about
    out of the line. It is here for a round that wants to freeze the other
    columns' actuals as well, and it is validated because an expectation that
    does not zero the mode's own columns is an expectation that approves the
    round it was written to judge.
    """
    required = WHOLE_TREE_ZEROS if whole_tree else DRIVEN_TO_ZERO
    seen = dict(part.split("=", 1) for part in expect.split())
    unknown = [c for c in seen if c not in probe.COLUMNS]
    if unknown or len(seen) != len(probe.COLUMNS):
        raise SystemExit(
            f"error: --expect must name every column exactly once "
            f"({' '.join(probe.COLUMNS)}); got {expect!r}")
    missing = [c for c in required if seen.get(c) != "0"]
    if missing:
        raise SystemExit(
            f"error: --expect must drive {', '.join(required)} to zero; "
            f"{', '.join(missing)} is not 0 in {expect!r}")
    return expect


def row_counts(prefixes, base, expect, whole_tree):
    """The trees are gone, and so is every pointer into them.

    Read off the probe's own count line rather than a string equality against
    it. Two of the sixteen columns -- `ecdirs` and `ecscan` -- report what the
    walk reached, not what the prefix holds, so they are nonzero for every
    prefix and cannot appear in an expectation written before the walk. A gate
    that demanded the whole line be all-zeros was a gate that went red on a
    finished round, and the only way to quiet it was to paste this run's
    liveness actuals into the next round's expectation.
    """
    tracked = 0
    for prefix in prefixes:
        code, out = run(["git", "-c", "core.fsmonitor=false", "ls-tree", "-r",
                         "--name-only", base, "--", prefix])
        if code != 0 or not out.strip():
            return False, (f"{prefix} named nothing tracked at {base}; a gate "
                           f"whose prefix is a typo passes every row below it")
        tracked += len(out.strip().splitlines())

    required = WHOLE_TREE_ZEROS if whole_tree else DRIVEN_TO_ZERO
    args = ["python3", "scripts/td-retire-probe.py"]
    if expect is not None:
        args += ["--expect", expect]
    code, out = run(args + list(prefixes))
    lines = [l for l in out.splitlines() if l.strip()]
    first = lines[0] if lines else "<none>"
    counted = [m for m in (COUNT_LINE.match(l) for l in lines) if m]
    if counted:
        got = dict(zip(probe.COLUMNS, (int(g) for g in counted[0].groups())))
        nonzero = [f"{c}={got[c]}" for c in required if got[c] != 0]
        if nonzero:
            return False, (f"tracked_at_base={tracked} still nonzero: "
                           f"{' '.join(nonzero)}")
        if expect is not None and code != 0:
            return False, (f"tracked_at_base={tracked} required columns are zero "
                           f"but --expect {expect!r} does not match {first!r}")
        return True, (f"tracked_at_base={tracked} counted, "
                      f"{len(required)} required columns at zero")
    if code != 0 and any(p in out for p in prefixes):
        return True, f"tracked_at_base={tracked} refused post-commit: {first!r}"
    return False, f"tracked_at_base={tracked} exit={code} first_line={first!r}"


def row_residue_whole(prefixes):
    """Under the prefixes: nothing at all is left, of any filename shape.

    The Markdown row below asks a subtractive question -- is the corpus gone and
    is everything else still here -- and it cannot be reused here, because under
    the whole-tree decision "everything else" is what the round is deleting.

    What that row's survival half protected against is protected here by
    `deletions` instead: a round that also took a file OUTSIDE a prefix removes
    its lines, and every removed line outside a prefix has to be a header the
    rule selects. A wholesale out-of-tree deletion is dozens of lines that are
    not.

    `left` counts every remaining file, not the three filename shapes the probe
    knew before this mode existed. #3737 is a retired `apps/meter/tech-design`
    surviving as one `.gitkeep`, still resolving as a prefix and still in the
    census, with `md`, `lock` and `py` all at zero.
    """
    left = []
    for prefix in prefixes:
        for dp, dns, fns in os.walk(os.path.join(ROOT, prefix)):
            dns[:] = [d for d in dns if d not in probe.SKIP]
            for fn in fns:
                left.append(os.path.relpath(os.path.join(dp, fn), ROOT))
        if os.path.isdir(os.path.join(ROOT, prefix)):
            left.append(prefix + "/  (the directory itself)")
    detail = f"files_left={len(left)}"
    if left:
        detail += f" e.g. {left[0]}"
    return not left, detail


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

    The selection test is `probe.selects`, the same predicate the probe counts
    with, so this row measures the rule instead of restating it. It covers
    external-contracts targets as well as tech-design ones: 104 of the header
    lines the lumen scope has to strip live in `apps/lumen/tests/` and name an
    `external-contracts/` path, and against `probe.REF` alone every one of them
    would be reported here as a deletion the rule does not select.
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
        rd = os.path.dirname(path)
        if not probe.selects(stripped, rd, lambda t: under(t, prefixes)):
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


DOC = re.compile(r"^\s*//!(.*)$")
# A project's own source roots: where naming a retired tree is a dangling
# pointer rather than a record of one. Everything else -- this campaign's
# account of itself, ADRs, tracker bodies, the repository META docs -- is
# allowed to name what it retired, which is why this row is scoped instead of
# asking the probe's `other` column to reach zero.
SOURCE_DIRS = ("src", "e2e", "tests", "benches", "examples", "k8s")
SOURCE_FILES = ("Cargo.toml",)


def row_knowledge(removed, prefixes, base):
    """Every `src/**/*.rs` that lost a pointer still carries a `//!` block.

    This is the row that makes D3 refusable. Deleting a header is deleting a
    pointer to prose, and the decision taken on 2026-08-19 is that the prose
    lands in the module doc of the `.rs` that owns it. Without this row nothing
    can tell that round from one that deleted the pointer and the knowledge
    together: `counts`, `residue`, `additions`, `deletions` and `build` are all
    green either way, because a `.rs` file with no `//!` compiles.

    Exempt, and counted out loud rather than dropped silently:

      * a file the round deleted outright -- there is nothing left to carry it;
      * a `.rs` outside a `src/` tree. `apps/lumen/e2e/` and `apps/lumen/tests/`
        hold 106 files whose headers name an `external-contracts/` path, and a
        case is not the owner of a rule: it is a reader of one. Requiring a
        module doc there would move the campaign's knowledge into the very
        files S3 is rewriting.
    """
    lost = set()
    for path, _n, text in removed:
        if path is None or under(path, prefixes) or not path.endswith(".rs"):
            continue
        if probe.selects(text.strip(), os.path.dirname(path),
                         lambda t: under(t, prefixes)):
            lost.add(path)
    gone = {p for p in lost if not os.path.exists(os.path.join(ROOT, p))}
    non_src = {p for p in lost - gone if "/src/" not in "/" + p}
    owed = sorted(lost - gone - non_src)
    bad = []
    for rel in owed:
        with open(os.path.join(ROOT, rel), encoding="utf-8",
                  errors="replace") as fh:
            body = [m.group(1).strip() for m in
                    (DOC.match(l) for l in fh) if m]
        if not any(body):
            bad.append(rel)
    detail = (f"lost_pointers={len(lost)} owed_doc={len(owed)} "
              f"without_doc={len(bad)} exempt_deleted={len(gone)} "
              f"exempt_non_src={len(non_src)}")
    if bad:
        detail += f" e.g. {bad[0]}"
    return not bad, detail


def row_references(prefixes):
    """Nothing under the project's own source roots still names a retired tree.

    `hdr` and `echdr` count well-formed headers, and `deletions` judges lines the
    round removed. Neither can see a reference that was never a comment. The
    measured case is `apps/lumen/k8s/operator/crd.yaml`: 13 pointers into
    `apps/lumen/tech-design/semantic/source/` and `libs/service-k8s/tech-design/`,
    all of them inside YAML `description:` strings that `schemars` folded out of
    the `///` doc comments on `operator/crd.rs`. Four of the 13 are a whole
    description, so stripping the doc comment does not shorten those lines -- it
    removes the `description:` key. The CRD is regenerated by its renderer, never
    line-edited, and because that is a write it belongs to a different commit
    from the deletion this gate judges.

    Whole-tree mode only. While a tree is still on disk its own Python names its
    own paths, and every one of those would be reported here.
    """
    roots = sorted({os.path.dirname(p.rstrip("/")) for p in prefixes})
    scanned, bad = 0, []
    for root in roots:
        targets = [os.path.join(ROOT, root, d) for d in SOURCE_DIRS]
        targets += [os.path.join(ROOT, root, f) for f in SOURCE_FILES]
        for target in targets:
            if os.path.isfile(target):
                walk = [(os.path.dirname(target), [], [os.path.basename(target)])]
            elif os.path.isdir(target):
                walk = os.walk(target)
            else:
                continue
            for dp, dns, fns in walk:
                dns[:] = [d for d in dns if d not in probe.SKIP]
                for fn in fns:
                    fp = os.path.join(dp, fn)
                    text = probe.read_text(fp)
                    if text is None:
                        continue
                    scanned += 1
                    rel = os.path.relpath(fp, ROOT)
                    if under(rel, prefixes):
                        continue
                    for i, line in enumerate(text.splitlines(), 1):
                        if any(pref in line for pref in prefixes):
                            bad.append((rel, i, line.strip()))
    detail = f"scanned_files={scanned} dangling={len(bad)}"
    if bad:
        rel, i, line = bad[0]
        detail += f" e.g. {rel}:{i} {line[:60]!r}"
    return not bad, detail


# Hazard 23's consumer. `reason="…"` on a header is prose, not a marker: 76 of the
# 97 in lumen scope are 15 words or more, and some are the only statement of the
# fact anywhere in the tree. S4 deletes the line they sit on, and rows 1-7 are all
# green either way -- `knowledge` asks whether the file that lost a POINTER still
# has a `//!`, never whether the header's own sentence survived.
LEDGER = "apps/lumen/docs/td-ec-reason-ledger.tsv"
REASON = re.compile(r'reason="([^"]*)"')
MIN_REASON_WORDS = 15
MIN_EVIDENCE_LINES = 2
VERDICTS = ("disposable", "merged", "deferred:S3")


def _doc_run(rel, lo, hi, marker):
    """The stripped lines of a claimed evidence run, or None if it is not one."""
    path = os.path.join(ROOT, rel)
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8", errors="replace") as fh:
        lines = fh.read().splitlines()
    if lo < 1 or hi > len(lines) or hi - lo + 1 < MIN_EVIDENCE_LINES:
        return None
    run = [l.strip() for l in lines[lo - 1:hi]]
    return run if all(l.startswith(marker) for l in run) else None


def _find_run(rel, marker, sha, lo, hi):
    """Locate the evidence by CONTENT, using the recorded range only as a hint.

    S4 strips header lines from the same files that carry the evidence, so every
    run below a stripped header shifts up in the very round this row runs in. A
    line-number anchor would fail the whole ledger for a reason that has nothing
    to do with the knowledge; a sha anchor still refuses evidence that was edited
    away, deleted, or never pointed at a doc comment in the first place.
    """
    exact = _doc_run(rel, lo, hi, marker)
    if exact is not None and _sha12("\n".join(exact)) == sha:
        return "exact"
    path = os.path.join(ROOT, rel)
    if not os.path.exists(path):
        return None
    with open(path, encoding="utf-8", errors="replace") as fh:
        lines = [l.strip() for l in fh.read().splitlines()]
    n = hi - lo + 1
    for start in range(0, max(0, len(lines) - n + 1)):
        window = lines[start:start + n]
        if all(l.startswith(marker) for l in window) and _sha12("\n".join(window)) == sha:
            return f"shifted:{start + 1}-{start + n}"
    return None


def _sha12(text):
    return hashlib.sha256(text.encode()).hexdigest()[:12]


def row_reasons(removed, prefixes, ledger_rel=LEDGER):
    """Every long `reason=` this round deleted is adjudicated, and still true.

    The ledger is written by a human; this row is what makes writing it
    non-optional and what makes a stale entry visible. It deliberately does NOT
    distinguish `merged` from `disposable` -- both claim the knowledge lives in a
    named doc-comment run, and both are checked the same way. That distinction is
    for the campaign's history, not for the gate.
    """
    ledger_path = os.path.join(ROOT, ledger_rel)
    if not os.path.exists(ledger_path):
        return False, f"ledger missing: {ledger_rel}"
    entries = {}
    with open(ledger_path, encoding="utf-8") as fh:
        header = fh.readline().rstrip("\n").split("\t")
        need = ("file", "line", "verdict", "evidence", "evidence_sha", "reason")
        if any(c not in header for c in need):
            return False, f"ledger header missing columns: {header}"
        idx = {c: header.index(c) for c in header}
        for raw in fh:
            f = raw.rstrip("\n").split("\t")
            if len(f) < len(header):
                continue
            entries[(f[idx["file"]], f[idx["reason"]])] = (
                f[idx["verdict"]], f[idx["evidence"]], f[idx["evidence_sha"]])

    deleted = []
    for path, _n, text in removed:
        if path is None or under(path, prefixes) or not path.endswith(".rs"):
            continue
        if "HANDWRITE" not in text and "SPEC-MANAGED" not in text:
            continue
        m = REASON.search(text)
        if m and len(m.group(1).split()) >= MIN_REASON_WORDS:
            deleted.append((path, m.group(1)))

    missing, bad_verdict, bad_evidence, stale_deferred, exempt_gone = [], [], [], [], []
    for key in deleted:
        if key not in entries:
            missing.append(key)
            continue
        verdict, evidence, sha = entries[key]
        if verdict not in VERDICTS:
            bad_verdict.append((key, verdict))
            continue
        if verdict == "deferred:S3":
            path = os.path.join(ROOT, key[0])
            if not os.path.exists(path):
                exempt_gone.append(key)
                continue
            with open(path, encoding="utf-8", errors="replace") as fh:
                if any(DOC.match(l) for l in fh):
                    stale_deferred.append(key)
            continue
        try:
            rel, span, marker = evidence.rsplit(":", 2)
            lo, hi = (int(x) for x in span.split("-"))
        except ValueError:
            bad_evidence.append((key, f"unparsable evidence {evidence!r}"))
            continue
        if _find_run(rel, marker, sha, lo, hi) is None:
            bad_evidence.append((key, f"no run matching {sha} near {evidence}"))

    bad = missing or bad_verdict or bad_evidence or stale_deferred
    detail = (f"deleted_reasons={len(deleted)} "
              f"covered={len(deleted) - len(missing)}/{len(deleted)} "
              f"bad_verdict={len(bad_verdict)} bad_evidence={len(bad_evidence)} "
              f"stale_deferred={len(stale_deferred)} exempt_deleted={len(exempt_gone)}")
    if missing:
        detail += f" e.g. unadjudicated {missing[0][0]}: {missing[0][1][:48]!r}"
    elif bad_evidence:
        detail += f" e.g. {bad_evidence[0][0][0]}: {bad_evidence[0][1]}"
    elif stale_deferred:
        detail += f" e.g. deferred-but-now-documented {stale_deferred[0][0]}"
    elif bad_verdict:
        detail += f" e.g. {bad_verdict[0][0][0]}: verdict {bad_verdict[0][1]!r}"
    return not bad, detail


def row_build(package, features):
    """The named crate still compiles, with the features that compile the code.

    `--features` is not decoration. `apps/lumen/Cargo.toml` sets `default = []`
    and puts `service-k8s` behind the `operator` feature, so a bare
    `cargo build -p lumen` does not compile the operator at all: it returns in
    under a second, green, having built none of the 85 files whose headers this
    campaign strips. Whether a feature set actually compiled anything is read
    off `Compiling <crate>` in the output, not off the exit code.
    """
    args = ["cargo", "build", "-p", package]
    if features:
        args += ["--features", features]
    code, out = run(args)
    compiled = sum(1 for l in out.splitlines() if l.strip().startswith("Compiling "))
    tail = out.strip().splitlines()[-1] if out.strip() else "<no output>"
    return code == 0, f"exit={code} compiling={compiled} {tail[:80]!r}"


def main(argv):
    base = head = expect = None
    whole_tree = False
    features = ""
    ledger = LEDGER
    prefixes, packages = [], []
    it = iter(argv)
    for arg in it:
        if arg == "--prefix":
            prefixes.append(next(it, None))
        elif arg == "--expect":
            expect = next(it, None)
        elif arg == "--base":
            base = next(it, None)
        elif arg == "--head":
            head = next(it, None)
        elif arg == "--package":
            packages.append(next(it, None))
        elif arg == "--features":
            features = next(it, None)
        elif arg == "--ledger":
            ledger = next(it, None)
        elif arg == "--whole-tree":
            whole_tree = True
        else:
            sys.exit(f"error: unknown argument: {arg}")
    if not prefixes or not base:
        sys.exit("error: --prefix and --base are both required")
    # Order matters: --expect is validated after the whole argument list is
    # read, because which columns it must zero depends on --whole-tree, and a
    # flag that only takes effect when it precedes the expectation is a flag
    # that silently does nothing half the time.
    if expect is not None:
        expect = check_expect(expect, whole_tree)

    rows = [("counts   the trees and their pointers are gone",
              row_counts(prefixes, base, expect, whole_tree))]
    if whole_tree:
        rows.append(("residue  nothing at all is left under the prefixes",
                     row_residue_whole(prefixes)))
    else:
        rows.append(("residue  the corpus is gone and nothing else is",
                     row_residue(prefixes, base)))
    added, removed = parse_diff(base, head)
    rows.append(("additions  the round adds no line", row_additions(added)))
    rows.append(("deletions  every deletion outside the tree is rule-selected",
                 row_deletions(removed, prefixes, base)))
    rows.append(("knowledge  every src/**.rs that lost a pointer has a //! doc",
                 row_knowledge(removed, prefixes, base)))
    if whole_tree:
        rows.append(("references  no source root still names a retired tree",
                     row_references(prefixes)))
        rows.append(("reasons  every long reason= the round deletes is adjudicated",
                     row_reasons(removed, prefixes, ledger)))
    for package in packages:
        rows.append((f"build    {package} still compiles",
                     row_build(package, features)))

    failed = 0
    for i, (label, (ok, detail)) in enumerate(rows, 1):
        failed += not ok
        print(f"{'PASS' if ok else 'FAIL'}  {i} {label}: {detail}")
    print(f"rows={len(rows)} failed={failed}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
