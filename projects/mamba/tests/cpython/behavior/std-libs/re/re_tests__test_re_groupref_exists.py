# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_re_groupref_exists"
# subject = "cpython.test_re.ReTests.test_re_groupref_exists"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_re_groupref_exists
"""Auto-ported test: ReTests::test_re_groupref_exists (CPython 3.12 oracle)."""


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

assert re.match('^(\\()?([^()]+)(?(1)\\))$', '(a)').groups() == ('(', 'a')

assert re.match('^(\\()?([^()]+)(?(1)\\))$', 'a').groups() == (None, 'a')

assert re.match('^(\\()?([^()]+)(?(1)\\))$', 'a)') is None

assert re.match('^(\\()?([^()]+)(?(1)\\))$', '(a') is None

assert re.match('^(?:(a)|c)((?(1)b|d))$', 'ab').groups() == ('a', 'b')

assert re.match('^(?:(a)|c)((?(1)b|d))$', 'cd').groups() == (None, 'd')

assert re.match('^(?:(a)|c)((?(1)|d))$', 'cd').groups() == (None, 'd')

assert re.match('^(?:(a)|c)((?(1)|d))$', 'a').groups() == ('a', '')
p = re.compile('(?P<g1>a)(?P<g2>b)?((?(g2)c|d))')

assert p.match('abc').groups() == ('a', 'b', 'c')

assert p.match('ad').groups() == ('a', None, 'd')

assert p.match('abd') is None

assert p.match('ac') is None
pat = '|'.join(('x(?P<a%d>%x)y' % (i, i) for i in range(1, 200 + 1)))
pat = '(?:%s)(?(200)z)' % pat

assert re.match(pat, 'xc8yz').span() == (0, 5)
print("ReTests::test_re_groupref_exists: ok")
