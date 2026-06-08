# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_725106"
# subject = "cpython.test_re.ReTests.test_bug_725106"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_725106
"""Auto-ported test: ReTests::test_bug_725106 (CPython 3.12 oracle)."""


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

assert re.match('^((a)|b)*', 'abc').groups() == ('b', 'a')

assert re.match('^(([ab])|c)*', 'abc').groups() == ('c', 'b')

assert re.match('^((d)|[ab])*', 'abc').groups() == ('b', None)

assert re.match('^((a)c|[ab])*', 'abc').groups() == ('b', None)

assert re.match('^((a)|b)*?c', 'abc').groups() == ('b', 'a')

assert re.match('^(([ab])|c)*?d', 'abcd').groups() == ('c', 'b')

assert re.match('^((d)|[ab])*?c', 'abc').groups() == ('b', None)

assert re.match('^((a)c|[ab])*?c', 'abc').groups() == ('b', None)
print("ReTests::test_bug_725106: ok")
