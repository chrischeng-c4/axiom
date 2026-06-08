# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_re_fullmatch"
# subject = "cpython.test_re.ReTests.test_re_fullmatch"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_re_fullmatch
"""Auto-ported test: ReTests::test_re_fullmatch (CPython 3.12 oracle)."""


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

assert re.fullmatch('a', 'a').span() == (0, 1)
for string in ('ab', S('ab')):

    assert re.fullmatch('a|ab', string).span() == (0, 2)
for string in (b'ab', B(b'ab'), bytearray(b'ab'), memoryview(b'ab')):

    assert re.fullmatch(b'a|ab', string).span() == (0, 2)
for a, b in ('àß', 'аб', '𝒜𝒞'):
    r = '%s|%s' % (a, a + b)

    assert re.fullmatch(r, a + b).span() == (0, 2)

assert re.fullmatch('.*?$', 'abc').span() == (0, 3)

assert re.fullmatch('.*?', 'abc').span() == (0, 3)

assert re.fullmatch('a.*?b', 'ab').span() == (0, 2)

assert re.fullmatch('a.*?b', 'abb').span() == (0, 3)

assert re.fullmatch('a.*?b', 'axxb').span() == (0, 4)

assert re.fullmatch('a+', 'ab') is None

assert re.fullmatch('abc$', 'abc\n') is None

assert re.fullmatch('abc\\Z', 'abc\n') is None

assert re.fullmatch('(?m)abc$', 'abc\n') is None

assert re.fullmatch('ab(?=c)cd', 'abcd').span() == (0, 4)

assert re.fullmatch('ab(?<=b)cd', 'abcd').span() == (0, 4)

assert re.fullmatch('(?=a|ab)ab', 'ab').span() == (0, 2)

assert re.compile('bc').fullmatch('abcd', pos=1, endpos=3).span() == (1, 3)

assert re.compile('.*?$').fullmatch('abcd', pos=1, endpos=3).span() == (1, 3)

assert re.compile('.*?').fullmatch('abcd', pos=1, endpos=3).span() == (1, 3)
print("ReTests::test_re_fullmatch: ok")
