# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_ignore_spaces"
# subject = "cpython.test_re.ReTests.test_ignore_spaces"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_ignore_spaces
"""Auto-ported test: ReTests::test_ignore_spaces (CPython 3.12 oracle)."""


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
for space in ' \t\n\r\x0b\x0c':

    assert re.fullmatch(space + 'a', 'a', re.VERBOSE)
for space in (b' ', b'\t', b'\n', b'\r', b'\x0b', b'\x0c'):

    assert re.fullmatch(space + b'a', b'a', re.VERBOSE)

assert re.fullmatch('(?x) a', 'a')

assert re.fullmatch(' (?x) a', 'a', re.VERBOSE)

assert re.fullmatch('(?x) (?x) a', 'a')

assert re.fullmatch(' a(?x: b) c', ' ab c')

assert re.fullmatch(' a(?-x: b) c', 'a bc', re.VERBOSE)

assert re.fullmatch('(?x) a(?-x: b) c', 'a bc')

assert re.fullmatch('(?x) a| b', 'a')

assert re.fullmatch('(?x) a| b', 'b')
print("ReTests::test_ignore_spaces: ok")
