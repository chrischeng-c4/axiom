# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "functional_test__test_atexit_instances"
# subject = "cpython.test_atexit.FunctionalTest.test_atexit_instances"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_atexit.py::FunctionalTest::test_atexit_instances
"""Auto-ported test: FunctionalTest::test_atexit_instances (CPython 3.12 oracle)."""


import atexit
import os
import textwrap
import unittest
from test import support
from test.support import script_helper


# --- test body ---
code = textwrap.dedent('\n            import sys\n            import atexit as atexit1\n            del sys.modules[\'atexit\']\n            import atexit as atexit2\n            del sys.modules[\'atexit\']\n\n            assert atexit2 is not atexit1\n\n            atexit1.register(print, "atexit1")\n            atexit2.register(print, "atexit2")\n        ')
res = script_helper.assert_python_ok('-c', code)

assert res.out.decode().splitlines() == ['atexit2', 'atexit1']

assert not res.err
print("FunctionalTest::test_atexit_instances: ok")
