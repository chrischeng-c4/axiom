# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "implementation_test__test_case_helpers"
# subject = "cpython.test_re.ImplementationTest.test_case_helpers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ImplementationTest::test_case_helpers
"""Auto-ported test: ImplementationTest::test_case_helpers (CPython 3.12 oracle)."""


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
import _sre
for i in range(128):
    c = chr(i)
    lo = ord(c.lower())

    assert _sre.ascii_tolower(i) == lo

    assert _sre.unicode_tolower(i) == lo
    iscased = c in string.ascii_letters

    assert _sre.ascii_iscased(i) == iscased

    assert _sre.unicode_iscased(i) == iscased
for i in list(range(128, 4096)) + [66560, 66600]:
    c = chr(i)

    assert _sre.ascii_tolower(i) == i
    if i != 304:

        assert _sre.unicode_tolower(i) == ord(c.lower())
    iscased = c != c.lower() or c != c.upper()

    assert not _sre.ascii_iscased(i)

    assert _sre.unicode_iscased(i) == (c != c.lower() or c != c.upper())

assert _sre.ascii_tolower(304) == 304

assert _sre.unicode_tolower(304) == ord('i')

assert not _sre.ascii_iscased(304)

assert _sre.unicode_iscased(304)
print("ImplementationTest::test_case_helpers: ok")
