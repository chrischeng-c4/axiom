# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "win32_process_test_case__test_terminate"
# subject = "cpython.test_subprocess.Win32ProcessTestCase.test_terminate"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subprocess.py::Win32ProcessTestCase::test_terminate
"""Auto-ported test: Win32ProcessTestCase::test_terminate (CPython 3.12 oracle)."""


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
def _kill_dead_process(method, *args):
    p = subprocess.Popen([sys.executable, '-c', "if 1:\n                             import sys, time\n                             sys.stdout.write('x\\n')\n                             sys.stdout.flush()\n                             sys.exit(42)\n                             "], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    with p:
        p.stdout.read(1)
        time.sleep(1)
        getattr(p, method)(*args)
        _, stderr = p.communicate()

        assert stderr == b''
        rc = p.wait()

    assert rc == 42

def _kill_process(method, *args):
    p = subprocess.Popen([sys.executable, '-c', "if 1:\n                             import sys, time\n                             sys.stdout.write('x\\n')\n                             sys.stdout.flush()\n                             time.sleep(30)\n                             "], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    with p:
        p.stdout.read(1)
        getattr(p, method)(*args)
        _, stderr = p.communicate()

        assert stderr == b''
        returncode = p.wait()

    assert returncode != 0
support.reap_children()
_kill_process('terminate')
print("Win32ProcessTestCase::test_terminate: ok")
