# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_current_frames"
# subject = "cpython.test_sys.SysModuleTest.test_current_frames"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_current_frames
"""Auto-ported test: SysModuleTest::test_current_frames (CPython 3.12 oracle)."""


import builtins
import codecs
import gc
import io
import locale
import operator
import os
import random
import struct
import subprocess
import sys
import sysconfig
import test.support
from test import support
from test.support import os_helper
from test.support.script_helper import assert_python_ok, assert_python_failure
from test.support import threading_helper
from test.support import import_helper
import textwrap
import unittest
import warnings


try:
    from test.support import interpreters
except ImportError:
    interpreters = None

def requires_subinterpreters(func):
    deco = unittest.skipIf(interpreters is None, 'Test requires subinterpreters')
    return deco(func)

DICT_KEY_STRUCT_FORMAT = 'n2BI2n'


# --- test body ---
import threading
import traceback
entered_g = threading.Event()
leave_g = threading.Event()
thread_info = []

def f123():
    g456()

def g456():
    thread_info.append(threading.get_ident())
    entered_g.set()
    leave_g.wait()
t = threading.Thread(target=f123)
t.start()
entered_g.wait()
try:

    assert len(thread_info) == 1
    thread_id = thread_info[0]
    d = sys._current_frames()
    for tid in d:

        assert isinstance(tid, int)

        assert tid > 0
    main_id = threading.get_ident()

    assert main_id in d

    assert thread_id in d
    frame = d.pop(main_id)

    assert frame is sys._getframe()
    frame = d.pop(thread_id)
    stack = traceback.extract_stack(frame)
    for i, (filename, lineno, funcname, sourceline) in enumerate(stack):
        if funcname == 'f123':
            break
    else:

        raise AssertionError("didn't find f123() on thread's call stack")

    assert sourceline == 'g456()'
    filename, lineno, funcname, sourceline = stack[i + 1]

    assert funcname == 'g456'

    assert sourceline in ['leave_g.wait()', 'entered_g.set()']
finally:
    leave_g.set()
    t.join()
print("SysModuleTest::test_current_frames: ok")
