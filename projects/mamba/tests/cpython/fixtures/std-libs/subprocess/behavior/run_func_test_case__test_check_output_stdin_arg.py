# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "run_func_test_case__test_check_output_stdin_arg"
# subject = "cpython.test_subprocess.RunFuncTestCase.test_check_output_stdin_arg"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subprocess.py::RunFuncTestCase::test_check_output_stdin_arg
"""Auto-ported test: RunFuncTestCase::test_check_output_stdin_arg (CPython 3.12 oracle)."""


import unittest
from unittest import mock
from test import support
from test.support import check_sanitizer
from test.support import import_helper
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
import subprocess
import sys
import signal
import io
import itertools
import os
import errno
import tempfile
import time
import traceback
import types
import selectors
import sysconfig
import select
import shutil
import threading
import gc
import textwrap
import json
from test.support.os_helper import FakePath


try:
    import _testcapi
except ImportError:
    _testcapi = None

try:
    import pwd
except ImportError:
    pwd = None

try:
    import grp
except ImportError:
    grp = None

try:
    import fcntl
except:
    fcntl = None

if support.PGO:
    raise unittest.SkipTest('test is not helpful for PGO')

if not support.has_subprocess_support:
    raise unittest.SkipTest('test module requires subprocess')

mswindows = sys.platform == 'win32'

if mswindows:
    SETBINARY = 'import msvcrt; msvcrt.setmode(sys.stdout.fileno(), os.O_BINARY);'
else:
    SETBINARY = ''

NONEXISTING_CMD = ('nonexisting_i_hope',)

NONEXISTING_ERRORS = (FileNotFoundError, NotADirectoryError, PermissionError)

ZERO_RETURN_CMD = (sys.executable, '-c', 'pass')

def setUpModule():
    shell_true = shutil.which('true')
    if shell_true is None:
        return
    if os.access(shell_true, os.X_OK) and subprocess.run([shell_true]).returncode == 0:
        global ZERO_RETURN_CMD
        ZERO_RETURN_CMD = (shell_true,)

class PopenTestException(Exception):
    pass

class PopenExecuteChildRaises(subprocess.Popen):
    """Popen subclass for testing cleanup of subprocess.PIPE filehandles when
    _execute_child fails.
    """

    def _execute_child(self, *args, **kwargs):
        raise PopenTestException('Forced Exception for Test')

def _get_test_grp_name():
    for name_group in ('staff', 'nogroup', 'grp', 'nobody', 'nfsnobody'):
        if grp:
            try:
                grp.getgrnam(name_group)
            except KeyError:
                continue
            return name_group
    else:
        raise unittest.SkipTest('No identified group name to use for this test on this platform.')


# --- test body ---
def run_python(code, **kwargs):
    """Run Python code in a subprocess using subprocess.run"""
    argv = [sys.executable, '-c', code]
    return subprocess.run(argv, **kwargs)
support.reap_children()
tf = tempfile.TemporaryFile()
pass
tf.write(b'pear')
tf.seek(0)
cp = run_python('import sys; sys.stdout.write(sys.stdin.read().upper())', stdin=tf, stdout=subprocess.PIPE)

assert b'PEAR' in cp.stdout
print("RunFuncTestCase::test_check_output_stdin_arg: ok")
