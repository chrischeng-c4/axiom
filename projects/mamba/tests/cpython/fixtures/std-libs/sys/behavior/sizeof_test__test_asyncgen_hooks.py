# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sizeof_test__test_asyncgen_hooks"
# subject = "cpython.test_sys.SizeofTest.test_asyncgen_hooks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_sys.py::SizeofTest::test_asyncgen_hooks
"""Auto-ported test: SizeofTest::test_asyncgen_hooks (CPython 3.12 oracle)."""


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
old = sys.get_asyncgen_hooks()

assert old.firstiter is None

assert old.finalizer is None
firstiter = lambda *a: None
sys.set_asyncgen_hooks(firstiter=firstiter)
hooks = sys.get_asyncgen_hooks()

assert hooks.firstiter is firstiter

assert hooks[0] is firstiter

assert hooks.finalizer is None

assert hooks[1] is None
finalizer = lambda *a: None
sys.set_asyncgen_hooks(finalizer=finalizer)
hooks = sys.get_asyncgen_hooks()

assert hooks.firstiter is firstiter

assert hooks[0] is firstiter

assert hooks.finalizer is finalizer

assert hooks[1] is finalizer
sys.set_asyncgen_hooks(*old)
cur = sys.get_asyncgen_hooks()

assert cur.firstiter is None

assert cur.finalizer is None
print("SizeofTest::test_asyncgen_hooks: ok")
