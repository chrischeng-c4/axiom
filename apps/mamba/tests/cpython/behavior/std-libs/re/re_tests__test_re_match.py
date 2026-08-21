# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_re_match"
# subject = "cpython.test_re.ReTests.test_re_match"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_re_match
"""Auto-ported test: ReTests::test_re_match (CPython 3.12 oracle)."""


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
for string in ('a', S('a')):

    assert re.match('a', string).groups() == ()

    assert re.match('(a)', string).groups() == ('a',)

    assert re.match('(a)', string).group(0) == 'a'

    assert re.match('(a)', string).group(1) == 'a'

    assert re.match('(a)', string).group(1, 1) == ('a', 'a')
for string in (b'a', B(b'a'), bytearray(b'a'), memoryview(b'a')):

    assert re.match(b'a', string).groups() == ()

    assert re.match(b'(a)', string).groups() == (b'a',)

    assert re.match(b'(a)', string).group(0) == b'a'

    assert re.match(b'(a)', string).group(1) == b'a'

    assert re.match(b'(a)', string).group(1, 1) == (b'a', b'a')
for a in ('à', 'а', '𝒜'):

    assert re.match(a, a).groups() == ()

    assert re.match('(%s)' % a, a).groups() == (a,)

    assert re.match('(%s)' % a, a).group(0) == a

    assert re.match('(%s)' % a, a).group(1) == a

    assert re.match('(%s)' % a, a).group(1, 1) == (a, a)
pat = re.compile('((a)|(b))(c)?')

assert pat.match('a').groups() == ('a', 'a', None, None)

assert pat.match('b').groups() == ('b', None, 'b', None)

assert pat.match('ac').groups() == ('a', 'a', None, 'c')

assert pat.match('bc').groups() == ('b', None, 'b', 'c')

assert pat.match('bc').groups('') == ('b', '', 'b', 'c')
pat = re.compile('(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?')

assert pat.match('a').group(1, 2, 3) == ('a', None, None)

assert pat.match('b').group('a1', 'b2', 'c3') == (None, 'b', None)

assert pat.match('ac').group(1, 'b2', 3) == ('a', None, 'c')
print("ReTests::test_re_match: ok")
