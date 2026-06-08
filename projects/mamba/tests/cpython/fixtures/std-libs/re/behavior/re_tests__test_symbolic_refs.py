# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_symbolic_refs"
# subject = "cpython.test_re.ReTests.test_symbolic_refs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_symbolic_refs
"""Auto-ported test: ReTests::test_symbolic_refs (CPython 3.12 oracle)."""


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

assert re.sub('(?P<a>x)|(?P<b>y)', '\\g<b>', 'xx') == ''

assert re.sub('(?P<a>x)|(?P<b>y)', '\\2', 'xx') == ''

assert re.sub(b'(?P<a1>x)', b'\\g<a1>', b'xx') == b'xx'

assert re.sub('(?P<µ>x)', '\\g<µ>', 'xx') == 'xx'

assert re.sub('(?P<𝔘𝔫𝔦𝔠𝔬𝔡𝔢>x)', '\\g<𝔘𝔫𝔦𝔠𝔬𝔡𝔢>', 'xx') == 'xx'
pat = '|'.join(('x(?P<a%d>%x)y' % (i, i) for i in range(1, 200 + 1)))

assert re.sub(pat, '\\g<200>', 'xc8yzxc8y') == 'c8zc8'
print("ReTests::test_symbolic_refs: ok")
