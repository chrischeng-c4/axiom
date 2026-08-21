# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_switchinterval"
# subject = "cpython.test_sys.SysModuleTest.test_switchinterval"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SysModuleTest::test_switchinterval
"""Auto-ported test: SysModuleTest::test_switchinterval (CPython 3.12 oracle)."""


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

try:
    sys.setswitchinterval()
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sys.setswitchinterval('a')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    sys.setswitchinterval(-1.0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    sys.setswitchinterval(0.0)
    raise AssertionError('expected ValueError')
except ValueError:
    pass
orig = sys.getswitchinterval()

assert orig < 0.5
try:
    for n in (1e-05, 0.05, 3.0, orig):
        sys.setswitchinterval(n)

        assert abs(sys.getswitchinterval() - n) < 1e-07
finally:
    sys.setswitchinterval(orig)
print("SysModuleTest::test_switchinterval: ok")
