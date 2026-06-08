# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "pathlib"
# dimension = "perf"
# case = "pathlib"
# subject = "pyperformance pathlib"
# kind = "bench"
# xfail = "mamba must run the pyperformance pathlib workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_pathlib/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance pathlib workload faster than CPython on CPU+RSS
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
Test the performance of pathlib operations.

This benchmark stresses the creation of small objects, globbing, and system
calls.
"""

# Python imports
import os
import pathlib
import shutil
import tempfile

import pyperf


NUM_FILES = 2000


def generate_filenames(tmp_path, num_files):
    i = 0
    while num_files:
        for ext in [".py", ".txt", ".tar.gz", ""]:
            i += 1
            yield os.path.join(tmp_path, str(i) + ext)
            num_files -= 1


def setup(num_files):
    tmp_path = tempfile.mkdtemp()
    for fn in generate_filenames(tmp_path, num_files):
        with open(fn, "wb") as f:
            f.write(b'benchmark')

    return tmp_path


def bench_pathlib(loops, tmp_path):
    base_path = pathlib.Path(tmp_path)

    # Warm up the filesystem cache and keep some objects in memory.
    path_objects = list(base_path.iterdir())
    # FIXME: does this code really cache anything?
    for p in path_objects:
        p.stat()
    assert len(path_objects) == NUM_FILES, len(path_objects)

    range_it = range(loops)
    t0 = pyperf.perf_counter()

    for _ in range_it:
        # Do something simple with each path.
        for p in base_path.iterdir():
            p.stat()
        for p in base_path.glob("*.py"):
            p.stat()
        for p in base_path.iterdir():
            p.stat()
        for p in base_path.glob("*.py"):
            p.stat()

    return pyperf.perf_counter() - t0


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = ("Test the performance of "
                                      "pathlib operations.")

    modname = pathlib.__name__
    runner.metadata['pathlib_module'] = modname

    tmp_path = setup(NUM_FILES)
    try:
        runner.bench_time_func('pathlib', bench_pathlib, tmp_path)
    finally:
        shutil.rmtree(tmp_path)
