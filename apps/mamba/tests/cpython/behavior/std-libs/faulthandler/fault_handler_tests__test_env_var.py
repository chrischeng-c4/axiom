# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "behavior"
# case = "fault_handler_tests__test_env_var"
# subject = "cpython.test_faulthandler.FaultHandlerTests.test_env_var"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_faulthandler.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_faulthandler.py::FaultHandlerTests::test_env_var
"""Auto-ported test: FaultHandlerTests::test_env_var (CPython 3.12 oracle)."""


from contextlib import contextmanager
import datetime
import faulthandler
import os
import re
import signal
import subprocess
import sys
from test import support
from test.support import os_helper, script_helper, is_android, MS_WINDOWS
import tempfile
import unittest
from textwrap import dedent


try:
    import _testcapi
except ImportError:
    _testcapi = None

if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

TIMEOUT = 0.5

def expected_traceback(lineno1, lineno2, header, min_count=1):
    regex = header
    regex += '  File "<string>", line %s in func\n' % lineno1
    regex += '  File "<string>", line %s in <module>' % lineno2
    if 1 < min_count:
        return '^' + (regex + '\n') * (min_count - 1) + regex
    else:
        return '^' + regex + '$'

def skip_segfault_on_android(test):
    return unittest.skipIf(is_android, 'raising SIGSEGV on Android is unreliable')(test)

@contextmanager
def temporary_filename():
    filename = tempfile.mktemp()
    try:
        yield filename
    finally:
        os_helper.unlink(filename)


# --- test body ---
code = 'import faulthandler; print(faulthandler.is_enabled())'
args = (sys.executable, '-c', code)
env = dict(os.environ)
env['PYTHONFAULTHANDLER'] = ''
env['PYTHONDEVMODE'] = ''
output = subprocess.check_output(args, env=env)

assert output.rstrip() == b'False'
env = dict(os.environ)
env['PYTHONFAULTHANDLER'] = '1'
env['PYTHONDEVMODE'] = ''
output = subprocess.check_output(args, env=env)

assert output.rstrip() == b'True'
print("FaultHandlerTests::test_env_var: ok")
