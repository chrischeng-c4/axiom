# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cmd_line"
# dimension = "behavior"
# case = "cmd_line_test__test_pythondevmode_env"
# subject = "cpython.test_cmd_line.CmdLineTest.test_pythondevmode_env"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_cmd_line.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_cmd_line.py::CmdLineTest::test_pythondevmode_env
"""Auto-ported test: CmdLineTest::test_pythondevmode_env (CPython 3.12 oracle)."""


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
code = 'import sys; print(sys.flags.dev_mode)'
env = dict(os.environ)
env.pop('PYTHONDEVMODE', None)
args = (sys.executable, '-c', code)
proc = subprocess.run(args, stdout=subprocess.PIPE, universal_newlines=True, env=env)

assert proc.stdout.rstrip() == 'False'

assert proc.returncode == 0
env['PYTHONDEVMODE'] = '1'
proc = subprocess.run(args, stdout=subprocess.PIPE, universal_newlines=True, env=env)

assert proc.stdout.rstrip() == 'True'

assert proc.returncode == 0
print("CmdLineTest::test_pythondevmode_env: ok")
