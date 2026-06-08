# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_scanner"
# subject = "cpython.test_re.ReTests.test_scanner"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_scanner
"""Auto-ported test: ReTests::test_scanner (CPython 3.12 oracle)."""


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

def s_ident(scanner, token):
    return token

def s_operator(scanner, token):
    return 'op%s' % token

def s_float(scanner, token):
    return float(token)

def s_int(scanner, token):
    return int(token)
scanner = Scanner([('[a-zA-Z_]\\w*', s_ident), ('\\d+\\.\\d*', s_float), ('\\d+', s_int), ('=|\\+|-|\\*|/', s_operator), ('\\s+', None)])

assert scanner.scanner.scanner('').pattern

assert scanner.scan('sum = 3*foo + 312.50 + bar') == (['sum', 'op=', 3, 'op*', 'foo', 'op+', 312.5, 'op+', 'bar'], '')
print("ReTests::test_scanner: ok")
