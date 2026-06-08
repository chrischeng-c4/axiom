# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_ignore_case_range"
# subject = "cpython.test_re.ReTests.test_ignore_case_range"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_ignore_case_range
"""Auto-ported test: ReTests::test_ignore_case_range (CPython 3.12 oracle)."""


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

assert re.match('[9-a]', '_', re.I)

assert re.match('[9-A]', '_', re.I) is None

assert re.match(b'[9-a]', b'_', re.I)

assert re.match(b'[9-A]', b'_', re.I) is None

assert re.match('[\\xc0-\\xde]', '×', re.I)

assert re.match('[\\xc0-\\xde]', 'ç', re.I)

assert re.match('[\\xc0-\\xde]', '÷', re.I) is None

assert re.match('[\\xe0-\\xfe]', '÷', re.I)

assert re.match('[\\xe0-\\xfe]', 'Ç', re.I)

assert re.match('[\\xe0-\\xfe]', '×', re.I) is None

assert re.match('[\\u0430-\\u045f]', 'ѐ', re.I)

assert re.match('[\\u0430-\\u045f]', 'Ѐ', re.I)

assert re.match('[\\u0400-\\u042f]', 'ѐ', re.I)

assert re.match('[\\u0400-\\u042f]', 'Ѐ', re.I)

assert re.match('[\\U00010428-\\U0001044f]', '𐐨', re.I)

assert re.match('[\\U00010428-\\U0001044f]', '𐐀', re.I)

assert re.match('[\\U00010400-\\U00010427]', '𐐨', re.I)

assert re.match('[\\U00010400-\\U00010427]', '𐐀', re.I)

assert re.match('[\\xc0-\\xde]', '×', re.I | re.A)

assert re.match('[\\xc0-\\xde]', 'ç', re.I | re.A) is None

assert re.match('[\\xe0-\\xfe]', '÷', re.I | re.A)

assert re.match('[\\xe0-\\xfe]', 'Ç', re.I | re.A) is None

assert re.match('[\\u0430-\\u045f]', 'ѐ', re.I | re.A)

assert re.match('[\\u0430-\\u045f]', 'Ѐ', re.I | re.A) is None

assert re.match('[\\u0400-\\u042f]', 'ѐ', re.I | re.A) is None

assert re.match('[\\u0400-\\u042f]', 'Ѐ', re.I | re.A)

assert re.match('[\\U00010428-\\U0001044f]', '𐐨', re.I | re.A)

assert re.match('[\\U00010428-\\U0001044f]', '𐐀', re.I | re.A) is None

assert re.match('[\\U00010400-\\U00010427]', '𐐨', re.I | re.A) is None

assert re.match('[\\U00010400-\\U00010427]', '𐐀', re.I | re.A)

assert re.match('[N-\\x7f]', 'A', re.I | re.A)

assert re.match('[n-\\x7f]', 'Z', re.I | re.A)

assert re.match('[N-\\uffff]', 'A', re.I | re.A)

assert re.match('[n-\\uffff]', 'Z', re.I | re.A)

assert re.match('[N-\\U00010000]', 'A', re.I | re.A)

assert re.match('[n-\\U00010000]', 'Z', re.I | re.A)
assert 'K'.lower() == 'K'.lower() == 'k'

assert re.match('[J-M]', 'K', re.I)

assert re.match('[j-m]', 'K', re.I)

assert re.match('[\\u2129-\\u212b]', 'K', re.I)

assert re.match('[\\u2129-\\u212b]', 'k', re.I)
assert 's'.upper() == 'ſ'.upper() == 'S'

assert re.match('[R-T]', 'ſ', re.I)

assert re.match('[r-t]', 'ſ', re.I)

assert re.match('[\\u017e-\\u0180]', 'S', re.I)

assert re.match('[\\u017e-\\u0180]', 's', re.I)
assert 'в'.upper() == 'ᲀ'.upper() == 'В'

assert re.match('[\\u0411-\\u0413]', 'в', re.I)

assert re.match('[\\u0411-\\u0413]', 'ᲀ', re.I)

assert re.match('[\\u0431-\\u0433]', 'В', re.I)

assert re.match('[\\u0431-\\u0433]', 'ᲀ', re.I)

assert re.match('[\\u1c80-\\u1c82]', 'В', re.I)

assert re.match('[\\u1c80-\\u1c82]', 'в', re.I)
assert 'ﬅ'.upper() == 'ﬆ'.upper() == 'ST'

assert re.match('[\\ufb04-\\ufb05]', 'ﬆ', re.I)

assert re.match('[\\ufb06-\\ufb07]', 'ﬅ', re.I)
print("ReTests::test_ignore_case_range: ok")
