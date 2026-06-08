# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "subinterpreter_test__test_callbacks_leak_refcycle"
# subject = "cpython.test_atexit.SubinterpreterTest.test_callbacks_leak_refcycle"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_atexit.py::SubinterpreterTest::test_callbacks_leak_refcycle
"""Auto-ported test: SubinterpreterTest::test_callbacks_leak_refcycle (CPython 3.12 oracle)."""


import atexit
import os
import textwrap
import unittest
from test import support
from test.support import script_helper


# --- test body ---
n = atexit._ncallbacks()
code = textwrap.dedent('\n            import atexit\n            def f():\n                pass\n            atexit.register(f)\n            atexit.__atexit = atexit\n        ')
ret = support.run_in_subinterp(code)

assert ret == 0

assert atexit._ncallbacks() == n
print("SubinterpreterTest::test_callbacks_leak_refcycle: ok")
