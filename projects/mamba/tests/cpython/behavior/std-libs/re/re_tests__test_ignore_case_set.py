# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_ignore_case_set"
# subject = "cpython.test_re.ReTests.test_ignore_case_set"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_ignore_case_set
"""Auto-ported test: ReTests::test_ignore_case_set (CPython 3.12 oracle)."""


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

assert re.match('[19A]', 'A', re.I)

assert re.match('[19a]', 'a', re.I)

assert re.match('[19a]', 'A', re.I)

assert re.match('[19A]', 'a', re.I)

assert re.match(b'[19A]', b'A', re.I)

assert re.match(b'[19a]', b'a', re.I)

assert re.match(b'[19a]', b'A', re.I)

assert re.match(b'[19A]', b'a', re.I)

assert re.match('[19\\xc7]', 'Ç', re.I)

assert re.match('[19\\xc7]', 'ç', re.I)

assert re.match('[19\\xe7]', 'Ç', re.I)

assert re.match('[19\\xe7]', 'ç', re.I)

assert re.match('[19\\u0400]', 'Ѐ', re.I)

assert re.match('[19\\u0400]', 'ѐ', re.I)

assert re.match('[19\\u0450]', 'Ѐ', re.I)

assert re.match('[19\\u0450]', 'ѐ', re.I)

assert re.match('[19\\U00010400]', '𐐀', re.I)

assert re.match('[19\\U00010400]', '𐐨', re.I)

assert re.match('[19\\U00010428]', '𐐀', re.I)

assert re.match('[19\\U00010428]', '𐐨', re.I)

assert re.match(b'[19A]', b'A', re.I)

assert re.match(b'[19a]', b'a', re.I)

assert re.match(b'[19a]', b'A', re.I)

assert re.match(b'[19A]', b'a', re.I)

assert re.match('[19A]', 'A', re.I | re.A)

assert re.match('[19a]', 'a', re.I | re.A)

assert re.match('[19a]', 'A', re.I | re.A)

assert re.match('[19A]', 'a', re.I | re.A)

assert re.match('[19\\xc7]', 'Ç', re.I | re.A)

assert re.match('[19\\xc7]', 'ç', re.I | re.A) is None

assert re.match('[19\\xe7]', 'Ç', re.I | re.A) is None

assert re.match('[19\\xe7]', 'ç', re.I | re.A)

assert re.match('[19\\u0400]', 'Ѐ', re.I | re.A)

assert re.match('[19\\u0400]', 'ѐ', re.I | re.A) is None

assert re.match('[19\\u0450]', 'Ѐ', re.I | re.A) is None

assert re.match('[19\\u0450]', 'ѐ', re.I | re.A)

assert re.match('[19\\U00010400]', '𐐀', re.I | re.A)

assert re.match('[19\\U00010400]', '𐐨', re.I | re.A) is None

assert re.match('[19\\U00010428]', '𐐀', re.I | re.A) is None

assert re.match('[19\\U00010428]', '𐐨', re.I | re.A)
assert 'K'.lower() == 'K'.lower() == 'k'

assert re.match('[19K]', 'K', re.I)

assert re.match('[19k]', 'K', re.I)

assert re.match('[19\\u212a]', 'K', re.I)

assert re.match('[19\\u212a]', 'k', re.I)
assert 's'.upper() == 'ſ'.upper() == 'S'

assert re.match('[19S]', 'ſ', re.I)

assert re.match('[19s]', 'ſ', re.I)

assert re.match('[19\\u017f]', 'S', re.I)

assert re.match('[19\\u017f]', 's', re.I)
assert 'в'.upper() == 'ᲀ'.upper() == 'В'

assert re.match('[19\\u0412]', 'в', re.I)

assert re.match('[19\\u0412]', 'ᲀ', re.I)

assert re.match('[19\\u0432]', 'В', re.I)

assert re.match('[19\\u0432]', 'ᲀ', re.I)

assert re.match('[19\\u1c80]', 'В', re.I)

assert re.match('[19\\u1c80]', 'в', re.I)
assert 'ﬅ'.upper() == 'ﬆ'.upper() == 'ST'

assert re.match('[19\\ufb05]', 'ﬆ', re.I)

assert re.match('[19\\ufb06]', 'ﬅ', re.I)
print("ReTests::test_ignore_case_set: ok")
