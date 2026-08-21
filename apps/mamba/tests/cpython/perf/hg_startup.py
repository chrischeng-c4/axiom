# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "perf"
# lib = "hg_startup"
# dimension = "perf"
# case = "hg_startup"
# subject = "pyperformance hg_startup"
# kind = "bench"
# xfail = "mamba must run the pyperformance hg_startup workload faster than CPython on CPU+RSS"
# mem_carveout = ""
# source = "pyperformance/data-files/benchmarks/bm_hg_startup/run_benchmark.py"
# status = "filled"
# ///
# mamba-xfail: mamba must run the pyperformance hg_startup workload faster than CPython on CPU+RSS
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

import sys
import subprocess

import pyperf
from pyperformance.venv import get_venv_program


def get_hg_version(hg_bin):
    # Fast-path: use directly the Python module
    try:
        from mercurial.__version__ import version
        if isinstance(version, bytes):
            return version.decode('utf8')
        else:
            return version
    except ImportError:
        pass

    # Slow-path: run the "hg --version" command
    proc = subprocess.Popen([sys.executable, hg_bin, "--version"],
                            stdout=subprocess.PIPE,
                            universal_newlines=True)
    stdout = proc.communicate()[0]
    if proc.returncode:
        print("ERROR: Mercurial command failed!")
        sys.exit(proc.returncode)
    return stdout.splitlines()[0]


if __name__ == "__main__":
    runner = pyperf.Runner(values=25)

    runner.metadata['description'] = "Performance of the Python startup"
    args = runner.parse_args()

    hg_bin = get_venv_program('hg')
    runner.metadata['hg_version'] = get_hg_version(hg_bin)

    command = [sys.executable, hg_bin, "help"]
    runner.bench_command('hg_startup', command)
