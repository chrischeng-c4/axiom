# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_min_until_mark_bug"
# subject = "cpython.test_re.ReTests.test_MIN_UNTIL_mark_bug"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_MIN_UNTIL_mark_bug
"""Auto-ported test: ReTests::test_MIN_UNTIL_mark_bug (CPython 3.12 oracle)."""


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
s = 'axxzbcz'
p = '(?:(?:a|bc)*?(xx)??z)*'

assert re.match(p, s).groups() == ('xx',)
s = 'xtcxyzxc'
p = '((x|yz)+?(t)??c)*'
m = re.match(p, s)

assert m.span() == (0, 8)

assert m.span(2) == (6, 7)

assert m.groups() == ('xyzxc', 'x', 't')
print("ReTests::test_MIN_UNTIL_mark_bug: ok")
