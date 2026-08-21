# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "sqlite_synth"
# dimension = "perf"
# case = "sqlite_synth"
# subject = "pyperformance sqlite_synth"
# kind = "bench"
# xfail = "mamba must run the pyperformance sqlite_synth workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_sqlite_synth/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance sqlite_synth workload faster than CPython on CPU+RSS
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
SQLite benchmark.

The goal of the benchmark is to test CFFI performance and going back and forth
between SQLite and Python a lot. Therefore the queries themselves are really
simple.
"""

import sqlite3
import math

import pyperf


class AvgLength(object):

    def __init__(self):
        self.sum = 0
        self.count = 0

    def step(self, x):
        if x is not None:
            self.count += 1
            self.sum += len(x)

    def finalize(self):
        return self.sum / float(self.count)


def bench_sqlite(loops):
    t0 = pyperf.perf_counter()

    conn = sqlite3.connect(":memory:")
    conn.execute('create table cos (x, y, z);')
    for i in range(loops):
        cos_i = math.cos(i)
        conn.execute('insert into cos values (?, ?, ?)',
                     [i, cos_i, str(i)])

    conn.create_function("cos", 1, math.cos)
    for x, cosx1, cosx2 in conn.execute("select x, cos(x), y from cos"):
        assert math.cos(x) == cosx1 == cosx2

    conn.create_aggregate("avglength", 1, AvgLength)
    cursor = conn.execute("select avglength(z) from cos;")
    cursor.fetchone()[0]

    conn.execute("delete from cos;")
    conn.close()

    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = "Benchmark Python aggregate for SQLite"
    runner.bench_time_func('sqlite_synth', bench_sqlite)
