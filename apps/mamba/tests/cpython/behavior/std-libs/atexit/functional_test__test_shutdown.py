# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "functional_test__test_shutdown"
# subject = "cpython.test_atexit.FunctionalTest.test_shutdown"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_atexit.py::FunctionalTest::test_shutdown
"""Auto-ported test: FunctionalTest::test_shutdown (CPython 3.12 oracle)."""


import atexit
import os
import textwrap
import unittest
from test import support
from test.support import script_helper


# --- test body ---
code = textwrap.dedent('\n            import atexit\n\n            def f(msg):\n                print(msg)\n\n            atexit.register(f, "one")\n            atexit.register(f, "two")\n        ')
res = script_helper.assert_python_ok('-c', code)

assert res.out.decode().splitlines() == ['two', 'one']

assert not res.err
print("FunctionalTest::test_shutdown: ok")
