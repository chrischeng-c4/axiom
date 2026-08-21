# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "external_tests__test_re_benchmarks"
# subject = "cpython.test_re.ExternalTests.test_re_benchmarks"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ExternalTests::test_re_benchmarks
"""Auto-ported test: ExternalTests::test_re_benchmarks (CPython 3.12 oracle)."""


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
"""re_tests benchmarks"""
from test.re_tests import benchmarks
for pattern, s in benchmarks:
    p = re.compile(pattern)

    assert p.search(s)

    assert p.match(s)

    assert p.fullmatch(s)
    s2 = ' ' * 10000 + s + ' ' * 10000

    assert p.search(s2)

    assert p.match(s2, 10000)

    assert p.match(s2, 10000, 10000 + len(s))

    assert p.fullmatch(s2, 10000, 10000 + len(s))
print("ExternalTests::test_re_benchmarks: ok")
