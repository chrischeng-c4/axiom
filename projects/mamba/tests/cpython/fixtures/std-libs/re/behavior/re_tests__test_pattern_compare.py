# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_pattern_compare"
# subject = "cpython.test_re.ReTests.test_pattern_compare"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_pattern_compare
"""Auto-ported test: ReTests::test_pattern_compare (CPython 3.12 oracle)."""


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
pattern1 = re.compile('abc', re.IGNORECASE)

assert pattern1 == pattern1

assert not pattern1 != pattern1
re.purge()
pattern2 = re.compile('abc', re.IGNORECASE)

assert hash(pattern2) == hash(pattern1)

assert pattern2 == pattern1
re.purge()
pattern3 = re.compile('XYZ', re.IGNORECASE)

assert pattern3 != pattern1
re.purge()
pattern4 = re.compile('abc')

assert pattern4 != pattern1
try:
    pattern1 < pattern2
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ReTests::test_pattern_compare: ok")
