# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_repeat_minmax_overflow"
# subject = "cpython.test_re.ReTests.test_repeat_minmax_overflow"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_repeat_minmax_overflow
"""Auto-ported test: ReTests::test_repeat_minmax_overflow (CPython 3.12 oracle)."""


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
string = 'x' * 100000

assert re.match('.{65535}', string).span() == (0, 65535)

assert re.match('.{,65535}', string).span() == (0, 65535)

assert re.match('.{65535,}?', string).span() == (0, 65535)

assert re.match('.{65536}', string).span() == (0, 65536)

assert re.match('.{,65536}', string).span() == (0, 65536)

assert re.match('.{65536,}?', string).span() == (0, 65536)

try:
    re.compile('.{%d}' % 2 ** 128)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    re.compile('.{,%d}' % 2 ** 128)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    re.compile('.{%d,}?' % 2 ** 128)
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass

try:
    re.compile('.{%d,%d}' % (2 ** 129, 2 ** 128))
    raise AssertionError('expected OverflowError')
except OverflowError:
    pass
print("ReTests::test_repeat_minmax_overflow: ok")
