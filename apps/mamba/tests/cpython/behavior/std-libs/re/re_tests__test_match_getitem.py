# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_match_getitem"
# subject = "cpython.test_re.ReTests.test_match_getitem"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_match_getitem
"""Auto-ported test: ReTests::test_match_getitem (CPython 3.12 oracle)."""


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
pat = re.compile('(?:(?P<a1>a)|(?P<b2>b))(?P<c3>c)?')
m = pat.match('a')

assert m['a1'] == 'a'

assert m['b2'] == None

assert m['c3'] == None

assert 'a1={a1} b2={b2} c3={c3}'.format_map(m) == 'a1=a b2=None c3=None'

assert m[0] == 'a'

assert m[1] == 'a'

assert m[2] == None

assert m[3] == None
try:
    m['X']
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    m[-1]
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    m[4]
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    m[0, 1]
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    m[0,]
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    m[0, 1]
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
try:
    'a1={a2}'.format_map(m)
    raise AssertionError('expected IndexError')
except IndexError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('no such group', str(_aR_e))
m = pat.match('ac')

assert m['a1'] == 'a'

assert m['b2'] == None

assert m['c3'] == 'c'

assert 'a1={a1} b2={b2} c3={c3}'.format_map(m) == 'a1=a b2=None c3=c'

assert m[0] == 'ac'

assert m[1] == 'a'

assert m[2] == None

assert m[3] == 'c'
try:
    m[0] = 1
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    len(m)
    raise AssertionError('expected TypeError')
except TypeError:
    pass
print("ReTests::test_match_getitem: ok")
