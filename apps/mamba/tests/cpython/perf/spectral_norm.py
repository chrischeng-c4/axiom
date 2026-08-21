# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "spectral_norm"
# dimension = "perf"
# case = "spectral_norm"
# subject = "pyperformance spectral_norm"
# kind = "bench"
# xfail = "mamba must run the pyperformance spectral_norm workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_spectral_norm/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance spectral_norm workload faster than CPython on CPU+RSS
import sys as _sys, types as _t
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

"""
MathWorld: "Hundred-Dollar, Hundred-Digit Challenge Problems", Challenge #3.
http://mathworld.wolfram.com/Hundred-DollarHundred-DigitChallengeProblems.html

The Computer Language Benchmarks Game
http://benchmarksgame.alioth.debian.org/u64q/spectralnorm-description.html#spectralnorm

Contributed by Sebastien Loisel
Fixed by Isaac Gouy
Sped up by Josh Goldfoot
Dirtily sped up by Simon Descarpentries
Concurrency by Jason Stitt
"""

import pyperf


DEFAULT_N = 130


def eval_A(i, j):
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)


def eval_times_u(func, u):
    return [func((i, u)) for i in range(len(list(u)))]


def eval_AtA_times_u(u):
    return eval_times_u(part_At_times_u, eval_times_u(part_A_times_u, u))


def part_A_times_u(i_u):
    i, u = i_u
    partial_sum = 0
    for j, u_j in enumerate(u):
        partial_sum += eval_A(i, j) * u_j
    return partial_sum


def part_At_times_u(i_u):
    i, u = i_u
    partial_sum = 0
    for j, u_j in enumerate(u):
        partial_sum += eval_A(j, i) * u_j
    return partial_sum


def bench_spectral_norm(loops):
    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for _ in range_it:
        u = [1] * DEFAULT_N

        for dummy in range(10):
            v = eval_AtA_times_u(u)
            u = eval_AtA_times_u(v)

        vBv = vv = 0

        for ue, ve in zip(u, v):
            vBv += ue * ve
            vv += ve * ve

    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = (
        'MathWorld: "Hundred-Dollar, Hundred-Digit Challenge Problems", '
        'Challenge #3.')
    runner.bench_time_func('spectral_norm', bench_spectral_norm)
