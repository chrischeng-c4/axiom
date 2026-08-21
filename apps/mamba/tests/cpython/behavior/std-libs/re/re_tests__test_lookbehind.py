# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_lookbehind"
# subject = "cpython.test_re.ReTests.test_lookbehind"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_lookbehind
"""Auto-ported test: ReTests::test_lookbehind (CPython 3.12 oracle)."""


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

assert re.match('ab(?<=b)c', 'abc')

assert re.match('ab(?<=c)c', 'abc') is None

assert re.match('ab(?<!b)c', 'abc') is None

assert re.match('ab(?<!c)c', 'abc')

assert re.match('(a)a(?<=\\1)c', 'aac')

assert re.match('(a)b(?<=\\1)a', 'abaa') is None

assert re.match('(a)a(?<!\\1)c', 'aac') is None

assert re.match('(a)b(?<!\\1)a', 'abaa')

assert re.match('(?:(a)|(x))b(?<=(?(2)x|c))c', 'abc') is None

assert re.match('(?:(a)|(x))b(?<=(?(2)b|x))c', 'abc') is None

assert re.match('(?:(a)|(x))b(?<=(?(2)x|b))c', 'abc')

assert re.match('(?:(a)|(x))b(?<=(?(1)c|x))c', 'abc') is None

assert re.match('(?:(a)|(x))b(?<=(?(1)b|x))c', 'abc')

try:
    re.compile('(a)b(?<=(?(2)b|x))(c)')
    raise AssertionError('expected re.error')
except re.error:
    pass

assert re.match('(a)b(?<=(?(1)c|x))(c)', 'abc') is None

assert re.match('(a)b(?<=(?(1)b|x))(c)', 'abc')

try:
    re.compile('(a)b(?<=(.)\\2)(c)')
    raise AssertionError('expected re.error')
except re.error:
    pass

try:
    re.compile('(a)b(?<=(?P<a>.)(?P=a))(c)')
    raise AssertionError('expected re.error')
except re.error:
    pass

try:
    re.compile('(a)b(?<=(a)(?(2)b|x))(c)')
    raise AssertionError('expected re.error')
except re.error:
    pass

try:
    re.compile('(a)b(?<=(.)(?<=\\2))(c)')
    raise AssertionError('expected re.error')
except re.error:
    pass
print("ReTests::test_lookbehind: ok")
