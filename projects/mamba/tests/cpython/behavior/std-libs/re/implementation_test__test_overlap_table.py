# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "implementation_test__test_overlap_table"
# subject = "cpython.test_re.ImplementationTest.test_overlap_table"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ImplementationTest::test_overlap_table
"""Auto-ported test: ImplementationTest::test_overlap_table (CPython 3.12 oracle)."""


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
f = re._compiler._generate_overlap_table

assert f('') == []

assert f('a') == [0]

assert f('abcd') == [0, 0, 0, 0]

assert f('aaaa') == [0, 1, 2, 3]

assert f('ababba') == [0, 0, 1, 2, 0, 1]

assert f('abcabdac') == [0, 0, 0, 1, 2, 0, 1, 0]
print("ImplementationTest::test_overlap_table: ok")
