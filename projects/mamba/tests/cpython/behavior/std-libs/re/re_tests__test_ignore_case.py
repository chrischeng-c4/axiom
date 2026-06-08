# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_ignore_case"
# subject = "cpython.test_re.ReTests.test_ignore_case"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_ignore_case
"""Auto-ported test: ReTests::test_ignore_case (CPython 3.12 oracle)."""


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

assert re.match('abc', 'ABC', re.I).group(0) == 'ABC'

assert re.match(b'abc', b'ABC', re.I).group(0) == b'ABC'

assert re.match('(a\\s[^a])', 'a b', re.I).group(1) == 'a b'

assert re.match('(a\\s[^a]*)', 'a bb', re.I).group(1) == 'a bb'

assert re.match('(a\\s[abc])', 'a b', re.I).group(1) == 'a b'

assert re.match('(a\\s[abc]*)', 'a bb', re.I).group(1) == 'a bb'

assert re.match('((a)\\s\\2)', 'a a', re.I).group(1) == 'a a'

assert re.match('((a)\\s\\2*)', 'a aa', re.I).group(1) == 'a aa'

assert re.match('((a)\\s(abc|a))', 'a a', re.I).group(1) == 'a a'

assert re.match('((a)\\s(abc|a)*)', 'a aa', re.I).group(1) == 'a aa'
assert 'K'.lower() == 'K'.lower() == 'k'

assert re.match('K', 'K', re.I)

assert re.match('k', 'K', re.I)

assert re.match('\\u212a', 'K', re.I)

assert re.match('\\u212a', 'k', re.I)
assert 's'.upper() == 'ſ'.upper() == 'S'

assert re.match('S', 'ſ', re.I)

assert re.match('s', 'ſ', re.I)

assert re.match('\\u017f', 'S', re.I)

assert re.match('\\u017f', 's', re.I)
assert 'в'.upper() == 'ᲀ'.upper() == 'В'

assert re.match('\\u0412', 'в', re.I)

assert re.match('\\u0412', 'ᲀ', re.I)

assert re.match('\\u0432', 'В', re.I)

assert re.match('\\u0432', 'ᲀ', re.I)

assert re.match('\\u1c80', 'В', re.I)

assert re.match('\\u1c80', 'в', re.I)
assert 'ﬅ'.upper() == 'ﬆ'.upper() == 'ST'

assert re.match('\\ufb05', 'ﬆ', re.I)

assert re.match('\\ufb06', 'ﬅ', re.I)
print("ReTests::test_ignore_case: ok")
