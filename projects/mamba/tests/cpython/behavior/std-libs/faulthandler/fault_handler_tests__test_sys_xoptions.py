# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "faulthandler"
# dimension = "behavior"
# case = "fault_handler_tests__test_sys_xoptions"
# subject = "cpython.test_faulthandler.FaultHandlerTests.test_sys_xoptions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_faulthandler.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_faulthandler.py::FaultHandlerTests::test_sys_xoptions
"""Auto-ported test: FaultHandlerTests::test_sys_xoptions (CPython 3.12 oracle)."""


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
args = filter(None, (sys.executable, '-E' if sys.flags.ignore_environment else '', '-X', 'faulthandler', '-c', code))
env = os.environ.copy()
env.pop('PYTHONFAULTHANDLER', None)
output = subprocess.check_output(args, env=env)

assert output.rstrip() == b'True'
print("FaultHandlerTests::test_sys_xoptions: ok")
