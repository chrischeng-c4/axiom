# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_unknown_options"
# subject = "cpython.test_cmd_line.CmdLineTest.test_unknown_options"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_unknown_options
"""Auto-ported test: CmdLineTest::test_unknown_options (CPython 3.12 oracle)."""


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
rc, out, err = assert_python_failure('-E', '-z')

assert b'Unknown option: -z' in err

assert err.splitlines().count(b'Unknown option: -z') == 1

assert b'' == out
rc, out, err = assert_python_failure('-z', without='-E')

assert b'Unknown option: -z' in err

assert err.splitlines().count(b'Unknown option: -z') == 1

assert b'' == out
rc, out, err = assert_python_failure('-a', '-z', without='-E')

assert b'Unknown option: -a' in err

assert b'Unknown option: -z' not in err

assert err.splitlines().count(b'Unknown option: -a') == 1

assert b'' == out
print("CmdLineTest::test_unknown_options: ok")
