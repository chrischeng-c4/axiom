# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_repr_tests__test_unknown_flags"
# subject = "cpython.test_re.PatternReprTests.test_unknown_flags"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::PatternReprTests::test_unknown_flags
"""Auto-ported test: PatternReprTests::test_unknown_flags (CPython 3.12 oracle)."""


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
def check(pattern, expected):

    assert repr(re.compile(pattern)) == expected

def check_flags(pattern, flags, expected):

    assert repr(re.compile(pattern, flags)) == expected
check_flags('random pattern', 1191936, "re.compile('random pattern', 0x123000)")
check_flags('random pattern', 1191936 | re.I, "re.compile('random pattern', re.IGNORECASE|0x123000)")
print("PatternReprTests::test_unknown_flags: ok")
