# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_re_subn"
# subject = "cpython.test_re.ReTests.test_re_subn"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_re_subn
"""Auto-ported test: ReTests::test_re_subn (CPython 3.12 oracle)."""


from test.support import gc_collect, bigmemtest, _2G, cpython_only, captured_stdout, check_disallow_instantiation, is_emscripten, is_wasi, SHORT_TIMEOUT, requires_resource
import locale
import re
import string
import sys
import time
import unittest
import warnings
from re import Scanner
from weakref import proxy


try:
    import _multiprocessing
except ImportError:
    multiprocessing = None
else:
    import multiprocessing

class S(str):

    def __getitem__(self, index):
        return S(super().__getitem__(index))

class B(bytes):

    def __getitem__(self, index):
        return B(super().__getitem__(index))

def get_debug_out(pat):
    with captured_stdout() as out:
        re.compile(pat, re.DEBUG)
    return out.getvalue()


# --- test body ---
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.subn('(?i)b+', 'x', 'bbbb BBBB') == ('x x', 2)

assert re.subn('b+', 'x', 'bbbb BBBB') == ('x BBBB', 1)

assert re.subn('b+', 'x', 'xyz') == ('xyz', 0)

assert re.subn('b*', 'x', 'xyz') == ('xxxyxzx', 4)

assert re.subn('b*', 'x', 'xyz', 2) == ('xxxyz', 2)

assert re.subn('b*', 'x', 'xyz', count=2) == ('xxxyz', 2)
print("ReTests::test_re_subn: ok")
