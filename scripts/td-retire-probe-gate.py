#!/usr/bin/env python3
"""Gate for `scripts/td-retire-probe.py` — #3721.

The probe answers "how much of this tech-design tree is left" for every child
of the #3694 retirement campaign, and its answer is a gate. This file is the
gate on that gate.

Four invocations, run from the repository root. Two of them must change and two
must not, and it is the pairing that carries the meaning:

    unresolved prefix        must be refused    (row 1)
    unsplit multi-prefix     must be refused    (row 2)
    resolving, counts some   must be accepted   (row 3)
    resolving, counts none   must be accepted   (row 4)

Rows 1 and 4 print the identical line today. A correct change makes them
different; every wrong change discussed on #3721 makes rows 3 or 4 agree with
rows 1 and 2 instead.

Run:  python3 scripts/td-retire-probe-gate.py
Exit: 0 when every row holds, 1 otherwise.
"""
from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ["python3", "scripts/td-retire-probe.py"]
ALL_ZEROS = "md=0 lock=0 py=0 hdr=0 files=0 other=0 embed=0"
COUNT_LINE = re.compile(
    r"^md=(\d+) lock=(\d+) py=(\d+) hdr=(\d+) files=(\d+) other=(\d+) embed=(\d+)$"
)

# A prefix that resolves and counts something, and one that resolves and counts
# nothing. Both are held fixed by this gate; neither is a retirement target of
# #3694, so neither is expected to move under the campaign.
RESOLVES_NONEMPTY = "apps/guard/tech-design"
RESOLVES_EMPTY = "README.md"
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


def refuses(label: str, args: list[str], names: str) -> tuple[str, bool, str]:
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
    return label, ok, detail


def accepts(label: str, args: list[str], want_empty: bool) -> tuple[str, bool, str]:
    """An accepted prefix: exit 0, exactly one count line, of the right kind."""
    code, out, _ = run(args)
    lines = count_lines(out)
    if code != 0 or len(lines) != 1:
        return label, False, f"exit={code} count_lines={len(lines)}"
    columns = [int(g) for g in lines[0].groups()]
    empty = all(c == 0 for c in columns)
    ok = empty is want_empty
    kind = "all-zero" if empty else "nonzero"
    return label, ok, f"exit=0 line={kind} ({lines[0].group(0)})"


def main() -> int:
    rows = [
        refuses(
            "1 unresolved prefix is refused",
            ["--expect", ALL_ZEROS, "no/such/tree"],
            "no/such/tree",
        ),
        refuses(
            "2 unsplit multi-prefix argument is refused",
            [UNSPLIT],
            UNSPLIT,
        ),
        accepts(
            "3 resolving prefix that counts something is accepted",
            [RESOLVES_NONEMPTY],
            want_empty=False,
        ),
        accepts(
            "4 resolving prefix that counts nothing is accepted",
            [RESOLVES_EMPTY],
            want_empty=True,
        ),
    ]
    for label, ok, detail in rows:
        print(f"{'PASS' if ok else 'FAIL'}  {label}: {detail}")
    failed = [r for r in rows if not r[1]]
    print(f"rows={len(rows)} failed={len(failed)}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
