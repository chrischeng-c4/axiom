# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "test_posix_spawn_p__test_dup2"
# subject = "cpython.test_posix.TestPosixSpawnP.test_dup2"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_posix.py::TestPosixSpawnP::test_dup2
"""Auto-ported test: TestPosixSpawnP::test_dup2 (CPython 3.12 oracle)."""


from test import support
from test.support import import_helper
from test.support import os_helper
from test.support import warnings_helper
from test.support.script_helper import assert_python_ok
import copy
import errno
import sys
import signal
import time
import os
import platform
import pickle
import stat
import tempfile
import unittest
import warnings
import textwrap
from contextlib import contextmanager


'Test posix functions'

try:
    import posix
except ImportError:
    import nt as posix

try:
    import pwd
except ImportError:
    pwd = None

_DUMMY_SYMLINK = os.path.join(tempfile.gettempdir(), os_helper.TESTFN + '-dummy-symlink')

requires_32b = unittest.skipUnless(sys.maxsize < 2 ** 32 and (not (support.is_emscripten or support.is_wasi)), 'test is only meaningful on 32-bit builds')

def _supports_sched():
    if not hasattr(posix, 'sched_getscheduler'):
        return False
    try:
        posix.sched_getscheduler(0)
    except OSError as e:
        if e.errno == errno.ENOSYS:
            return False
    return True

requires_sched = unittest.skipUnless(_supports_sched(), 'requires POSIX scheduler API')

def tearDownModule():
    support.reap_children()


# --- test body ---
NOOP_PROGRAM = (sys.executable, '-I', '-S', '-c', 'pass')
spawn_func = None
spawn_func = getattr(posix, 'posix_spawnp', None)

def python_args(*args):
    return (sys.executable, '-I', '-S', *args)
dupfile = os_helper.TESTFN
pass
script = 'if 1:\n            import sys\n            sys.stdout.write("hello")\n            '
with open(dupfile, 'wb') as childfile:
    file_actions = [(os.POSIX_SPAWN_DUP2, childfile.fileno(), 1)]
    args = python_args('-c', script)
    pid = spawn_func(args[0], args, os.environ, file_actions=file_actions)
    support.wait_process(pid, exitcode=0)
with open(dupfile, encoding='utf-8') as f:

    assert f.read() == 'hello'
print("TestPosixSpawnP::test_dup2: ok")
