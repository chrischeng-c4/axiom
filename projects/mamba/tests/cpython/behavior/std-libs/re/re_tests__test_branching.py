# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_branching"
# subject = "cpython.test_re.ReTests.test_branching"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_branching
"""Auto-ported test: ReTests::test_branching (CPython 3.12 oracle)."""


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
"Test Branching\n        Test expressions using the OR ('|') operator."

assert re.match('(ab|ba)', 'ab').span() == (0, 2)

assert re.match('(ab|ba)', 'ba').span() == (0, 2)

assert re.match('(abc|bac|ca|cb)', 'abc').span() == (0, 3)

assert re.match('(abc|bac|ca|cb)', 'bac').span() == (0, 3)

assert re.match('(abc|bac|ca|cb)', 'ca').span() == (0, 2)

assert re.match('(abc|bac|ca|cb)', 'cb').span() == (0, 2)

assert re.match('((a)|(b)|(c))', 'a').span() == (0, 1)

assert re.match('((a)|(b)|(c))', 'b').span() == (0, 1)

assert re.match('((a)|(b)|(c))', 'c').span() == (0, 1)
print("ReTests::test_branching: ok")
