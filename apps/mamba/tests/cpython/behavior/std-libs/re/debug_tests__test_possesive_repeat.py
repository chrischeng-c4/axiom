# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "debug_tests__test_possesive_repeat"
# subject = "cpython.test_re.DebugTests.test_possesive_repeat"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::DebugTests::test_possesive_repeat
"""Auto-ported test: DebugTests::test_possesive_repeat (CPython 3.12 oracle)."""


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

assert get_debug_out('(?:ab)?+') == "POSSESSIVE_REPEAT 0 1\n  LITERAL 97\n  LITERAL 98\n\n 0. INFO 4 0b0 0 2 (to 5)\n 5: POSSESSIVE_REPEAT 7 0 1 (to 13)\n 9.   LITERAL 0x61 ('a')\n11.   LITERAL 0x62 ('b')\n13: SUCCESS\n14. SUCCESS\n"
print("DebugTests::test_possesive_repeat: ok")
