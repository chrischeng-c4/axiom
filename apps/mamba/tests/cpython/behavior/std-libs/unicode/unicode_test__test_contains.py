# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_contains"
# subject = "cpython.test_unicode.UnicodeTest.test_contains"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_contains
"""Auto-ported test: UnicodeTest::test_contains (CPython 3.12 oracle)."""


import _string
import codecs
import datetime
import itertools
import operator
import pickle
import struct
import sys
import textwrap
import unicodedata
import unittest
import warnings
from test.support import warnings_helper
from test import support, string_tests
from test.support.script_helper import assert_python_failure


' Test script for the Unicode implementation.\n\nWritten by Marc-Andre Lemburg (mal@lemburg.com).\n\n(c) Copyright CNRI, All Rights Reserved. NO WARRANTY.\n\n'

try:
    import _testcapi
except ImportError:
    _testcapi = None

def search_function(encoding):

    def decode1(input, errors='strict'):
        return 42

    def encode1(input, errors='strict'):
        return 42

    def encode2(input, errors='strict'):
        return (42, 42)

    def decode2(input, errors='strict'):
        return (42, 42)
    if encoding == 'test.unicode1':
        return (encode1, decode1, None, None)
    elif encoding == 'test.unicode2':
        return (encode2, decode2, None, None)
    else:
        return None

def duplicate_string(text):
    """
    Try to get a fresh clone of the specified text:
    new object with a reference count of 1.

    This is a best-effort: latin1 single letters and the empty
    string ('') are singletons and cannot be cloned.
    """
    return text.encode().decode()

class StrSubclass(str):
    pass

class OtherStrSubclass(str):
    pass

class WithStr:

    def __init__(self, value):
        self.value = value

    def __str__(self):
        return self.value

class WithRepr:

    def __init__(self, value):
        self.value = value

    def __repr__(self):
        return self.value


# --- test body ---
type2test = str

def assertCorrectUTF8Decoding(seq, res, err):
    """
        Check that an invalid UTF-8 sequence raises a UnicodeDecodeError when
        'strict' is used, returns res when 'replace' is used, and that doesn't
        return anything when 'ignore' is used.
        """
    try:
        seq.decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError as _aR_e:
        import types as _types_aR
        cm = _types_aR.SimpleNamespace(exception=_aR_e)
    exc = cm.exception

    assert err in str(exc)

    assert seq.decode('utf-8', 'replace') == res

    assert (b'aaaa' + seq + b'bbbb').decode('utf-8', 'replace') == 'aaaa' + res + 'bbbb'
    res = res.replace('�', '')

    assert seq.decode('utf-8', 'ignore') == res

    assert (b'aaaa' + seq + b'bbbb').decode('utf-8', 'ignore') == 'aaaa' + res + 'bbbb'

def assertTypedEqual(actual, expected):

    assert type(actual) is type(expected)

    assert actual == expected

def checkequalnofix(result, object, methodname, *args):
    method = getattr(object, methodname)
    realresult = method(*args)

    assert realresult == result

    assert type(realresult) is type(result)
    if realresult is object:

        class usub(str):

            def __repr__(self):
                return 'usub(%r)' % str.__repr__(self)
        object = usub(object)
        method = getattr(object, methodname)
        realresult = method(*args)

        assert realresult == result

        assert object is not realresult
codecs.register(search_function)
pass

assert 'a' in 'abdb'

assert 'a' in 'bdab'

assert 'a' in 'bdaba'

assert 'a' in 'bdba'

assert 'a' not in 'bdb'

assert 'a' in 'bdba'

assert 'a' in ('a', 1, None)

assert 'a' in (1, None, 'a')

assert 'a' in ('a', 1, None)

assert 'a' in (1, None, 'a')

assert 'a' not in ('x', 1, 'y')

assert 'a' not in ('x', 1, None)

assert 'abcd' not in 'abcxxxx'

assert 'ab' in 'abcd'

assert 'ab' in 'abc'

assert 'ab' in (1, None, 'ab')

assert '' in 'abc'

assert '' in ''

assert '' in 'abc'

assert '\x00' not in 'abc'

assert '\x00' in '\x00abc'

assert '\x00' in 'abc\x00'

assert 'a' in '\x00abc'

assert 'asdf' in 'asdf'

assert 'asdf' not in 'asd'

assert 'asdf' not in ''

try:
    'abc'.__contains__()
    raise AssertionError('expected TypeError')
except TypeError:
    pass
for fill in ('a', 'Ā', '𐌀'):
    fill *= 9
    for delim in ('c', 'Ă', '𐌂'):

        assert delim not in fill

        assert delim in fill + delim

        assert delim * 2 not in fill

        assert delim * 2 in fill + delim * 2
print("UnicodeTest::test_contains: ok")
