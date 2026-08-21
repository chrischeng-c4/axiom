# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "popen"
# dimension = "behavior"
# case = "popen_test__test_popen"
# subject = "cpython.test_popen.PopenTest.test_popen"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_popen.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_popen.py::PopenTest::test_popen
"""Auto-ported test: PopenTest::test_popen (CPython 3.12 oracle)."""


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
def _do_test_commandline(cmdline, expected):
    cmd = '%s -c "import sys; print(sys.argv)" %s'
    cmd = cmd % (python, cmdline)
    with os.popen(cmd) as p:
        data = p.read()
    got = eval(data)[1:]

    assert got == expected

try:
    os.popen()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
_do_test_commandline('foo bar', ['foo', 'bar'])
_do_test_commandline('foo "spam and eggs" "silly walk"', ['foo', 'spam and eggs', 'silly walk'])
_do_test_commandline('foo "a \\"quoted\\" arg" bar', ['foo', 'a "quoted" arg', 'bar'])
support.reap_children()
print("PopenTest::test_popen: ok")
