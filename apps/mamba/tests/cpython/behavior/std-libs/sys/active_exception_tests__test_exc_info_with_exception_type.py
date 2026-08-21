# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "active_exception_tests__test_exc_info_with_exception_type"
# subject = "cpython.test_sys.ActiveExceptionTests.test_exc_info_with_exception_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::ActiveExceptionTests::test_exc_info_with_exception_type
"""Auto-ported test: ActiveExceptionTests::test_exc_info_with_exception_type (CPython 3.12 oracle)."""


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
def f():
    raise ValueError
try:
    f()
except Exception as e_:
    e = e_
    exc_info = sys.exc_info()

assert isinstance(e, ValueError)

assert exc_info[0] is ValueError

assert exc_info[1] is e

assert exc_info[2] is e.__traceback__
print("ActiveExceptionTests::test_exc_info_with_exception_type: ok")
