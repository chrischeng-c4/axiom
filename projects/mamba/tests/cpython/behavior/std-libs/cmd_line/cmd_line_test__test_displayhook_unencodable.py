# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_displayhook_unencodable"
# subject = "cpython.test_cmd_line.CmdLineTest.test_displayhook_unencodable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_displayhook_unencodable
"""Auto-ported test: CmdLineTest::test_displayhook_unencodable (CPython 3.12 oracle)."""


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
for encoding in ('ascii', 'latin-1', 'utf-8'):
    env = os.environ.copy()
    env['PYTHONIOENCODING'] = encoding
    p = subprocess.Popen([sys.executable, '-i'], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, env=env)
    text = 'a=é b=\udc80 c=𐀀 d=\U0010ffff'
    p.stdin.write(ascii(text).encode('ascii') + b'\n')
    p.stdin.write(b'exit()\n')
    data = kill_python(p)
    escaped = repr(text).encode(encoding, 'backslashreplace')

    assert escaped in data
print("CmdLineTest::test_displayhook_unencodable: ok")
