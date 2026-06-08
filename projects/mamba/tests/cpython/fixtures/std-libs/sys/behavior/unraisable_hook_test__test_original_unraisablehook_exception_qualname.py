# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "unraisable_hook_test__test_original_unraisablehook_exception_qualname"
# subject = "cpython.test_sys.UnraisableHookTest.test_original_unraisablehook_exception_qualname"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::UnraisableHookTest::test_original_unraisablehook_exception_qualname
"""Auto-ported test: UnraisableHookTest::test_original_unraisablehook_exception_qualname (CPython 3.12 oracle)."""


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

class A:

    class B:

        class X(Exception):
            pass
for moduleName in ('builtins', '__main__', 'some_module'):
    A.B.X.__module__ = moduleName
    with test.support.captured_stderr() as stderr, test.support.swap_attr(sys, 'unraisablehook', sys.__unraisablehook__):
        expected = write_unraisable_exc(A.B.X(), 'msg', 'obj')
    report = stderr.getvalue()

    assert A.B.X.__qualname__ in report
    if moduleName in ['builtins', '__main__']:

        assert moduleName + '.' not in report
    else:

        assert moduleName + '.' in report
print("UnraisableHookTest::test_original_unraisablehook_exception_qualname: ok")
