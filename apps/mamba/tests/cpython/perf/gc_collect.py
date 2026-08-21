# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "gc_collect"
# dimension = "perf"
# case = "gc_collect"
# subject = "pyperformance gc_collect"
# kind = "bench"
# xfail = "mamba must run the pyperformance gc_collect workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_gc_collect/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance gc_collect workload faster than CPython on CPU+RSS
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

CYCLES = 100
LINKS = 20


class Node:
    def __init__(self):
        self.next = None
        self.prev = None

    def link_next(self, next):
        self.next = next
        self.next.prev = self


def create_cycle(node, n_links):
    """Create a cycle of n_links nodes, starting with node."""

    if n_links == 0:
        return

    current = node
    for i in range(n_links):
        next_node = Node()
        current.link_next(next_node)
        current = next_node

    current.link_next(node)


def create_gc_cycles(n_cycles, n_links):
    """Create n_cycles cycles n_links+1 nodes each."""

    cycles = []
    for _ in range(n_cycles):
        node = Node()
        cycles.append(node)
        create_cycle(node, n_links)
    return cycles


def benchamark_collection(loops, cycles, links):
    total_time = 0
    for _ in range(loops):
        gc.collect()
        all_cycles = create_gc_cycles(cycles, links)

        # Main loop to measure
        del all_cycles
        t0 = pyperf.perf_counter()
        collected = gc.collect()
        total_time += pyperf.perf_counter() - t0

        assert collected is None or collected >= cycles * (links + 1)

    return total_time


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata["description"] = "GC link benchmark"
    runner.bench_time_func("create_gc_cycles", benchamark_collection, CYCLES, LINKS)
