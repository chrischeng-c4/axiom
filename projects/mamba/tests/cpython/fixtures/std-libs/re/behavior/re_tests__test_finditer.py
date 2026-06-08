# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_finditer"
# subject = "cpython.test_re.ReTests.test_finditer"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_finditer
"""Auto-ported test: ReTests::test_finditer (CPython 3.12 oracle)."""


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
iter = re.finditer(':+', 'a:b::c:::d')

assert [item.group(0) for item in iter] == [':', '::', ':::']
pat = re.compile(':+')
iter = pat.finditer('a:b::c:::d', 1, 10)

assert [item.group(0) for item in iter] == [':', '::', ':::']
pat = re.compile(':+')
iter = pat.finditer('a:b::c:::d', pos=1, endpos=10)

assert [item.group(0) for item in iter] == [':', '::', ':::']
pat = re.compile(':+')
iter = pat.finditer('a:b::c:::d', endpos=10, pos=1)

assert [item.group(0) for item in iter] == [':', '::', ':::']
pat = re.compile(':+')
iter = pat.finditer('a:b::c:::d', pos=3, endpos=8)

assert [item.group(0) for item in iter] == ['::', '::']
print("ReTests::test_finditer: ok")
