#!/usr/bin/env python3.12
"""WALL coverage meter — how complete the test corpus is against each denominator.

Phase 1 of the full-completion goal (先築牆): every denominator item must have a
test case, even if mamba fails it (red is correct). This prints coverage % per
dimension; the WALL is done when all four reach 100%. Pass-rate (the KEEP /
Phase 2) is a separate meter (gate_status.py) and must not be touched until the
wall is complete.

  ① Type      = type cases / typeshed stdlib typed signatures
  ② Behavior  = behavior fixtures / CPython 3.12 Lib/test methods   (denominator TODO)
  ③ Perf      = bench fixtures / pyperformance benchmarks           (denominator TODO)
  ④ Safety    = safety cases / (error-path × secret-class) matrix   (denominator TODO)

    python3.12 tests/harness/cpython/tools/wall_status.py

A coverage % can't be gamed: the denominator is an external standard
(typeshed / Lib/test / pyperformance), not something we control.
"""

from __future__ import annotations

import sys
from pathlib import Path

MAMBA_DIR = Path(__file__).resolve().parents[4]   # projects/mamba
TYPESHED_STDLIB = MAMBA_DIR / "vendor" / "typeshed" / "stdlib"
CPYTHON_DIR = MAMBA_DIR / "tests" / "cpython"
BEHAVIOR_GAPS = MAMBA_DIR / "tests" / "harness" / "cpython" / "config" / "behavior_gaps.txt"

sys.path.insert(0, str(Path(__file__).resolve().parent))
import type_wall_gen  # type: ignore[import-not-found]  # noqa: E402 — defines ① denominator
import behavior_wall_gen  # type: ignore[import-not-found]  # noqa: E402 — defines ② denominator
import perf_wall_gen  # type: ignore[import-not-found]  # noqa: E402 — defines ③ denominator


def type_wall_signatures_and_cases() -> tuple[int, int]:
    """① Type denominator & numerator, both derived from the wall generator.

    Denominator = public typeshed wrongable signatures the generator recognizes —
    each has some positional (excluding self) with a violable type contract.
    Private and object/Any/0-param-only signatures are excluded: they carry no
    contract to test, so no wrong-typed-arg case exists for them (counting them
    would be dishonest; the set is still typeshed-derived, not ours).

    Numerator = those signatures that have a generated case file. Overload /
    version variants that collapse onto one shared case file each still count as
    covered — every signature is covered, even when several map to one file (the
    union contract one case tests). This counts COVERED SIGNATURES, not unique
    files, so collapse doesn't understate coverage.
    """
    cands = list(type_wall_gen.candidates({"module", "init", "smethod", "method"}))
    total = len(cands)
    covered = sum(1 for c in cands
                  if (type_wall_gen.OUT_DIR / type_wall_gen.render(c)[0]).exists())
    return total, covered


def _load_behavior_gaps() -> set[str]:
    """The honest denominator exclusions: cases the live CPython 3.12 oracle
    itself does not pass (CPython-fail / resource / platform skip), recorded by
    behavior_extract.py. Stored as `mod.Class.method` (Class may be dotted for
    nested classes); reduce each to the innermost `Class.method` key so it lines
    up with the wall denominator's keying."""
    if not BEHAVIOR_GAPS.exists():
        return set()
    keys: set[str] = set()
    for ln in BEHAVIOR_GAPS.read_text(encoding="utf-8").splitlines():
        ln = ln.strip()
        if not ln or ln.startswith("#"):
            continue
        segs = ln.split(".")
        if len(segs) >= 2:
            keys.add(f"{segs[-2]}.{segs[-1]}")
    return keys


def behavior_signatures_and_cases() -> tuple[int, int, int]:
    """② Behavior denominator & numerator — the existing behavior dimension IS the wall.

    Denominator = class-bound Lib/test methods (mod.Class.method keys), recursing
    into nested classes and If-version blocks; module-level/function-local test*
    are excluded (not unittest test cases) — MINUS the recorded CPython-fail gaps,
    which are not mamba's to pass and so leave the denominator (the same way ①
    Type excludes unwrongable signatures). Numerator = methods covered by an
    existing behavior-dimension fixture (matched by subject) across all buckets.
    Those auto-ported fixtures inline the test body (or run it via the unittest
    loader) and ARE the Lib/test wall; there is no separate libtest/ tree.

    Returns (passable_total, covered, excluded).
    """
    testdir = behavior_wall_gen.lib_test_dir()
    if testdir is None:
        return 0, 0, 0
    # Key on (innermost class, method) — robust to the existing fixtures' varied
    # subject module paths (top-level test_X vs test.subpkg.test_Y vs __init__).
    def key(cls: str, method: str) -> str:
        return f"{cls.split('.')[-1]}.{method}"
    wanted = {key(cls, method)
              for _, cls, method in behavior_wall_gen.candidates(testdir)}
    gaps = _load_behavior_gaps() & wanted   # only gaps that are real denominator keys
    passable = wanted - gaps
    covered = set()
    for py in CPYTHON_DIR.rglob("*.py"):
        # Dimension-first layout: a behavior fixture's FACET is the first path
        # segment under tests/cpython (behavior/{bucket}/{lib}/{case}.py), not its
        # parent dir name (which is now the lib dir).
        rel_parts = py.relative_to(CPYTHON_DIR).parts
        if not rel_parts or rel_parts[0] != "behavior":
            continue
        subj = ""
        with py.open(encoding="utf-8", errors="replace") as fh:
            for line in fh:
                if line.startswith("# subject = "):
                    parts = line.split('"')
                    subj = parts[1] if len(parts) > 1 else ""
                    break
                if not line.startswith("#") and line.strip():
                    break  # past the header
        segs = subj.split(".")
        if len(segs) >= 2:
            k = f"{segs[-2]}.{segs[-1]}"
            if k in passable:
                covered.add(k)
    return len(passable), len(covered), len(gaps)


def perf_signatures_and_cases() -> tuple[int, int]:
    """③ Perf denominator & numerator, both from the wall generator.

    Denominator = pyperformance workload programs (bm_*/run_benchmark.py) — the
    community's real-workload bar (the manifest's 97 names are arg-variants of
    these). Numerator = those with a generated self-contained fixture (sibling
    pure-python modules are bundled; data-file/third-party workloads still get a
    fixture that goes red until the keep supports them, but the workload exists).
    """
    bmdir = perf_wall_gen.benchmarks_dir()
    if bmdir is None:
        return 0, 0
    cands = list(perf_wall_gen.candidates(bmdir))
    total = len(cands)
    covered = sum(1 for name, _ in cands if (perf_wall_gen.OUT_DIR / f"{name}.py").exists())
    return total, covered


SECRET_CLASSES = ("path", "addr", "env", "source")  # filesystem / memory / env var / source snippet


def safety_denominator() -> int:
    """The Safety denominator: error-leak matrix = builtin exceptions × secret classes.

    Every builtin exception mamba can raise (non-Warning) × every secret class a
    message must never leak. One cell = one (exception, secret-class) probe.
    """
    import builtins
    excs = [n for n in dir(builtins)
            if isinstance(getattr(builtins, n), type)
            and issubclass(getattr(builtins, n), BaseException)
            and not issubclass(getattr(builtins, n), Warning)]
    return len(excs) * len(SECRET_CLASSES)


def safety_cases() -> int:
    """Safety wall: error-leak matrix cells under cpython/security-matrix/.

    safety_wall_gen.py emits one cell per (exception, secret-class), so this
    counts the matrix built against the 228-cell denominator.
    """
    d = CPYTHON_DIR / "security-matrix"
    return sum(1 for _ in d.rglob("*.py")) if d.exists() else 0


def main() -> int:
    sigs, cases = type_wall_signatures_and_cases()
    cov1 = 100 * cases / sigs if sigs else 0.0

    lib_total, beh_cases, beh_excl = behavior_signatures_and_cases()
    cov2 = 100 * beh_cases / lib_total if lib_total else 0.0
    beh_line = (f"{cov2:5.1f}%  ({beh_cases} covered / {lib_total} passable)"
                f"  [excluded {beh_excl} CPython-fail]"
                if lib_total else " N/A    (python3.12 test package unavailable)")

    perf_total, perf_n = perf_signatures_and_cases()
    cov3 = 100 * perf_n / perf_total if perf_total else 0.0
    perf_line = (f"{cov3:5.1f}%  ({perf_n} fixtures / {perf_total} pyperformance workloads)"
                 if perf_total else " N/A    (pyperformance not installed — pip install pyperformance)")

    safe_total = safety_denominator()
    safe_n = safety_cases()
    cov4 = 100 * safe_n / safe_total if safe_total else 0.0
    safe_line = f"{cov4:5.1f}%  ({safe_n} security cases / {safe_total} error-leak matrix cells)"

    print(f"① Type      coverage = {cov1:5.1f}%  ({cases} covered / {sigs} public wrongable signatures)")
    print(f"② Behavior  coverage = {beh_line}")
    print(f"③ Perf      coverage = {perf_line}")
    print(f"④ Safety    coverage = {safe_line}")
    cov2_s = f"{cov2:.1f}%" if lib_total else "N/A"
    cov3_s = f"{cov3:.1f}%" if perf_total else "N/A"
    print(f"WALL: ① {cov1:.1f}%  ② {cov2_s}  ③ {cov3_s}  ④ {cov4:.1f}%   (done = all four 100%)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
