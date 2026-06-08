# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "unraisable_hook_test__test_custom_unraisablehook_fail"
# subject = "cpython.test_sys.UnraisableHookTest.test_custom_unraisablehook_fail"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::UnraisableHookTest::test_custom_unraisablehook_fail
"""Auto-ported test: UnraisableHookTest::test_custom_unraisablehook_fail (CPython 3.12 oracle)."""


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
def write_unraisable_exc(exc, err_msg, obj):
    import _testcapi
    import types
    err_msg2 = f'Exception ignored {err_msg}'
    try:
        _testcapi.write_unraisable_exc(exc, err_msg, obj)
        return types.SimpleNamespace(exc_type=type(exc), exc_value=exc, exc_traceback=exc.__traceback__, err_msg=err_msg2, object=obj)
    finally:
        exc = None

def hook_func(*args):
    raise Exception('hook_func failed')
with test.support.captured_output('stderr') as stderr:
    with test.support.swap_attr(sys, 'unraisablehook', hook_func):
        write_unraisable_exc(ValueError(42), 'custom hook fail', None)
err = stderr.getvalue()

assert f'Exception ignored in sys.unraisablehook: {hook_func!r}\n' in err

assert 'Traceback (most recent call last):\n' in err

assert 'Exception: hook_func failed\n' in err
print("UnraisableHookTest::test_custom_unraisablehook_fail: ok")
