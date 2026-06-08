# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_int_max_str_digits"
# subject = "cpython.test_cmd_line.CmdLineTest.test_int_max_str_digits"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_int_max_str_digits
"""Auto-ported test: CmdLineTest::test_int_max_str_digits (CPython 3.12 oracle)."""


import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from test import support
from test.support import os_helper
from test.support.script_helper import spawn_python, kill_python, assert_python_ok, assert_python_failure, interpreter_requires_environment


if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

def _kill_python_and_exit_code(p):
    data = kill_python(p)
    returncode = p.wait()
    return (data, returncode)

def tearDownModule():
    support.reap_children()


# --- test body ---
code = 'import sys; print(sys.flags.int_max_str_digits, sys.get_int_max_str_digits())'
assert_python_failure('-X', 'int_max_str_digits', '-c', code)
assert_python_failure('-X', 'int_max_str_digits=foo', '-c', code)
assert_python_failure('-X', 'int_max_str_digits=100', '-c', code)
assert_python_failure('-X', 'int_max_str_digits', '-c', code, PYTHONINTMAXSTRDIGITS='4000')
assert_python_failure('-c', code, PYTHONINTMAXSTRDIGITS='foo')
assert_python_failure('-c', code, PYTHONINTMAXSTRDIGITS='100')

def res2int(res):
    out = res.out.strip().decode('utf-8')
    return tuple((int(i) for i in out.split()))
res = assert_python_ok('-c', code)
current_max = sys.get_int_max_str_digits()

assert res2int(res) == (current_max, current_max)
res = assert_python_ok('-X', 'int_max_str_digits=0', '-c', code)

assert res2int(res) == (0, 0)
res = assert_python_ok('-X', 'int_max_str_digits=4000', '-c', code)

assert res2int(res) == (4000, 4000)
res = assert_python_ok('-X', 'int_max_str_digits=100000', '-c', code)

assert res2int(res) == (100000, 100000)
res = assert_python_ok('-c', code, PYTHONINTMAXSTRDIGITS='0')

assert res2int(res) == (0, 0)
res = assert_python_ok('-c', code, PYTHONINTMAXSTRDIGITS='4000')

assert res2int(res) == (4000, 4000)
res = assert_python_ok('-X', 'int_max_str_digits=6000', '-c', code, PYTHONINTMAXSTRDIGITS='4000')

assert res2int(res) == (6000, 6000)
print("CmdLineTest::test_int_max_str_digits: ok")
