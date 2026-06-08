# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "gc_traversal"
# dimension = "perf"
# case = "gc_traversal"
# subject = "pyperformance gc_traversal"
# kind = "bench"
# xfail = "mamba must run the pyperformance gc_traversal workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_gc_traversal/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance gc_traversal workload faster than CPython on CPU+RSS
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
import gc

N_LEVELS = 1000


def create_recursive_containers(n_levels):

    current_list = []
    for n in range(n_levels):
        new_list = [None] * n
        for index in range(n):
            new_list[index] = current_list
        current_list = new_list

    return current_list


def benchamark_collection(loops, n_levels):
    total_time = 0
    all_cycles = create_recursive_containers(n_levels)
    for _ in range(loops):
        gc.collect()
        # Main loop to measure
        t0 = pyperf.perf_counter()
        collected = gc.collect()
        total_time += pyperf.perf_counter() - t0

        assert collected is None or collected == 0

    return total_time


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata["description"] = "GC traversal benchmark"
    runner.bench_time_func("gc_traversal", benchamark_collection, N_LEVELS)
