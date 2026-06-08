# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "popen"
# dimension = "behavior"
# case = "popen_test__test_return_code"
# subject = "cpython.test_popen.PopenTest.test_return_code"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_popen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_popen.py::PopenTest::test_return_code
"""Auto-ported test: PopenTest::test_return_code (CPython 3.12 oracle)."""


import unittest
from test import support
import os, sys


'Basic tests for os.popen()\n\n  Particularly useful for platforms that fake popen.\n'

if not hasattr(os, 'popen'):
    raise unittest.SkipTest('need os.popen()')

python = sys.executable

if ' ' in python:
    python = '"' + python + '"'


# --- test body ---

assert os.popen('exit 0').close() == None
status = os.popen('exit 42').close()
if os.name == 'nt':

    assert status == 42
else:

    assert os.waitstatus_to_exitcode(status) == 42
print("PopenTest::test_return_code: ok")
