# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "docutils"
# dimension = "perf"
# case = "docutils"
# subject = "pyperformance docutils"
# kind = "bench"
# xfail = "mamba must run the pyperformance docutils workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_docutils/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance docutils workload faster than CPython on CPU+RSS
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
Convert Docutils' documentation from reStructuredText to <format>.
"""

import contextlib
from pathlib import Path

import docutils
from docutils import core
import pyperf

try:
    from docutils.utils.math.math2html import Trace
except ImportError:
    pass
else:
    Trace.show = lambda message, channel: ...  # don't print to console

DOC_ROOT = (Path(__file__).parent / "data" / "docs").resolve()


def build_html(doc_root):
    elapsed = 0
    for file in doc_root.rglob("*.txt"):
        file_contents = file.read_text(encoding="utf-8")
        t0 = pyperf.perf_counter()
        with contextlib.suppress(docutils.ApplicationError):
            core.publish_string(source=file_contents,
                                reader_name="standalone",
                                parser_name="restructuredtext",
                                writer_name="html5",
                                settings_overrides={
                                    "input_encoding": "unicode",
                                    "output_encoding": "unicode",
                                    "report_level": 5,
                                })
        elapsed += pyperf.perf_counter() - t0
    return elapsed


def bench_docutils(loops, doc_root):
    runs_total = 0
    for _ in range(loops):
        runs_total += build_html(doc_root)
    return runs_total


if __name__ == "__main__":
    runner = pyperf.Runner()

    runner.metadata['description'] = "Render documentation with Docutils"
    args = runner.parse_args()

    runner.bench_time_func("docutils", bench_docutils, DOC_ROOT)
