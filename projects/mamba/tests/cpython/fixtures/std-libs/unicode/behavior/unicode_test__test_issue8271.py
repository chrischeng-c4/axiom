# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_issue8271"
# subject = "cpython.test_unicode.UnicodeTest.test_issue8271"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_issue8271
"""Auto-ported test: UnicodeTest::test_issue8271 (CPython 3.12 oracle)."""


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
FFFD = '�'
sequences = [(b'\x80', FFFD), (b'\x80\x80', FFFD * 2), (b'\xc0', FFFD), (b'\xc0\xc0', FFFD * 2), (b'\xc1', FFFD), (b'\xc1\xc0', FFFD * 2), (b'\xc0\xc1', FFFD * 2), (b'\xc2', FFFD), (b'\xc2\xc2', FFFD * 2), (b'\xc2\xc2\xc2', FFFD * 3), (b'\xc2A', FFFD + 'A'), (b'\xe1', FFFD), (b'\xe1\xe1', FFFD * 2), (b'\xe1\xe1\xe1', FFFD * 3), (b'\xe1\xe1\xe1\xe1', FFFD * 4), (b'\xe1\x80', FFFD), (b'\xe1A', FFFD + 'A'), (b'\xe1A\x80', FFFD + 'A' + FFFD), (b'\xe1AA', FFFD + 'AA'), (b'\xe1\x80A', FFFD + 'A'), (b'\xe1\x80\xe1A', FFFD * 2 + 'A'), (b'\xe1A\xe1\x80', FFFD + 'A' + FFFD), (b'\xf1', FFFD), (b'\xf1\xf1', FFFD * 2), (b'\xf1\xf1\xf1', FFFD * 3), (b'\xf1\xf1\xf1\xf1', FFFD * 4), (b'\xf1\xf1\xf1\xf1\xf1', FFFD * 5), (b'\xf1\x80', FFFD), (b'\xf1\x80\x80', FFFD), (b'\xf1\x80A', FFFD + 'A'), (b'\xf1\x80AA', FFFD + 'AA'), (b'\xf1\x80\x80A', FFFD + 'A'), (b'\xf1A\x80', FFFD + 'A' + FFFD), (b'\xf1A\x80\x80', FFFD + 'A' + FFFD * 2), (b'\xf1A\x80A', FFFD + 'A' + FFFD + 'A'), (b'\xf1AA\x80', FFFD + 'AA' + FFFD), (b'\xf1A\xf1\x80', FFFD + 'A' + FFFD), (b'\xf1A\x80\xf1', FFFD + 'A' + FFFD * 2), (b'\xf1\xf1\x80A', FFFD * 2 + 'A'), (b'\xf1A\xf1\xf1', FFFD + 'A' + FFFD * 2), (b'\xf5', FFFD), (b'\xf5\xf5', FFFD * 2), (b'\xf5\x80', FFFD * 2), (b'\xf5\x80\x80', FFFD * 3), (b'\xf5\x80\x80\x80', FFFD * 4), (b'\xf5\x80A', FFFD * 2 + 'A'), (b'\xf5\x80A\xf5', FFFD * 2 + 'A' + FFFD), (b'\xf5A\x80\x80A', FFFD + 'A' + FFFD * 2 + 'A'), (b'\xf8', FFFD), (b'\xf8\xf8', FFFD * 2), (b'\xf8\x80', FFFD * 2), (b'\xf8\x80A', FFFD * 2 + 'A'), (b'\xf8\x80\x80\x80\x80', FFFD * 5), (b'\xfc', FFFD), (b'\xfc\xfc', FFFD * 2), (b'\xfc\x80\x80', FFFD * 3), (b'\xfc\x80\x80\x80\x80\x80', FFFD * 6), (b'\xfe', FFFD), (b'\xfe\x80\x80', FFFD * 3), (b'\xf1\x80ABC', '�ABC'), (b'\xf1\x80\xffBC', '��BC'), (b'\xf1\x80\xc2\x81C', '�\x81C'), (b'a\xf1\x80\x80\xe1\x80\xc2b\x80c\x80\xbfd', 'a���b�c��d')]
for n, (seq, res) in enumerate(sequences):

    try:
        seq.decode('utf-8', 'strict')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass

    assert seq.decode('utf-8', 'replace') == res

    assert (seq + b'b').decode('utf-8', 'replace') == res + 'b'

    assert seq.decode('utf-8', 'ignore') == res.replace('�', '')
print("UnicodeTest::test_issue8271: ok")
