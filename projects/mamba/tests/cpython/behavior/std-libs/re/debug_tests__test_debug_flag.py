# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "debug_tests__test_debug_flag"
# subject = "cpython.test_re.DebugTests.test_debug_flag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::DebugTests::test_debug_flag
"""Auto-ported test: DebugTests::test_debug_flag (CPython 3.12 oracle)."""


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
pat = '(\\.)(?:[ch]|py)(?(1)$|: )'
dump = "SUBPATTERN 1 0 0\n  LITERAL 46\nBRANCH\n  IN\n    LITERAL 99\n    LITERAL 104\nOR\n  LITERAL 112\n  LITERAL 121\nGROUPREF_EXISTS 1\n  AT AT_END\nELSE\n  LITERAL 58\n  LITERAL 32\n\n 0. INFO 8 0b1 2 5 (to 9)\n      prefix_skip 0\n      prefix [0x2e] ('.')\n      overlap [0]\n 9: MARK 0\n11. LITERAL 0x2e ('.')\n13. MARK 1\n15. BRANCH 10 (to 26)\n17.   IN 6 (to 24)\n19.     LITERAL 0x63 ('c')\n21.     LITERAL 0x68 ('h')\n23.     FAILURE\n24:   JUMP 9 (to 34)\n26: branch 7 (to 33)\n27.   LITERAL 0x70 ('p')\n29.   LITERAL 0x79 ('y')\n31.   JUMP 2 (to 34)\n33: FAILURE\n34: GROUPREF_EXISTS 0 6 (to 41)\n37. AT END\n39. JUMP 5 (to 45)\n41: LITERAL 0x3a (':')\n43. LITERAL 0x20 (' ')\n45: SUCCESS\n"

assert get_debug_out(pat) == dump

assert get_debug_out(pat) == dump
print("DebugTests::test_debug_flag: ok")
