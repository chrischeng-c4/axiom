# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "debug_tests__test_atomic_group"
# subject = "cpython.test_re.DebugTests.test_atomic_group"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::DebugTests::test_atomic_group
"""Auto-ported test: DebugTests::test_atomic_group (CPython 3.12 oracle)."""


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
maxDiff = None

assert get_debug_out('(?>ab?)') == "ATOMIC_GROUP\n  LITERAL 97\n  MAX_REPEAT 0 1\n    LITERAL 98\n\n 0. INFO 4 0b0 1 2 (to 5)\n 5: ATOMIC_GROUP 11 (to 17)\n 7.   LITERAL 0x61 ('a')\n 9.   REPEAT_ONE 6 0 1 (to 16)\n13.     LITERAL 0x62 ('b')\n15.     SUCCESS\n16:   SUCCESS\n17: SUCCESS\n"
print("DebugTests::test_atomic_group: ok")
