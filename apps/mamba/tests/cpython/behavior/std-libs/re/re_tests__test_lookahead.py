# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_lookahead"
# subject = "cpython.test_re.ReTests.test_lookahead"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_lookahead
"""Auto-ported test: ReTests::test_lookahead (CPython 3.12 oracle)."""


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

assert re.match('(a(?=\\s[^a]))', 'a b').group(1) == 'a'

assert re.match('(a(?=\\s[^a]*))', 'a b').group(1) == 'a'

assert re.match('(a(?=\\s[abc]))', 'a b').group(1) == 'a'

assert re.match('(a(?=\\s[abc]*))', 'a bc').group(1) == 'a'

assert re.match('(a)(?=\\s\\1)', 'a a').group(1) == 'a'

assert re.match('(a)(?=\\s\\1*)', 'a aa').group(1) == 'a'

assert re.match('(a)(?=\\s(abc|a))', 'a a').group(1) == 'a'

assert re.match('(a(?!\\s[^a]))', 'a a').group(1) == 'a'

assert re.match('(a(?!\\s[abc]))', 'a d').group(1) == 'a'

assert re.match('(a)(?!\\s\\1)', 'a b').group(1) == 'a'

assert re.match('(a)(?!\\s(abc|a))', 'a b').group(1) == 'a'

assert re.match('(a)b(?=\\1)a', 'aba')

assert re.match('(a)b(?=\\1)c', 'abac') is None

assert re.match('(?:(a)|(x))b(?=(?(2)x|c))c', 'abc')

assert re.match('(?:(a)|(x))b(?=(?(2)c|x))c', 'abc') is None

assert re.match('(?:(a)|(x))b(?=(?(2)x|c))c', 'abc')

assert re.match('(?:(a)|(x))b(?=(?(1)b|x))c', 'abc') is None

assert re.match('(?:(a)|(x))b(?=(?(1)c|x))c', 'abc')

assert re.match('(a)b(?=(?(2)x|c))(c)', 'abc')

assert re.match('(a)b(?=(?(2)b|x))(c)', 'abc') is None

assert re.match('(a)b(?=(?(1)c|x))(c)', 'abc')
print("ReTests::test_lookahead: ok")
