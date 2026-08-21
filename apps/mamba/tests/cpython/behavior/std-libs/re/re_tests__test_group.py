# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_group"
# subject = "cpython.test_re.ReTests.test_group"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_group
"""Auto-ported test: ReTests::test_group (CPython 3.12 oracle)."""


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

class Index:

    def __init__(self, value):
        self.value = value

    def __index__(self):
        return self.value
m = re.match('(a)(b)', 'ab')

assert m.group() == 'ab'

assert m.group(0) == 'ab'

assert m.group(1) == 'a'

assert m.group(Index(1)) == 'a'

try:
    m.group(-1)
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    m.group(3)
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    m.group(1 << 1000)
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    m.group(Index(1 << 1000))
    raise AssertionError('expected IndexError')
except IndexError:
    pass

try:
    m.group('x')
    raise AssertionError('expected IndexError')
except IndexError:
    pass

assert m.group(2, 1) == ('b', 'a')

assert m.group(Index(2), Index(1)) == ('b', 'a')
print("ReTests::test_group: ok")
