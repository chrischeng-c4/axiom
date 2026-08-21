# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "popen"
# dimension = "behavior"
# case = "popen_test__test_keywords"
# subject = "cpython.test_popen.PopenTest.test_keywords"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_popen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_popen.py::PopenTest::test_keywords
"""Auto-ported test: PopenTest::test_keywords (CPython 3.12 oracle)."""


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
with os.popen(cmd='echo hello', mode='r', buffering=-1) as f:

    assert f.read() == 'hello\n'

    assert not f.closed

assert f.closed
print("PopenTest::test_keywords: ok")
