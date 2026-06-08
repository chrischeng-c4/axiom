# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_ascii_and_unicode_flag"
# subject = "cpython.test_re.ReTests.test_ascii_and_unicode_flag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_ascii_and_unicode_flag
"""Auto-ported test: ReTests::test_ascii_and_unicode_flag (CPython 3.12 oracle)."""


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
for flags in (0, re.UNICODE):
    pat = re.compile('À', flags | re.IGNORECASE)

    assert pat.match('à')
    pat = re.compile('\\w', flags)

    assert pat.match('à')
pat = re.compile('À', re.ASCII | re.IGNORECASE)

assert pat.match('à') is None
pat = re.compile('(?a)À', re.IGNORECASE)

assert pat.match('à') is None
pat = re.compile('\\w', re.ASCII)

assert pat.match('à') is None
pat = re.compile('(?a)\\w')

assert pat.match('à') is None
for flags in (0, re.ASCII):
    pat = re.compile(b'\xc0', flags | re.IGNORECASE)

    assert pat.match(b'\xe0') is None
    pat = re.compile(b'\\w', flags)

    assert pat.match(b'\xe0') is None

try:
    re.compile(b'\\w', re.UNICODE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile(b'(?u)\\w')
    raise AssertionError('expected re.error')
except re.error:
    pass

try:
    re.compile('\\w', re.UNICODE | re.ASCII)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile('(?u)\\w', re.ASCII)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile('(?a)\\w', re.UNICODE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile('(?au)\\w')
    raise AssertionError('expected re.error')
except re.error:
    pass
print("ReTests::test_ascii_and_unicode_flag: ok")
