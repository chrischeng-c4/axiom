# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "decimal_pi"
# dimension = "perf"
# case = "decimal_pi"
# subject = "pyperformance decimal_pi"
# kind = "bench"
# xfail = "mamba must run the pyperformance decimal_pi workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_decimal_pi/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance decimal_pi workload faster than CPython on CPU+RSS
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
Calculate `pi` using the decimal module.

The `pidigits` benchmark does a similar thing using regular (long) ints.

- 2024-06-14: Michael Droettboom copied this from
  Modules/_decimal/tests/bench.py in the CPython source and adapted to use
  pyperf.
"""

# Original copyright notice in CPython source:

#
# Copyright (C) 2001-2012 Python Software Foundation. All Rights Reserved.
# Modified and extended by Stefan Krah.
#


import decimal


import pyperf


def pi_decimal():
    """decimal"""
    D = decimal.Decimal
    lasts, t, s, n, na, d, da = D(0), D(3), D(3), D(1), D(0), D(0), D(24)
    while s != lasts:
        lasts = s
        n, na = n + na, na + 8
        d, da = d + da, da + 32
        t = (t * n) / d
        s += t
    return s


def bench_decimal_pi():
    for prec in [9, 19]:
        decimal.getcontext().prec = prec
        for _ in range(10000):
            _ = pi_decimal()


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata["description"] = "decimal_pi benchmark"

    args = runner.parse_args()
    runner.bench_func("decimal_pi", bench_decimal_pi)
