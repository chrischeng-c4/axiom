# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "networkx"
# dimension = "perf"
# case = "networkx"
# subject = "pyperformance networkx"
# kind = "bench"
# xfail = "mamba must run the pyperformance networkx workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_networkx/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance networkx workload faster than CPython on CPU+RSS
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
Some graph algorithm benchmarks using networkx

This uses the public domain Amazon data set from the SNAP benchmarks:

    https://snap.stanford.edu/data/amazon0302.html

Choice of benchmarks inspired by Timothy Lin's work here:

    https://www.timlrx.com/blog/benchmark-of-popular-graph-network-packages
"""

import collections
from pathlib import Path

import networkx

import pyperf


DATA_FILE = Path(__file__).parent / "data" / "amazon0302.txt.gz"


graph = networkx.read_adjlist(DATA_FILE)


def bench_shortest_path():
    collections.deque(networkx.shortest_path_length(graph, "0"))


def bench_connected_components():
    networkx.number_connected_components(graph)


def bench_k_core():
    networkx.k_core(graph)


BENCHMARKS = {
    "shortest_path": bench_shortest_path,
    "connected_components": bench_connected_components,
    "k_core": bench_k_core,
}


def add_cmdline_args(cmd, args):
    cmd.append(args.benchmark)


def add_parser_args(parser):
    parser.add_argument("benchmark", choices=BENCHMARKS, help="Which benchmark to run.")


if __name__ == "__main__":
    runner = pyperf.Runner(add_cmdline_args=add_cmdline_args)
    runner.metadata["description"] = "NetworkX benchmark"
    add_parser_args(runner.argparser)
    args = runner.parse_args()
    benchmark = args.benchmark

    runner.bench_func(args.benchmark, BENCHMARKS[args.benchmark])
