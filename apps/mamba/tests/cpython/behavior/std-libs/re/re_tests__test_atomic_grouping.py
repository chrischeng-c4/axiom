# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_atomic_grouping"
# subject = "cpython.test_re.ReTests.test_atomic_grouping"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_atomic_grouping
"""Auto-ported test: ReTests::test_atomic_grouping (CPython 3.12 oracle)."""


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
'Test Atomic Grouping\n        Test non-capturing groups of the form (?>...), which does\n        not maintain any stack point created within the group once the\n        group is finished being evaluated.'
pattern1 = re.compile('a(?>bc|b)c')

assert pattern1.match('abc') is None

assert pattern1.match('abcc')

assert re.match('(?>.*).', 'abc') is None

assert re.match('(?>x)++', 'xxx')

assert re.match('(?>x++)', 'xxx')

assert re.match('(?>x)++x', 'xxx') is None

assert re.match('(?>x++)x', 'xxx') is None
print("ReTests::test_atomic_grouping: ok")
