# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "implementation_test__test_immutable"
# subject = "cpython.test_re.ImplementationTest.test_immutable"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ImplementationTest::test_immutable
"""Auto-ported test: ImplementationTest::test_immutable (CPython 3.12 oracle)."""


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
try:
    re.Match.foo = 1
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    re.Pattern.foo = 1
    raise AssertionError('expected TypeError')
except TypeError:
    pass
try:
    pat = re.compile('')
    tp = type(pat.scanner(''))
    tp.foo = 1
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ImplementationTest::test_immutable: ok")
