# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "behavior"
# case = "misc_tests__test_getoutput"
# subject = "cpython.test_subprocess.MiscTests.test_getoutput"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_subprocess.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_subprocess.py::MiscTests::test_getoutput
"""Auto-ported test: MiscTests::test_getoutput (CPython 3.12 oracle)."""


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
def _test_keyboardinterrupt_no_kill(popener, mock__communicate, **kwargs):
    """Fake a SIGINT happening during Popen._communicate() and ._wait().

        This avoids the need to actually try and get test environments to send
        and receive signals reliably across platforms.  The net effect of a ^C
        happening during a blocking subprocess execution which we want to clean
        up from is a KeyboardInterrupt coming out of communicate() or wait().
        """
    mock__communicate.side_effect = KeyboardInterrupt
    try:
        with mock.patch.object(subprocess.Popen, '_wait') as mock__wait:
            mock__wait.side_effect = KeyboardInterrupt
            with mock.patch.object(subprocess, 'Popen', self_RecordingPopen):
                try:
                    popener([sys.executable, '-c', "import time\ntime.sleep(9)\nimport sys\nsys.stderr.write('\\n!runaway child!\\n')"], stdout=subprocess.DEVNULL, **kwargs)
                    raise AssertionError('expected KeyboardInterrupt')
                except KeyboardInterrupt:
                    pass
            for call in mock__wait.call_args_list[1:]:

                assert call != mock.call(timeout=None)
            sigint_calls = []
            for call in mock__wait.call_args_list:
                if call == mock.call(timeout=0.25):
                    sigint_calls.append(call)

            assert mock__wait.call_count <= 2

            assert len(sigint_calls) == 1
    finally:
        process = self_RecordingPopen.instances_created.pop()
        process.kill()
        process.wait()

        assert [] == self_RecordingPopen.instances_created

assert subprocess.getoutput('echo xyzzy') == 'xyzzy'

assert subprocess.getstatusoutput('echo xyzzy') == (0, 'xyzzy')
dir = None
try:
    dir = tempfile.mkdtemp()
    name = os.path.join(dir, 'foo')
    status, output = subprocess.getstatusoutput(('type ' if mswindows else 'cat ') + name)

    assert status != 0
finally:
    if dir is not None:
        os.rmdir(dir)
print("MiscTests::test_getoutput: ok")
