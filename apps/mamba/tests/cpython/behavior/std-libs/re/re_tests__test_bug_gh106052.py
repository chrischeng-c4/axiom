# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_gh106052"
# subject = "cpython.test_re.ReTests.test_bug_gh106052"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_gh106052
"""Auto-ported test: ReTests::test_bug_gh106052 (CPython 3.12 oracle)."""


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
LITERAL_CHARS = string.ascii_letters + string.digits + '!"%\',/:;<=>@_`'

assert re.match('(?>(?:.(?!D))+)', 'ABCDE').span() == (0, 2)

assert re.match('(?:.(?!D))++', 'ABCDE').span() == (0, 2)

assert re.match('(?>(?:.(?!D))*)', 'ABCDE').span() == (0, 2)

assert re.match('(?:.(?!D))*+', 'ABCDE').span() == (0, 2)

assert re.match('(?>(?:.(?!D))?)', 'CDE').span() == (0, 0)

assert re.match('(?:.(?!D))?+', 'CDE').span() == (0, 0)

assert re.match('(?>(?:.(?!D)){1,3})', 'ABCDE').span() == (0, 2)

assert re.match('(?:.(?!D)){1,3}+', 'ABCDE').span() == (0, 2)

assert re.match('(?>(?:ab?c)+)', 'aca').span() == (0, 2)

assert re.match('(?:ab?c)++', 'aca').span() == (0, 2)

assert re.match('(?>(?:ab?c)*)', 'aca').span() == (0, 2)

assert re.match('(?:ab?c)*+', 'aca').span() == (0, 2)

assert re.match('(?>(?:ab?c)?)', 'a').span() == (0, 0)

assert re.match('(?:ab?c)?+', 'a').span() == (0, 0)

assert re.match('(?>(?:ab?c){1,3})', 'aca').span() == (0, 2)

assert re.match('(?:ab?c){1,3}+', 'aca').span() == (0, 2)
print("ReTests::test_bug_gh106052: ok")
