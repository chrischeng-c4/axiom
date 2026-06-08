#!/usr/bin/env python3.12
"""Freeze CPython expected-outputs as committed goldens for the KEEP meter.

The KEEP meter (keep_status.py) classifies behavior/surface/errors/real_world/
security fixtures by comparing mamba stdout against the CPython oracle stdout.
Historically that oracle was spawned LIVE every run (`python3.12 <fixture>`),
coupling the gate's verdict to whatever CPython happens to be installed and
paying a full interpreter startup per fixture. This tool freezes that oracle
once so the meter can read a committed golden instead.

For each fixture under the requested facet dir(s) it runs `python3.12 <fixture>`
TWICE (each in a fresh throwaway cwd, mirroring keep_status's _SCRATCH_CWD
isolation so CPython test.support TESTFN side-effects never leak into the repo)
and classifies the fixture into one of four golden states:

  ORACLE_SKIP     CPython itself errors (rc != 0). The meter already treats these
                  as ORACLE_SKIP, so we record NOTHING — no golden, no list entry.
  NON-DETERMINISTIC  the two runs printed DIFFERENT stdout (random/time/pid/hash
                  ordering). A frozen golden would be a lie, so we record the
                  repo-relative path in config/nondeterministic_goldens.txt and
                  the meter falls back to a live python3.12 spawn for these.
  SELF-CHECKER    stdout is EXACTLY "<case> OK\n". This is the overwhelming common
                  case (fixtures assert internally then print the marker), so we
                  write NO file — the meter DERIVES the expected bytes from the
                  `# case = "..."` header. Zero committed bytes for the common case.
  SIDECAR         any other deterministic stdout. We write the EXACT stdout BYTES
                  to <fixture-dir>/<stem>.expected (trailing newline preserved,
                  bytes not normalized text) and the meter reads them verbatim.

Usage:
    python3.12 tools/gen_goldens.py errors            # whole errors facet
    python3.12 tools/gen_goldens.py errors behavior   # multiple facets
    python3.12 tools/gen_goldens.py behavior --sample 300  # stratified subset
"""

from __future__ import annotations

import argparse
import sys
from collections import Counter
from pathlib import Path

import harness_lib  # shared oracle/SUT runner (tools/): isolated scratch CWD + PYTHONBREAKPOINT=0
from wall_gen_core import parse_header  # shared PEP-723 [tool.mamba] header parse

TOOLS_DIR = Path(__file__).resolve().parent
FIXTURES_DIR = TOOLS_DIR.parents[2] / "cpython"
REPO_ROOT = TOOLS_DIR.parents[5]
NONDET_LIST = TOOLS_DIR / "config" / "nondeterministic_goldens.txt"
ORACLE_SKIP_LIST = TOOLS_DIR / "config" / "oracle_skip_goldens.txt"


def parse_case(fixture: Path) -> str | None:
    """Extract the `case` name from the fixture's PEP-723 [tool.mamba] header."""
    return parse_header(fixture).get("case")


def _repo_rel(fixture: Path) -> str:
    try:
        return str(fixture.resolve().relative_to(REPO_ROOT))
    except ValueError:
        return str(fixture)


def run_oracle(fixture: Path, timeout: int) -> tuple[int | None, bytes]:
    """Run python3.12 <fixture> in a fresh throwaway cwd; capture (rc, stdout-bytes).

    stdout is captured as RAW BYTES (no text decode / newline translation) so the
    frozen golden is byte-identical to what the meter compares against."""
    rc, out, _err = harness_lib.run_fixture(["python3.12", str(fixture)], timeout, text=False)
    return rc, out


def load_list(path: Path) -> set[str]:
    if not path.exists():
        return set()
    out: set[str] = set()
    for ln in path.read_text(encoding="utf-8").splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        out.add(ln)
    return out


_NONDET_HEADER = """\
# Non-deterministic golden fixtures — meter falls back to a LIVE python3.12 spawn.
#
# One repo-relative fixture path per line ('#' comments allowed) — same format and
# discipline as type_divergences.txt / behavior_gaps.txt. gen_goldens.py runs each
# fixture under python3.12 TWICE in fresh cwds; a fixture whose two runs print
# DIFFERENT stdout (random/time/pid/hash-ordering) cannot be frozen into a golden,
# so it is listed here. keep_status.py --oracle golden reads this list and falls
# back to a live python3.12 spawn for these fixtures (identical to --oracle live).
# Everything NOT listed here is frozen: a committed <stem>.expected sidecar, an
# oracle_skip_goldens.txt entry, or — for the common "<case> OK\\n" self-checkers —
# derived from the `# case` header.
"""

_ORACLE_SKIP_HEADER = """\
# Oracle-skip golden fixtures — CPython itself errors (rc != 0), so there is no
# expected output to freeze; the meter treats these as ORACLE_SKIP.
#
# One repo-relative fixture path per line ('#' comments allowed) — same format as
# nondeterministic_goldens.txt. gen_goldens.py records every fixture whose
# python3.12 run exits non-zero here. Without this list, --oracle golden could not
# tell an oracle-skip fixture apart from a "<case> OK\\n" self-checker (both write
# no sidecar) and would mis-derive a golden for it, diverging from --oracle live.
# keep_status.py --oracle golden returns ORACLE_SKIP for every fixture listed here.
"""


def write_list(path: Path, header: str, paths: set[str]) -> None:
    """Merge new paths into a committed config list (idempotent, sorted)."""
    path.parent.mkdir(parents=True, exist_ok=True)
    merged = load_list(path) | paths
    body = "".join(f"{p}\n" for p in sorted(merged))
    path.write_text(header + "\n" + body, encoding="utf-8")


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("facets", nargs="+",
                    help="facet dir name(s) under tests/cpython (e.g. errors behavior)")
    ap.add_argument("--timeout", type=int, default=10)
    ap.add_argument("--sample", type=int, default=None,
                    help="stratified subset: at most N fixtures spread across the facet")
    args = ap.parse_args(argv)

    counts: Counter = Counter()
    nondet_paths: set[str] = set()
    skip_paths: set[str] = set()

    for facet in args.facets:
        root = FIXTURES_DIR / facet
        if not root.exists():
            print(f"!! facet dir not found: {root}", file=sys.stderr)
            continue
        fixtures = sorted(root.rglob("*.py"))
        if args.sample and len(fixtures) > args.sample:
            step = max(1, len(fixtures) // args.sample)
            fixtures = fixtures[::step][: args.sample]

        for fx in fixtures:
            counts["total"] += 1
            case = parse_case(fx)

            rc1, out1 = run_oracle(fx, args.timeout)
            rc2, out2 = run_oracle(fx, args.timeout)

            # CPython itself errors -> ORACLE_SKIP fixture. We RECORD the path (no
            # golden bytes) so --oracle golden can return ORACLE_SKIP instead of
            # mis-deriving a "<case> OK" self-checker for it — without this list a
            # skip is indistinguishable from a self-checker (both write no sidecar).
            # rc-non-determinism (one run 0, the other non-zero) also lands here:
            # the live meter would ORACLE_SKIP it on the 0-run and grade it on the
            # other, so the safe frozen verdict is ORACLE_SKIP.
            if rc1 != 0 or rc2 != 0:
                counts["oracle_skip"] += 1
                skip_paths.add(_repo_rel(fx))
                continue

            if out1 != out2:
                # Same rc 0 both runs but DIFFERENT stdout -> non-deterministic.
                counts["nondeterministic"] += 1
                nondet_paths.add(_repo_rel(fx))
                continue

            # Deterministic, rc==0. Is it a plain "<case> OK\n" self-checker?
            expected_marker = f"{case} OK\n".encode("utf-8") if case is not None else None
            if expected_marker is not None and out1 == expected_marker:
                # Self-checker: meter DERIVES the marker from the header; write NO file.
                counts["self_checker"] += 1
                continue

            # Otherwise freeze the exact stdout bytes as a sidecar.
            sidecar = fx.with_suffix(".expected")
            sidecar.write_bytes(out1)
            counts["sidecar"] += 1

    if nondet_paths:
        write_list(NONDET_LIST, _NONDET_HEADER, nondet_paths)
    if skip_paths:
        write_list(ORACLE_SKIP_LIST, _ORACLE_SKIP_HEADER, skip_paths)

    print("=== gen_goldens summary ===")
    print(f"  total           {counts['total']:6}")
    print(f"  oracle_skip     {counts['oracle_skip']:6}  (CPython rc!=0; listed, no golden)")
    print(f"  nondeterministic{counts['nondeterministic']:6}  (listed -> live fallback)")
    print(f"  self_checker    {counts['self_checker']:6}  (derived '<case> OK'; no file)")
    print(f"  sidecar         {counts['sidecar']:6}  (.expected file written)")
    if nondet_paths:
        print(f"  -> appended {len(nondet_paths)} path(s) to {_repo_rel(NONDET_LIST)}")
    if skip_paths:
        print(f"  -> appended {len(skip_paths)} path(s) to {_repo_rel(ORACLE_SKIP_LIST)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
