# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_comparison"
# subject = "cpython.test_unicode.UnicodeTest.test_comparison"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_comparison
"""Auto-ported test: UnicodeTest::test_comparison (CPython 3.12 oracle)."""


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

assert 'abc' == 'abc'

assert 'abcd' > 'abc'

assert 'abc' < 'abcd'
if 0:

    assert 'a' < '€'

    assert 'a' < '\ud800\udc02'

    def test_lecmp(s, s2):
        self.assertTrue(s < s2)

    def test_fixup(s):
        s2 = '\ud800\udc01'
        test_lecmp(s, s2)
        s2 = '\ud900\udc01'
        test_lecmp(s, s2)
        s2 = '\uda00\udc01'
        test_lecmp(s, s2)
        s2 = '\udb00\udc01'
        test_lecmp(s, s2)
        s2 = '\ud800\udd01'
        test_lecmp(s, s2)
        s2 = '\ud900\udd01'
        test_lecmp(s, s2)
        s2 = '\uda00\udd01'
        test_lecmp(s, s2)
        s2 = '\udb00\udd01'
        test_lecmp(s, s2)
        s2 = '\ud800\ude01'
        test_lecmp(s, s2)
        s2 = '\ud900\ude01'
        test_lecmp(s, s2)
        s2 = '\uda00\ude01'
        test_lecmp(s, s2)
        s2 = '\udb00\ude01'
        test_lecmp(s, s2)
        s2 = '\ud800\udfff'
        test_lecmp(s, s2)
        s2 = '\ud900\udfff'
        test_lecmp(s, s2)
        s2 = '\uda00\udfff'
        test_lecmp(s, s2)
        s2 = '\udb00\udfff'
        test_lecmp(s, s2)
        test_fixup('\ue000')
        test_fixup('｡')

assert '\ud800\udc02' < '\ud84d\udc56'
print("UnicodeTest::test_comparison: ok")
