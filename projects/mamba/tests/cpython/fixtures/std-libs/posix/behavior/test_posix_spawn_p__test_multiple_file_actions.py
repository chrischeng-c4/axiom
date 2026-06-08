# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "test_posix_spawn_p__test_multiple_file_actions"
# subject = "cpython.test_posix.TestPosixSpawnP.test_multiple_file_actions"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_posix.py::TestPosixSpawnP::test_multiple_file_actions
"""Auto-ported test: TestPosixSpawnP::test_multiple_file_actions (CPython 3.12 oracle)."""


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
file_actions = [(os.POSIX_SPAWN_OPEN, 3, os.path.realpath(__file__), os.O_RDONLY, 0), (os.POSIX_SPAWN_CLOSE, 0), (os.POSIX_SPAWN_DUP2, 1, 4)]
pid = spawn_func(NOOP_PROGRAM[0], NOOP_PROGRAM, os.environ, file_actions=file_actions)
support.wait_process(pid, exitcode=0)
print("TestPosixSpawnP::test_multiple_file_actions: ok")
