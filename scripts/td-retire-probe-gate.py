#!/usr/bin/env python3
"""Gate for `scripts/td-retire-probe.py` — #3721.

The probe answers "how much of this tech-design tree is left" for every child
of the #3694 retirement campaign, and its answer is a gate. This file is the
gate on that gate.

Nine rows, run from the repository root, in two groups.

Rows 1-4 are about the probe's answer to an argument, and it is the pairing that
carries the meaning:

    unresolved prefix        must be refused    (row 1)
    unsplit multi-prefix     must be refused    (row 2)
    resolving, counts some   must be accepted   (row 3)
    resolving, counts none   must be accepted   (row 4)

Rows 1 and 4 print the identical line today. A correct change makes them
different; every wrong change discussed on #3721 makes rows 3 or 4 agree with
rows 1 and 2 instead.

Rows 5-7 are about the four columns the whole-tree retirement added, and they
exist because rows 1-4 cannot see a column: `accepts` only tells an all-zero
line from a nonzero one, so folding `ecev` into `ecrest`, or dropping `echdr`
entirely, leaves all four green. Each is stated against a pair of trees that
differ in one respect, so a reading a wrong implementation would also produce
fails the other half:

    tech-design tree, no Markdown left, still holds files      (row 5)
    external-contracts tree with evidence / without it         (rows 6a, 6b)
    external-contracts tree headers point at / nothing does    (rows 7a, 7b)

Run:  python3 scripts/td-retire-probe-gate.py
Exit: 0 when every row holds, 1 otherwise.

Each row is one full repository walk, so the whole gate takes roughly three
minutes -- longer than a default command timeout. `--only <label-prefix>` runs
the rows whose label starts with the given string, for measuring one row's
negative control without paying for the other eight. A run under `--only` is
NOT this gate: it prints `selected=` and the row count it skipped, because a
green subset reported as a green gate is the failure this file exists to catch,
one level up.
"""
from __future__ import annotations

import importlib.util
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ["python3", "scripts/td-retire-probe.py"]

# The column list is read out of the probe, never restated. Both defects this
# gate has itself carried -- #3746's seven-column ALL_ZEROS and #3762's
# seven-column parse -- were a literal that stopped matching the probe beside
# it, and a gate whose parse silently misses is a gate that passes vacuously.
_spec = importlib.util.spec_from_file_location(
    "td_retire_probe", str(ROOT / "scripts" / "td-retire-probe.py"))
probe = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(probe)
COLUMNS = probe.COLUMNS
ALL_ZEROS = " ".join(c + "=0" for c in COLUMNS)
COUNT_LINE = re.compile("^" + " ".join(c + r"=(\d+)" for c in COLUMNS) + "$")
# Walk-liveness columns, not populations: they count what the walk reached
# regardless of the prefix, so they are nonzero for every prefix including one
# that names nothing. Excluded by NAME, because their position moves whenever a
# column is added and a positional literal would then exclude the wrong two.
LIVENESS = ("ecdirs", "ecscan")

# A prefix that resolves and counts something, and one that resolves and counts
# nothing. Both are held fixed by this gate; neither is a retirement target of
# #3694, so neither is expected to move under the campaign.
RESOLVES_NONEMPTY = "apps/guard/tech-design"
RESOLVES_EMPTY = "README.md"
# Two external-contracts trees that differ in exactly one respect each, so the
# columns added for the whole-tree retirement have a pair behind them rather
# than a single reading any wrong implementation could also produce. Neither is
# a retirement target: `guard` holds generated evidence and nothing points at
# it, `defer` is pointed at by 33 header lines and holds no evidence.
EC_WITH_EVIDENCE = "apps/guard/external-contracts"
EC_WITHOUT_EVIDENCE = "apps/defer/external-contracts"
# Two prefixes that each resolve on their own. As one argument they name a path
# that does not exist -- what a shell that does not word-split produces.
UNSPLIT = f"{RESOLVES_NONEMPTY} libs/build-stamp/tech-design"


def run(args: list[str]) -> tuple[int, str, str]:
    done = subprocess.run(
        PROBE + args, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        text=True,
    )
    return done.returncode, done.stdout, done.stderr


def count_lines(stdout: str) -> list[re.Match[str]]:
    return [m for m in (COUNT_LINE.match(l) for l in stdout.splitlines()) if m]


def refuses(args: list[str], names: str) -> tuple[bool, str]:
    """A refused prefix: nonzero exit, no count line, and it says which one."""
    code, out, err = run(args)
    lines = count_lines(out)
    ok = code != 0 and not lines
    detail = f"exit={code} count_lines={len(lines)}"
    # The refusal has to be actionable. A probe that refuses without naming the
    # offending prefix sends its caller to the wrong argument.
    if ok and names not in out + err:
        ok = False
        detail += f" (refusal does not name {names!r})"
    return ok, detail


def columns(args: list[str]) -> dict[str, int] | None:
    """The one count line a resolving prefix prints, as {column: value}."""
    code, out, _ = run(args)
    lines = count_lines(out)
    if code != 0 or len(lines) != 1:
        return None
    return dict(zip(COLUMNS, (int(g) for g in lines[0].groups())))


def accepts(args: list[str], want_empty: bool) -> tuple[bool, str]:
    """An accepted prefix: exit 0, exactly one count line, of the right kind."""
    got = columns(args)
    if got is None:
        return False, "no single count line"
    populations = [v for c, v in got.items() if c not in LIVENESS]
    empty = all(v == 0 for v in populations)
    kind = "all-zero" if empty else "nonzero"
    return empty is want_empty, f"exit=0 line={kind}"


def pins(args: list[str], claim, shown: tuple[str, ...]) -> tuple[bool, str]:
    """A column-level claim about one prefix, reported with the columns it read.

    `accepts` can only tell an all-zero line from a nonzero one, so it cannot
    see a column folded into its neighbour or dropped altogether. These rows
    name the columns instead.
    """
    got = columns(args)
    if got is None:
        return False, "no single count line"
    return bool(claim(got)), " ".join(f"{c}={got[c]}" for c in shown)


# (label, thunk). A table rather than a sequence of calls, so `--only` can
# select a row without evaluating the other eight and without a second copy of
# any row's claim living in whatever script measures its control.
ROWS: list[tuple[str, object]] = [
    ("1 unresolved prefix is refused",
     lambda: refuses(["--expect", ALL_ZEROS, "no/such/tree"], "no/such/tree")),
    ("2 unsplit multi-prefix argument is refused",
     lambda: refuses([UNSPLIT], UNSPLIT)),
    ("3 resolving prefix that counts something is accepted",
     lambda: accepts([RESOLVES_NONEMPTY], want_empty=False)),
    ("4 resolving prefix that counts nothing is accepted",
     lambda: accepts([RESOLVES_EMPTY], want_empty=True)),
    ("5 a tree with no Markdown left is still not empty",
     lambda: pins([RESOLVES_NONEMPTY],
                  lambda c: c["md"] == 0 and c["lock"] == 0 and c["tdrest"] > 0,
                  ("md", "lock", "py", "tdrest"))),
    ("6a generated evidence is counted, and not as ecrest",
     lambda: pins([EC_WITH_EVIDENCE],
                  lambda c: c["ecev"] > 0 and c["ecev"] != c["ecrest"],
                  ("ec", "ecrest", "ecev"))),
    ("6b a tree with no evidence borrows none",
     lambda: pins([EC_WITHOUT_EVIDENCE],
                  lambda c: c["ecev"] == 0 and c["ecrest"] > 0,
                  ("ec", "ecrest", "ecev"))),
    ("7a headers into an external-contracts tree are counted, apart from hdr",
     lambda: pins([EC_WITHOUT_EVIDENCE],
                  lambda c: c["echdr"] > 0 and c["ecfiles"] > 0 and c["hdr"] == 0,
                  ("hdr", "files", "echdr", "ecfiles"))),
    ("7b a tree no header names gets echdr=0",
     lambda: pins([EC_WITH_EVIDENCE],
                  lambda c: c["echdr"] == 0 and c["ecfiles"] == 0,
                  ("hdr", "echdr", "ecfiles"))),
]


def main(argv: list[str]) -> int:
    only = None
    it = iter(argv)
    for arg in it:
        if arg == "--only":
            only = next(it, None)
        else:
            sys.exit(f"error: unknown argument: {arg}")
    selected = [(l, t) for l, t in ROWS if only is None or l.startswith(only)]
    if not selected:
        sys.exit(f"error: --only {only!r} selected none of {len(ROWS)} rows")
    failed = 0
    for label, thunk in selected:
        ok, detail = thunk()
        failed += not ok
        print(f"{'PASS' if ok else 'FAIL'}  {label}: {detail}")
    tail = f"rows={len(selected)} failed={failed}"
    if only is not None:
        tail += f" selected={only!r} skipped={len(ROWS) - len(selected)}"
    print(tail)
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
