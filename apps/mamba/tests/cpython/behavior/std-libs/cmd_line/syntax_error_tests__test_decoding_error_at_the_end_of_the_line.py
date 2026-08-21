# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "syntax_error_tests__test_decoding_error_at_the_end_of_the_line"
# subject = "cpython.test_cmd_line.SyntaxErrorTests.test_decoding_error_at_the_end_of_the_line"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::SyntaxErrorTests::test_decoding_error_at_the_end_of_the_line
"""Auto-ported test: SyntaxErrorTests::test_decoding_error_at_the_end_of_the_line (CPython 3.12 oracle)."""


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
def check_string(code):
    proc = subprocess.run([sys.executable, '-'], input=code, stdout=subprocess.PIPE, stderr=subprocess.PIPE)

    assert proc.returncode != 0

    assert proc.stderr != None

    assert b'\nSyntaxError' in proc.stderr
check_string(b"'\\u1f'")
print("SyntaxErrorTests::test_decoding_error_at_the_end_of_the_line: ok")
