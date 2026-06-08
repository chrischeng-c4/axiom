# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "string_literals"
# dimension = "behavior"
# case = "test_literals__test_eval_bytes_incomplete"
# subject = "cpython.test_string_literals.TestLiterals.test_eval_bytes_incomplete"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_string_literals.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_string_literals.py::TestLiterals::test_eval_bytes_incomplete
"""Auto-ported test: TestLiterals::test_eval_bytes_incomplete (CPython 3.12 oracle)."""


import os
import sys
import shutil
import tempfile
import unittest
import warnings


"Test correct treatment of various string literals by the parser.\n\nThere are four types of string literals:\n\n    'abc'             -- normal str\n    r'abc'            -- raw str\n    b'xyz'            -- normal bytes\n    br'xyz' | rb'xyz' -- raw bytes\n\nThe difference between normal and raw strings is of course that in a\nraw string, \\ escapes (while still used to determine the end of the\nliteral) are not interpreted, so that r'\\x00' contains four\ncharacters: a backslash, an x, and two zeros; while '\\x00' contains a\nsingle character (code point zero).\n\nThe tricky thing is what should happen when non-ASCII bytes are used\ninside literals.  For bytes literals, this is considered illegal.  But\nfor str literals, those bytes are supposed to be decoded using the\nencoding declared for the file (UTF-8 by default).\n\nWe have to test this with various file encodings.  We also test it with\nexec()/eval(), which uses a different code path.\n\nThis file is really about correct treatment of encodings and\nbackslashes.  It doesn't concern itself with issues like single\nvs. double quotes or singly- vs. triply-quoted strings: that's dealt\nwith elsewhere (I assume).\n"

TEMPLATE = "# coding: %s\na = 'x'\nassert ord(a) == 120\nb = '\\x01'\nassert ord(b) == 1\nc = r'\\x01'\nassert list(map(ord, c)) == [92, 120, 48, 49]\nd = '\\x81'\nassert ord(d) == 0x81\ne = r'\\x81'\nassert list(map(ord, e)) == [92, 120, 56, 49]\nf = '\\u1881'\nassert ord(f) == 0x1881\ng = r'\\u1881'\nassert list(map(ord, g)) == [92, 117, 49, 56, 56, 49]\nh = '\\U0001d120'\nassert ord(h) == 0x1d120\ni = r'\\U0001d120'\nassert list(map(ord, i)) == [92, 85, 48, 48, 48, 49, 100, 49, 50, 48]\n"

def byte(i):
    return bytes([i])


# --- test body ---
self_save_path = sys.path[:]
self_tmpdir = tempfile.mkdtemp()
sys.path.insert(0, self_tmpdir)

try:
    eval(" b'\\x' ")
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass

try:
    eval(" b'\\x0' ")
    raise AssertionError('expected SyntaxError')
except SyntaxError:
    pass
print("TestLiterals::test_eval_bytes_incomplete: ok")
