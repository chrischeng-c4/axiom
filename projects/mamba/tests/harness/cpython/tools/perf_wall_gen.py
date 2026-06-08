#!/usr/bin/env python3.12
"""Generate ③ Perf wall: one self-contained fixture per pyperformance benchmark.

The Perf wall = every pyperformance workload (the community's real-workload bar)
has a mamba bench fixture. Each fixture bundles a minimal pyperf shim (runs each
workload once instead of calibrating) with the benchmark's run_benchmark.py, so
it is self-contained — no pyperf dependency at run time. The CPU + peak-RSS
comparison vs CPython is Phase 2 (external getrusage); the wall only needs the
workload to exist as a fixture. mamba red (parse/runtime gap) is a correct marker.

Single-file benchmarks (just run_benchmark.py) are generated; multi-file ones
(extra workload modules / data / pip deps) are reported as a remaining gap.

    python3.12 perf_wall_gen.py --dry-run
    python3.12 perf_wall_gen.py --write
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from wall_gen_core import PEP723Header  # noqa: E402

MAMBA_DIR = Path(__file__).resolve().parents[4]
OUT_DIR = MAMBA_DIR / "tests" / "cpython" / "perf"

# Minimal pyperf replacement: registers a fake `pyperf` module so a benchmark's
# `import pyperf` resolves to this, and runs each bench_*func workload once.
SHIM = '''import sys as _sys, types as _t
class _Args:
    """Minimal argparser stand-in (no `import argparse`, which a sibling
    perf/argparse.py fixture would shadow). Records add_argument defaults."""
    def __init__(self):
        self._defaults = {}
    def add_argument(self, *names, **k):
        dest = k.get("dest")
        if not dest:
            for n in names:
                if isinstance(n, str) and n.startswith("--"):
                    dest = n[2:].replace("-", "_"); break
                if isinstance(n, str) and not n.startswith("-"):
                    dest = n; break
        if dest:
            self._defaults[dest] = k.get("default")
    def add_mutually_exclusive_group(self, *a, **k):
        return self
    def add_argument_group(self, *a, **k):
        return self
class _Runner:
    def __init__(self, *a, **k):
        self.metadata = {}
        self.argparser = _Args()
    def parse_args(self, *a, **k):
        return _t.SimpleNamespace(**self.argparser._defaults)
    def bench_func(self, name, func, *args, **k):
        func(*args)                       # func runs the workload itself
    def bench_time_func(self, name, func, *args, **k):
        func(1, *args)                    # pyperf passes loops as the 1st arg
    def bench_async_func(self, name, func, *args, **k):
        import asyncio
        asyncio.run(func(*args))
def _reg(_name, _code):
    _m = _t.ModuleType(_name)
    exec(compile(_code, _name, "exec"), _m.__dict__)
    _sys.modules[_name] = _m
_p = _t.ModuleType("pyperf")
_p.Runner = _Runner
def _pc():
    import time
    return time.perf_counter()
_p.perf_counter = _pc
_sys.modules["pyperf"] = _p
'''


def benchmarks_dir() -> Path | None:
    code = ("import pyperformance,os;"
            "print(os.path.join(os.path.dirname(pyperformance.__file__),"
            "'data-files','benchmarks'))")
    try:
        out = subprocess.run(["python3.12", "-c", code],
                             capture_output=True, text=True, timeout=30).stdout.strip()
    except (OSError, subprocess.SubprocessError):
        return None
    d = Path(out)
    return d if d.is_dir() else None


def candidates(bmdir: Path):
    """Yield (name, benchmark_dir) for every pyperformance workload program."""
    for bm in sorted(bmdir.glob("bm_*")):
        if (bm / "run_benchmark.py").is_file():
            yield bm.name[3:], bm


def render(name: str, bm: Path) -> tuple[str, str]:
    src = (bm / "run_benchmark.py").read_text(encoding="utf-8", errors="replace")
    # Bundle sibling pure-python modules so `import bm_x` resolves inside the
    # fixture (e.g. regex_compile pulls in bm_regex_effbot / bm_regex_v8). Data
    # files and third-party deps are NOT bundled — those fixtures go red until the
    # keep supports them, but the workload still exists on the wall.
    siblings = ""
    for sib in sorted(bm.glob("*.py")):
        if sib.name == "run_benchmark.py":
            continue
        sib_src = sib.read_text(encoding="utf-8", errors="replace")
        siblings += f"_reg({sib.stem!r}, {sib_src!r})\n"
    header = PEP723Header(
        bucket="perf", lib=name, dimension="perf", case=name,
        subject=f"pyperformance {name}", kind="bench",
        xfail=f"mamba must run the pyperformance {name} workload faster than CPython on CPU+RSS",
        mem_carveout="",
        source=f"pyperformance/data-files/benchmarks/bm_{name}/run_benchmark.py",
        status="filled",
    ).render()
    return f"{name}.py", header + SHIM + siblings + "\n" + src


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--write", action="store_true")
    args = ap.parse_args()

    bmdir = benchmarks_dir()
    if bmdir is None:
        print("pyperformance not installed (pip install --user pyperformance)")
        return 1

    rows = list(candidates(bmdir))

    if args.dry_run or not args.write:
        print(f"pyperformance workload programs: {len(rows)}")
        return 0

    written = 0
    for name, bm in rows:
        rel, text = render(name, bm)
        path = OUT_DIR / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")
        written += 1
    print(f"wrote {written} perf-wall cases under {OUT_DIR}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
