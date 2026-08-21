# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_bug_34294"
# subject = "cpython.test_re.ReTests.test_bug_34294"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_bug_34294
"""Auto-ported test: ReTests::test_bug_34294 (CPython 3.12 oracle)."""


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
s = 'a\tx'
p = '\\b(?=(\\t)|(x))x'

assert re.search(p, s).groups() == (None, 'x')
s = 'ab'
p = '(?=(.)(.)?)'

assert re.findall(p, s) == [('a', 'b'), ('b', '')]

assert [m.groups() for m in re.finditer(p, s)] == [('a', 'b'), ('b', None)]
p = '(?=<(?P<tag>\\w+)/?>(?:(?P<text>.+?)</(?P=tag)>)?)'
s = '<test><foo2/></test>'

assert re.findall(p, s) == [('test', '<foo2/>'), ('foo2', '')]

assert [m.groupdict() for m in re.finditer(p, s)] == [{'tag': 'test', 'text': '<foo2/>'}, {'tag': 'foo2', 'text': None}]
s = '<test>Hello</test><foo/>'

assert [m.groupdict() for m in re.finditer(p, s)] == [{'tag': 'test', 'text': 'Hello'}, {'tag': 'foo', 'text': None}]
s = '<test>Hello</test><foo/><foo/>'

assert [m.groupdict() for m in re.finditer(p, s)] == [{'tag': 'test', 'text': 'Hello'}, {'tag': 'foo', 'text': None}, {'tag': 'foo', 'text': None}]
print("ReTests::test_bug_34294: ok")
