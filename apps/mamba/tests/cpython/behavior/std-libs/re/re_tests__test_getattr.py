# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_getattr"
# subject = "cpython.test_re.ReTests.test_getattr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_getattr
"""Auto-ported test: ReTests::test_getattr (CPython 3.12 oracle)."""


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

assert re.compile('(?i)(a)(b)').pattern == '(?i)(a)(b)'

assert re.compile('(?i)(a)(b)').flags == re.I | re.U

assert re.compile('(?i)(a)(b)').groups == 2

assert re.compile('(?i)(a)(b)').groupindex == {}

assert re.compile('(?i)(?P<first>a)(?P<other>b)').groupindex == {'first': 1, 'other': 2}

assert re.match('(a)', 'a').pos == 0

assert re.match('(a)', 'a').endpos == 1

assert re.match('(a)', 'a').string == 'a'

assert re.match('(a)', 'a').regs == ((0, 1), (0, 1))

assert re.match('(a)', 'a').re
p = re.compile('(?i)(?P<first>a)(?P<other>b)')

assert sorted(p.groupindex) == ['first', 'other']

assert p.groupindex['other'] == 2
try:
    p.groupindex['other'] = 0
    raise AssertionError('expected TypeError')
except TypeError:
    pass

assert p.groupindex['other'] == 2
print("ReTests::test_getattr: ok")
