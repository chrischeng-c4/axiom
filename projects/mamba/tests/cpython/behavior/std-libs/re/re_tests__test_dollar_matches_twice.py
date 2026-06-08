# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_dollar_matches_twice"
# subject = "cpython.test_re.ReTests.test_dollar_matches_twice"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_dollar_matches_twice
"""Auto-ported test: ReTests::test_dollar_matches_twice (CPython 3.12 oracle)."""


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
'Test that $ does not include \\n\n        $ matches the end of string, and just before the terminating \\n'
pattern = re.compile('$')

assert pattern.sub('#', 'a\nb\n') == 'a\nb#\n#'

assert pattern.sub('#', 'a\nb\nc') == 'a\nb\nc#'

assert pattern.sub('#', '\n') == '#\n#'
pattern = re.compile('$', re.MULTILINE)

assert pattern.sub('#', 'a\nb\n') == 'a#\nb#\n#'

assert pattern.sub('#', 'a\nb\nc') == 'a#\nb#\nc#'

assert pattern.sub('#', '\n') == '#\n#'
print("ReTests::test_dollar_matches_twice: ok")
