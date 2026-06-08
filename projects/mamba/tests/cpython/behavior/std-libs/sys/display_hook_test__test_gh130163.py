# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "display_hook_test__test_gh130163"
# subject = "cpython.test_sys.DisplayHookTest.test_gh130163"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_sys.py::DisplayHookTest::test_gh130163
"""Auto-ported test: DisplayHookTest::test_gh130163 (CPython 3.12 oracle)."""


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
class X:

    def __repr__(self):
        sys.stdout = io.StringIO()
        support.gc_collect()
        return 'foo'
with support.swap_attr(sys, 'stdout', None):
    sys.stdout = io.StringIO()
    sys.displayhook(X())
print("DisplayHookTest::test_gh130163: ok")
