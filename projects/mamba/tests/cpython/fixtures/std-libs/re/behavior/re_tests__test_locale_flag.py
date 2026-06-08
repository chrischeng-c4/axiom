# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_locale_flag"
# subject = "cpython.test_re.ReTests.test_locale_flag"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_locale_flag
"""Auto-ported test: ReTests::test_locale_flag (CPython 3.12 oracle)."""


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
enc = locale.getpreferredencoding()
for i in range(128, 256):
    try:
        c = bytes([i]).decode(enc)
        sletter = c.lower()
        if sletter == c:
            continue
        bletter = sletter.encode(enc)
        if len(bletter) != 1:
            continue
        if bletter.decode(enc) != sletter:
            continue
        bpat = re.escape(bytes([i]))
        break
    except (UnicodeError, TypeError):
        pass
else:
    bletter = None
    bpat = b'A'
pat = re.compile(bpat, re.LOCALE | re.IGNORECASE)
if bletter:

    assert pat.match(bletter)
pat = re.compile(b'(?L)' + bpat, re.IGNORECASE)
if bletter:

    assert pat.match(bletter)
pat = re.compile(bpat, re.IGNORECASE)
if bletter:

    assert pat.match(bletter) is None
pat = re.compile(b'\\w', re.LOCALE)
if bletter:

    assert pat.match(bletter)
pat = re.compile(b'(?L)\\w')
if bletter:

    assert pat.match(bletter)
pat = re.compile(b'\\w')
if bletter:

    assert pat.match(bletter) is None

try:
    re.compile('', re.LOCALE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile('(?L)')
    raise AssertionError('expected re.error')
except re.error:
    pass

try:
    re.compile(b'', re.LOCALE | re.ASCII)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile(b'(?L)', re.ASCII)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile(b'(?a)', re.LOCALE)
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    re.compile(b'(?aL)')
    raise AssertionError('expected re.error')
except re.error:
    pass
print("ReTests::test_locale_flag: ok")
