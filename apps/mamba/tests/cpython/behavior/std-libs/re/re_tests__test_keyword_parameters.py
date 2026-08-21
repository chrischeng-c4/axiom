# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_keyword_parameters"
# subject = "cpython.test_re.ReTests.test_keyword_parameters"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_keyword_parameters
"""Auto-ported test: ReTests::test_keyword_parameters (CPython 3.12 oracle)."""


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
pat = re.compile('(ab)')

assert pat.match(string='abracadabra', pos=7, endpos=10).span() == (7, 9)

assert pat.fullmatch(string='abracadabra', pos=7, endpos=9).span() == (7, 9)

assert pat.search(string='abracadabra', pos=3, endpos=10).span() == (7, 9)

assert pat.findall(string='abracadabra', pos=3, endpos=10) == ['ab']

assert pat.split(string='abracadabra', maxsplit=1) == ['', 'ab', 'racadabra']

assert pat.scanner(string='abracadabra', pos=3, endpos=10).search().span() == (7, 9)
print("ReTests::test_keyword_parameters: ok")
