# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_special_escapes"
# subject = "cpython.test_re.ReTests.test_special_escapes"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_special_escapes
"""Auto-ported test: ReTests::test_special_escapes (CPython 3.12 oracle)."""


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

assert re.search('\\b(b.)\\b', 'abcd abc bcd bx').group(1) == 'bx'

assert re.search('\\B(b.)\\B', 'abc bcd bc abxd').group(1) == 'bx'

assert re.search('\\b(b.)\\b', 'abcd abc bcd bx', re.ASCII).group(1) == 'bx'

assert re.search('\\B(b.)\\B', 'abc bcd bc abxd', re.ASCII).group(1) == 'bx'

assert re.search('^abc$', '\nabc\n', re.M).group(0) == 'abc'

assert re.search('^\\Aabc\\Z$', 'abc', re.M).group(0) == 'abc'

assert re.search('^\\Aabc\\Z$', '\nabc\n', re.M) is None

assert re.search(b'\\b(b.)\\b', b'abcd abc bcd bx').group(1) == b'bx'

assert re.search(b'\\B(b.)\\B', b'abc bcd bc abxd').group(1) == b'bx'

assert re.search(b'\\b(b.)\\b', b'abcd abc bcd bx', re.LOCALE).group(1) == b'bx'

assert re.search(b'\\B(b.)\\B', b'abc bcd bc abxd', re.LOCALE).group(1) == b'bx'

assert re.search(b'^abc$', b'\nabc\n', re.M).group(0) == b'abc'

assert re.search(b'^\\Aabc\\Z$', b'abc', re.M).group(0) == b'abc'

assert re.search(b'^\\Aabc\\Z$', b'\nabc\n', re.M) is None

assert re.search('\\d\\D\\w\\W\\s\\S', '1aa! a').group(0) == '1aa! a'

assert re.search(b'\\d\\D\\w\\W\\s\\S', b'1aa! a').group(0) == b'1aa! a'

assert re.search('\\d\\D\\w\\W\\s\\S', '1aa! a', re.ASCII).group(0) == '1aa! a'

assert re.search(b'\\d\\D\\w\\W\\s\\S', b'1aa! a', re.LOCALE).group(0) == b'1aa! a'
print("ReTests::test_special_escapes: ok")
