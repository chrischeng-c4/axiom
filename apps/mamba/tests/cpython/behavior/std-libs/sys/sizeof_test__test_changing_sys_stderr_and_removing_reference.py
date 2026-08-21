# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sizeof_test__test_changing_sys_stderr_and_removing_reference"
# subject = "cpython.test_sys.SizeofTest.test_changing_sys_stderr_and_removing_reference"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SizeofTest::test_changing_sys_stderr_and_removing_reference
"""Auto-ported test: SizeofTest::test_changing_sys_stderr_and_removing_reference (CPython 3.12 oracle)."""


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
check_sizeof = test.support.check_sizeof
self_P = struct.calcsize('P')
self_longdigit = sys.int_info.sizeof_digit
import _testinternalcapi
self_gc_headsize = _testinternalcapi.SIZEOF_PYGC_HEAD
code = textwrap.dedent('\n            import sys\n            class MyStderr:\n                def write(self, s):\n                    sys.stderr = None\n            sys.stderr = MyStderr()\n            1/0\n        ')
rc, out, err = assert_python_failure('-c', code)

assert out == b''

assert err == b''
print("SizeofTest::test_changing_sys_stderr_and_removing_reference: ok")
