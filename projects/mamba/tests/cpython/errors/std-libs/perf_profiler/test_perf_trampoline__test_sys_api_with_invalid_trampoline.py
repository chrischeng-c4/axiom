# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "perf_profiler"
# dimension = "errors"
# case = "test_perf_trampoline__test_sys_api_with_invalid_trampoline"
# subject = "cpython.test_perf_profiler.TestPerfTrampoline.test_sys_api_with_invalid_trampoline"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_perf_profiler.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_perf_profiler.py::TestPerfTrampoline::test_sys_api_with_invalid_trampoline
"""Auto-ported test: TestPerfTrampoline::test_sys_api_with_invalid_trampoline."""


import subprocess
import sys
import sysconfig


if int(sysconfig.get_config_var("PY_HAVE_PERF_TRAMPOLINE") or 0) != 1:
    print("TestPerfTrampoline::test_sys_api_with_invalid_trampoline: skipped, perf trampoline unsupported")
    raise SystemExit(0)

code = "import sys; sys.activate_stack_trampoline('invalid')"
process = subprocess.run(
    [sys.executable, "-c", code],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    check=False,
)

assert process.returncode != 0, process.returncode
assert b"invalid backend: invalid" in process.stderr, process.stderr

print("TestPerfTrampoline::test_sys_api_with_invalid_trampoline: ok")
