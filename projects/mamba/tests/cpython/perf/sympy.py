# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "sympy"
# dimension = "perf"
# case = "sympy"
# subject = "pyperformance sympy"
# kind = "bench"
# xfail = "mamba must run the pyperformance sympy workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_sympy/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance sympy workload faster than CPython on CPU+RSS
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

import pyperf

from sympy import expand, symbols, integrate, tan, summation
from sympy.core.cache import clear_cache


def bench_expand():
    x, y, z = symbols('x y z')
    expand((1 + x + y + z) ** 20)


def bench_integrate():
    x, y = symbols('x y')
    f = (1 / tan(x)) ** 10
    return integrate(f, x)


def bench_sum():
    x, i = symbols('x i')
    summation(x ** i / i, (i, 1, 400))


def bench_str():
    x, y, z = symbols('x y z')
    str(expand((x + 2 * y + 3 * z) ** 30))


def bench_sympy(loops, func):
    timer = pyperf.perf_counter
    dt = 0

    for _ in range(loops):
        # Don't benchmark clear_cache(), exclude it of the benchmark
        clear_cache()

        t0 = timer()
        func()
        dt += (timer() - t0)

    return dt


BENCHMARKS = ("expand", "integrate", "sum", "str")


def add_cmdline_args(cmd, args):
    if args.benchmark:
        cmd.append(args.benchmark)


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata['description'] = "SymPy benchmark"
    runner.argparser.add_argument("benchmark", nargs='?',
                                  choices=BENCHMARKS)

    import gc
    gc.disable()

    args = runner.parse_args()
    if args.benchmark:
        benchmarks = (args.benchmark,)
    else:
        benchmarks = BENCHMARKS

    for bench in benchmarks:
        name = 'sympy_%s' % bench
        func = globals()['bench_' + bench]
        runner.bench_time_func(name, bench_sympy, func)
