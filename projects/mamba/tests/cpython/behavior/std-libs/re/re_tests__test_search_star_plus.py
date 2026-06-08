# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_search_star_plus"
# subject = "cpython.test_re.ReTests.test_search_star_plus"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_search_star_plus
"""Auto-ported test: ReTests::test_search_star_plus (CPython 3.12 oracle)."""


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

assert re.search('x*', 'axx').span(0) == (0, 0)

assert re.search('x*', 'axx').span() == (0, 0)

assert re.search('x+', 'axx').span(0) == (1, 3)

assert re.search('x+', 'axx').span() == (1, 3)

assert re.search('x', 'aaa') is None

assert re.match('a*', 'xxx').span(0) == (0, 0)

assert re.match('a*', 'xxx').span() == (0, 0)

assert re.match('x*', 'xxxa').span(0) == (0, 3)

assert re.match('x*', 'xxxa').span() == (0, 3)

assert re.match('a+', 'xxx') is None
print("ReTests::test_search_star_plus: ok")
