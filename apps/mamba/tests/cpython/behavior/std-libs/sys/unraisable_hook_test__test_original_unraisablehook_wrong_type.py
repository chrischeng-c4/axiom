# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "unraisable_hook_test__test_original_unraisablehook_wrong_type"
# subject = "cpython.test_sys.UnraisableHookTest.test_original_unraisablehook_wrong_type"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::UnraisableHookTest::test_original_unraisablehook_wrong_type
"""Auto-ported test: UnraisableHookTest::test_original_unraisablehook_wrong_type (CPython 3.12 oracle)."""


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
exc = ValueError(42)
with test.support.swap_attr(sys, 'unraisablehook', sys.__unraisablehook__):
    try:
        sys.unraisablehook(exc)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
print("UnraisableHookTest::test_original_unraisablehook_wrong_type: ok")
