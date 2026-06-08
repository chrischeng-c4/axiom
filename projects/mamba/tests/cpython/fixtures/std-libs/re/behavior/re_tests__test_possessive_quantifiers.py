# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_possessive_quantifiers"
# subject = "cpython.test_re.ReTests.test_possessive_quantifiers"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_possessive_quantifiers
"""Auto-ported test: ReTests::test_possessive_quantifiers (CPython 3.12 oracle)."""


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
'Test Possessive Quantifiers\n        Test quantifiers of the form @+ for some repetition operator @,\n        e.g. x{3,5}+ meaning match from 3 to 5 greadily and proceed\n        without creating a stack frame for rolling the stack back and\n        trying 1 or more fewer matches.'

assert re.match('e*+e', 'eeee') is None

assert re.match('e++a', 'eeea').group(0) == 'eeea'

assert re.match('e?+a', 'ea').group(0) == 'ea'

assert re.match('e{2,4}+a', 'eeea').group(0) == 'eeea'

assert re.match('(.)++.', 'ee') is None

assert re.match('(ae)*+a', 'aea').groups() == ('ae',)

assert re.match('([ae][ae])?+a', 'aea').groups() == ('ae',)

assert re.match('(e?){2,4}+a', 'eeea').groups() == ('',)

assert re.match('()*+a', 'a').groups() == ('',)

assert re.search('x*+', 'axx').span() == (0, 0)

assert re.search('x++', 'axx').span() == (1, 3)

assert re.match('a*+', 'xxx').span() == (0, 0)

assert re.match('x*+', 'xxxa').span() == (0, 3)

assert re.match('a++', 'xxx') is None

assert re.match('^(\\w){1}+$', 'abc') is None

assert re.match('^(\\w){1,2}+$', 'abc') is None

assert re.match('^(\\w){3}+$', 'abc').group(1) == 'c'

assert re.match('^(\\w){1,3}+$', 'abc').group(1) == 'c'

assert re.match('^(\\w){1,4}+$', 'abc').group(1) == 'c'

assert re.match('^x{1}+$', 'xxx') is None

assert re.match('^x{1,2}+$', 'xxx') is None

assert re.match('^x{3}+$', 'xxx')

assert re.match('^x{1,3}+$', 'xxx')

assert re.match('^x{1,4}+$', 'xxx')

assert re.match('^x{}+$', 'xxx') is None

assert re.match('^x{}+$', 'x{}')
print("ReTests::test_possessive_quantifiers: ok")
