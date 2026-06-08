# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "pattern_repr_tests__test_flags_repr"
# subject = "cpython.test_re.PatternReprTests.test_flags_repr"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::PatternReprTests::test_flags_repr
"""Auto-ported test: PatternReprTests::test_flags_repr (CPython 3.12 oracle)."""


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

assert repr(re.I) == 're.IGNORECASE'

assert repr(re.I | re.S | re.X) == 're.IGNORECASE|re.DOTALL|re.VERBOSE'

assert repr(re.I | re.S | re.X | 1 << 20) == 're.IGNORECASE|re.DOTALL|re.VERBOSE|0x100000'

assert repr(~re.I) == 're.ASCII|re.LOCALE|re.UNICODE|re.MULTILINE|re.DOTALL|re.VERBOSE|re.TEMPLATE|re.DEBUG'

assert repr(~(re.I | re.S | re.X)) == 're.ASCII|re.LOCALE|re.UNICODE|re.MULTILINE|re.TEMPLATE|re.DEBUG'

assert repr(~(re.I | re.S | re.X | 1 << 20)) == 're.ASCII|re.LOCALE|re.UNICODE|re.MULTILINE|re.TEMPLATE|re.DEBUG|0xffe00'
print("PatternReprTests::test_flags_repr: ok")
