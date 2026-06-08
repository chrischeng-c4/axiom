# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_error"
# subject = "cpython.test_re.ReTests.test_error"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_error
"""Auto-ported test: ReTests::test_error (CPython 3.12 oracle)."""


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
try:
    re.compile('(€))')
    raise AssertionError('expected re.error')
except re.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
err = cm.exception

assert isinstance(err.pattern, str)

assert err.pattern == '(€))'

assert err.pos == 3

assert err.lineno == 1

assert err.colno == 4

assert err.msg in str(err)

assert ' at position 3' in str(err)

assert ' at position 3' not in err.msg
try:
    re.compile(b'(\xa4))')
    raise AssertionError('expected re.error')
except re.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
err = cm.exception

assert isinstance(err.pattern, bytes)

assert err.pattern == b'(\xa4))'

assert err.pos == 3
try:
    re.compile('\n                (\n                    abc\n                )\n                )\n                (\n                ', re.VERBOSE)
    raise AssertionError('expected re.error')
except re.error as _aR_e:
    import types as _types_aR
    cm = _types_aR.SimpleNamespace(exception=_aR_e)
err = cm.exception

assert err.pos == 77

assert err.lineno == 5

assert err.colno == 17

assert err.msg in str(err)

assert ' at position 77' in str(err)

assert '(line 5, column 17)' in str(err)
print("ReTests::test_error: ok")
