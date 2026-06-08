# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_zerowidth"
# subject = "cpython.test_re.ReTests.test_zerowidth"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_zerowidth
"""Auto-ported test: ReTests::test_zerowidth (CPython 3.12 oracle)."""


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

assert re.split('\\b', 'a::bc') == ['', 'a', '::', 'bc', '']

assert re.split('\\b|:+', 'a::bc') == ['', 'a', '', '', 'bc', '']

assert re.split('(?<!\\w)(?=\\w)|:+', 'a::bc') == ['', 'a', '', 'bc']

assert re.split('(?<=\\w)(?!\\w)|:+', 'a::bc') == ['a', '', 'bc', '']

assert re.sub('\\b', '-', 'a::bc') == '-a-::-bc-'

assert re.sub('\\b|:+', '-', 'a::bc') == '-a---bc-'

assert re.sub('(\\b|:+)', '[\\1]', 'a::bc') == '[]a[][::][]bc[]'

assert re.findall('\\b|:+', 'a::bc') == ['', '', '::', '', '']

assert re.findall('\\b|\\w+', 'a::bc') == ['', 'a', '', '', 'bc', '']

assert [m.span() for m in re.finditer('\\b|:+', 'a::bc')] == [(0, 0), (1, 1), (1, 3), (3, 3), (5, 5)]

assert [m.span() for m in re.finditer('\\b|\\w+', 'a::bc')] == [(0, 0), (0, 1), (1, 1), (3, 3), (3, 5), (5, 5)]
print("ReTests::test_zerowidth: ok")
