# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "except_hook_test__test_excepthook_bytes_filename"
# subject = "cpython.test_sys.ExceptHookTest.test_excepthook_bytes_filename"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::ExceptHookTest::test_excepthook_bytes_filename
"""Auto-ported test: ExceptHookTest::test_excepthook_bytes_filename (CPython 3.12 oracle)."""


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
with warnings.catch_warnings():
    warnings.simplefilter('ignore', BytesWarning)
    try:
        raise SyntaxError('msg', (b'bytes_filename', 123, 0, 'text'))
    except SyntaxError as exc:
        with support.captured_stderr() as err:
            sys.__excepthook__(*sys.exc_info())
err = err.getvalue()

assert '  File "b\'bytes_filename\'", line 123\n' in err

assert '    text\n' in err

assert err.endswith('SyntaxError: msg\n')
print("ExceptHookTest::test_excepthook_bytes_filename: ok")
