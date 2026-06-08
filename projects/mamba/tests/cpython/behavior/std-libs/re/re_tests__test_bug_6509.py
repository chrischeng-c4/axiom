# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_6509"
# subject = "cpython.test_re.ReTests.test_bug_6509"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_6509
"""Auto-ported test: ReTests::test_bug_6509 (CPython 3.12 oracle)."""


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
pat = re.compile('a(\\w)')

assert pat.sub('b\\1', 'ac') == 'bc'
pat = re.compile('a(.)')

assert pat.sub('b\\1', 'aሴ') == 'bሴ'
pat = re.compile('..')

assert pat.sub(lambda m: 'str', 'a5') == 'str'
pat = re.compile(b'a(\\w)')

assert pat.sub(b'b\\1', b'ac') == b'bc'
pat = re.compile(b'a(.)')

assert pat.sub(b'b\\1', b'a\xcd') == b'b\xcd'
pat = re.compile(b'..')

assert pat.sub(lambda m: b'bytes', b'a5') == b'bytes'
print("ReTests::test_bug_6509: ok")
