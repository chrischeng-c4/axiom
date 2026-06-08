# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "external_tests__test_re_tests"
# subject = "cpython.test_re.ExternalTests.test_re_tests"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ExternalTests::test_re_tests
"""Auto-ported test: ExternalTests::test_re_tests (CPython 3.12 oracle)."""


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
"""re_tests test suite"""
from test.re_tests import tests, FAIL, SYNTAX_ERROR
for t in tests:
    pattern = s = outcome = repl = expected = None
    if len(t) == 5:
        pattern, s, outcome, repl, expected = t
    elif len(t) == 3:
        pattern, s, outcome = t
    else:
        raise ValueError('Test tuples should have 3 or 5 fields', t)
    if outcome == SYNTAX_ERROR:
        try:
            re.compile(pattern)
            raise AssertionError('expected re.error')
        except re.error:
            pass
        continue
    obj = re.compile(pattern)
    result = obj.search(s)
    if outcome == FAIL:

        assert result is None
        continue

    assert result
    start, end = result.span(0)
    vardict = {'found': result.group(0), 'groups': result.group(), 'flags': result.re.flags}
    for i in range(1, 100):
        try:
            gi = result.group(i)
            if gi is None:
                gi = 'None'
        except IndexError:
            gi = 'Error'
        vardict['g%d' % i] = gi
    for i in result.re.groupindex.keys():
        try:
            gi = result.group(i)
            if gi is None:
                gi = 'None'
        except IndexError:
            gi = 'Error'
        vardict[i] = gi

    assert eval(repl, vardict) == expected
    try:
        bpat = bytes(pattern, 'ascii')
        bs = bytes(s, 'ascii')
    except UnicodeEncodeError:
        pass
    else:
        obj = re.compile(bpat)

        assert obj.search(bs)
        obj = re.compile(bpat, re.LOCALE)
        result = obj.search(bs)
        if result is None:
            print('=== Fails on locale-sensitive match', t)
    if pattern[:2] != '\\B' and pattern[-2:] != '\\B' and (result is not None):
        obj = re.compile(pattern)

        assert obj.search(s, start, end + 1)
    obj = re.compile(pattern, re.IGNORECASE)

    assert obj.search(s)
    obj = re.compile(pattern, re.UNICODE)

    assert obj.search(s)
print("ExternalTests::test_re_tests: ok")
