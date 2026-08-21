# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "display_hook_test__test_original_displayhook"
# subject = "cpython.test_sys.DisplayHookTest.test_original_displayhook"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::DisplayHookTest::test_original_displayhook
"""Auto-ported test: DisplayHookTest::test_original_displayhook (CPython 3.12 oracle)."""


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
dh = sys.__displayhook__
with support.captured_stdout() as out:
    dh(42)

assert out.getvalue() == '42\n'

assert builtins._ == 42
del builtins._
with support.captured_stdout() as out:
    dh(None)

assert out.getvalue() == ''

assert not hasattr(builtins, '_')

try:
    dh()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
stdout = sys.stdout
try:
    del sys.stdout

    try:
        dh(42)
        raise AssertionError('expected RuntimeError')
    except RuntimeError:
        pass
finally:
    sys.stdout = stdout
print("DisplayHookTest::test_original_displayhook: ok")
