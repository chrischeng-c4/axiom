# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_thread_info"
# subject = "cpython.test_sys.SysModuleTest.test_thread_info"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_thread_info
"""Auto-ported test: SysModuleTest::test_thread_info (CPython 3.12 oracle)."""


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
info = sys.thread_info

assert len(info) == 3

assert info.name in ('nt', 'pthread', 'pthread-stubs', 'solaris', None)

assert info.lock in ('semaphore', 'mutex+cond', None)
if sys.platform.startswith(('linux', 'freebsd')):

    assert info.name == 'pthread'
elif sys.platform == 'win32':

    assert info.name == 'nt'
elif sys.platform == 'emscripten':

    assert info.name in {'pthread', 'pthread-stubs'}
elif sys.platform == 'wasi':

    assert info.name == 'pthread-stubs'
print("SysModuleTest::test_thread_info: ok")
