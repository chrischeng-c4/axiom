# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_large_pythonpath"
# subject = "cpython.test_cmd_line.CmdLineTest.test_large_PYTHONPATH"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_large_PYTHONPATH
"""Auto-ported test: CmdLineTest::test_large_PYTHONPATH (CPython 3.12 oracle)."""


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
path1 = 'ABCDE' * 100
path2 = 'FGHIJ' * 100
path = path1 + os.pathsep + path2
code = 'if 1:\n            import sys\n            path = ":".join(sys.path)\n            path = path.encode("ascii", "backslashreplace")\n            sys.stdout.buffer.write(path)'
rc, out, err = assert_python_ok('-S', '-c', code, PYTHONPATH=path)

assert path1.encode('ascii') in out

assert path2.encode('ascii') in out
print("CmdLineTest::test_large_PYTHONPATH: ok")
