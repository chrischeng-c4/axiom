# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "pprint"
# dimension = "perf"
# case = "pprint"
# subject = "pyperformance pprint"
# kind = "bench"
# xfail = "mamba must run the pyperformance pprint workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_pprint/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance pprint workload faster than CPython on CPU+RSS
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

"""Test the performance of pprint.PrettyPrinter.

This benchmark was available as `python -m pprint` until Python 3.12.

Authors: Fred Drake (original), Oleg Iarygin (pyperformance port).
"""

import pyperf
from pprint import PrettyPrinter


printable = [('string', (1, 2), [3, 4], {5: 6, 7: 8})] * 100_000
p = PrettyPrinter()


if __name__ == '__main__':
    runner = pyperf.Runner()
    runner.metadata['description'] = 'pprint benchmark'

    if hasattr(p, '_safe_repr'):
        runner.bench_func('pprint_safe_repr', p._safe_repr,
                          printable, {}, None, 0)
    runner.bench_func('pprint_pformat', p.pformat, printable)
