# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "re"
# dimension = "behavior"
# case = "re_tests__test_word_boundaries"
# subject = "cpython.test_re.ReTests.test_word_boundaries"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_re.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_re.py::ReTests::test_word_boundaries
"""Auto-ported test: ReTests::test_word_boundaries (CPython 3.12 oracle)."""


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

assert re.search('\\b(abc)\\b', 'abc').group(1) == 'abc'

assert re.search('\\b(abc)\\b', 'abc', re.ASCII).group(1) == 'abc'

assert re.search(b'\\b(abc)\\b', b'abc').group(1) == b'abc'

assert re.search(b'\\b(abc)\\b', b'abc', re.LOCALE).group(1) == b'abc'

assert re.search('\\b(ьюя)\\b', 'ьюя').group(1) == 'ьюя'

assert re.search('\\b(ьюя)\\b', 'ьюя', re.ASCII) is None

assert re.match('.\\b', 'a=')

assert re.match('.\\b', 'a=', re.ASCII)

assert re.match(b'.\\b', b'a=')

assert re.match(b'.\\b', b'a=', re.LOCALE)

assert re.match('.\\b', 'я=')

assert re.match('.\\b', 'я=', re.ASCII) is None

assert re.match('.\\b', '=a')

assert re.match('.\\b', '=a', re.ASCII)

assert re.match(b'.\\b', b'=a')

assert re.match(b'.\\b', b'=a', re.LOCALE)

assert re.match('.\\b', '=я')

assert re.match('.\\b', '=я', re.ASCII) is None

assert re.match('.\\b', 'ab') is None

assert re.match('.\\b', 'ab', re.ASCII) is None

assert re.match(b'.\\b', b'ab') is None

assert re.match(b'.\\b', b'ab', re.LOCALE) is None

assert re.match('.\\b', 'юя') is None

assert re.match('.\\b', 'юя', re.ASCII) is None

assert re.match('.\\b', '=-') is None

assert re.match('.\\b', '=-', re.ASCII) is None

assert re.match(b'.\\b', b'=-') is None

assert re.match(b'.\\b', b'=-', re.LOCALE) is None

assert re.match('.\\B', 'a=') is None

assert re.match('.\\B', 'a=', re.ASCII) is None

assert re.match(b'.\\B', b'a=') is None

assert re.match(b'.\\B', b'a=', re.LOCALE) is None

assert re.match('.\\B', 'я=') is None

assert re.match('.\\B', 'я=', re.ASCII)

assert re.match('.\\B', '=a') is None

assert re.match('.\\B', '=a', re.ASCII) is None

assert re.match(b'.\\B', b'=a') is None

assert re.match(b'.\\B', b'=a', re.LOCALE) is None

assert re.match('.\\B', '=я') is None

assert re.match('.\\B', '=я', re.ASCII)

assert re.match('.\\B', 'ab')

assert re.match('.\\B', 'ab', re.ASCII)

assert re.match(b'.\\B', b'ab')

assert re.match(b'.\\B', b'ab', re.LOCALE)

assert re.match('.\\B', 'юя')

assert re.match('.\\B', 'юя', re.ASCII)

assert re.match('.\\B', '=-')

assert re.match('.\\B', '=-', re.ASCII)

assert re.match(b'.\\B', b'=-')

assert re.match(b'.\\B', b'=-', re.LOCALE)

assert re.match('\\b', 'abc')

assert re.match('\\b', 'abc', re.ASCII)

assert re.match(b'\\b', b'abc')

assert re.match(b'\\b', b'abc', re.LOCALE)

assert re.match('\\b', 'ьюя')

assert re.match('\\b', 'ьюя', re.ASCII) is None

assert re.fullmatch('.+\\b', 'abc')

assert re.fullmatch('.+\\b', 'abc', re.ASCII)

assert re.fullmatch(b'.+\\b', b'abc')

assert re.fullmatch(b'.+\\b', b'abc', re.LOCALE)

assert re.fullmatch('.+\\b', 'ьюя')

assert re.search('\\b', 'ьюя', re.ASCII) is None

assert re.search('\\B', 'abc').span() == (1, 1)

assert re.search('\\B', 'abc', re.ASCII).span() == (1, 1)

assert re.search(b'\\B', b'abc').span() == (1, 1)

assert re.search(b'\\B', b'abc', re.LOCALE).span() == (1, 1)

assert re.search('\\B', 'ьюя').span() == (1, 1)

assert re.search('\\B', 'ьюя', re.ASCII).span() == (0, 0)

assert re.match('\\B', 'abc') is None

assert re.match('\\B', 'abc', re.ASCII) is None

assert re.match(b'\\B', b'abc') is None

assert re.match(b'\\B', b'abc', re.LOCALE) is None

assert re.match('\\B', 'ьюя') is None

assert re.match('\\B', 'ьюя', re.ASCII)

assert re.fullmatch('.+\\B', 'abc') is None

assert re.fullmatch('.+\\B', 'abc', re.ASCII) is None

assert re.fullmatch(b'.+\\B', b'abc') is None

assert re.fullmatch(b'.+\\B', b'abc', re.LOCALE) is None

assert re.fullmatch('.+\\B', 'ьюя') is None

assert re.fullmatch('.+\\B', 'ьюя', re.ASCII)

assert re.search('\\b', '') is None

assert re.search('\\b', '', re.ASCII) is None

assert re.search(b'\\b', b'') is None

assert re.search(b'\\b', b'', re.LOCALE) is None

assert re.search('\\B', '') is None

assert re.search('\\B', '', re.ASCII) is None

assert re.search(b'\\B', b'') is None

assert re.search(b'\\B', b'', re.LOCALE) is None

assert len(re.findall('\\b', 'a')) == 2

assert len(re.findall('\\b', 'a', re.ASCII)) == 2

assert len(re.findall(b'\\b', b'a')) == 2

assert len(re.findall(b'\\b', b'a', re.LOCALE)) == 2

assert len(re.findall('\\B', 'a')) == 0

assert len(re.findall('\\B', 'a', re.ASCII)) == 0

assert len(re.findall(b'\\B', b'a')) == 0

assert len(re.findall(b'\\B', b'a', re.LOCALE)) == 0

assert len(re.findall('\\b', ' ')) == 0

assert len(re.findall('\\b', ' ', re.ASCII)) == 0

assert len(re.findall(b'\\b', b' ')) == 0

assert len(re.findall(b'\\b', b' ', re.LOCALE)) == 0

assert len(re.findall('\\b', '   ')) == 0

assert len(re.findall('\\b', '   ', re.ASCII)) == 0

assert len(re.findall(b'\\b', b'   ')) == 0

assert len(re.findall(b'\\b', b'   ', re.LOCALE)) == 0

assert len(re.findall('\\B', ' ')) == 2

assert len(re.findall('\\B', ' ', re.ASCII)) == 2

assert len(re.findall(b'\\B', b' ')) == 2

assert len(re.findall(b'\\B', b' ', re.LOCALE)) == 2
print("ReTests::test_word_boundaries: ok")
