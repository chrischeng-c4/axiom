# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "html5lib"
# dimension = "perf"
# case = "html5lib"
# subject = "pyperformance html5lib"
# kind = "bench"
# xfail = "mamba must run the pyperformance html5lib workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_html5lib/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance html5lib workload faster than CPython on CPU+RSS
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

"""Wrapper script for testing the performance of the html5lib HTML 5 parser.

The input data is the spec document for HTML 5, written in HTML 5.
The spec was pulled from http://svn.whatwg.org/webapps/index.
"""
import io
import os.path

import html5lib
import pyperf


__author__ = "collinwinter@google.com (Collin Winter)"


def bench_html5lib(html_file):
    html_file.seek(0)
    html5lib.parse(html_file)


if __name__ == "__main__":
    runner = pyperf.Runner()
    runner.metadata['description'] = (
        "Test the performance of the html5lib parser.")
    runner.metadata['html5lib_version'] = html5lib.__version__

    # Get all our IO over with early.
    filename = os.path.join(os.path.dirname(__file__),
                            "data", "w3_tr_html5.html")
    with open(filename, "rb") as fp:
        html_file = io.BytesIO(fp.read())

    runner.bench_func('html5lib', bench_html5lib, html_file)
