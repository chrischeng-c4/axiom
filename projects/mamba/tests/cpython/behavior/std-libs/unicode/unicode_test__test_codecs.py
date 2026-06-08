# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_codecs"
# subject = "cpython.test_unicode.UnicodeTest.test_codecs"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_codecs
"""Auto-ported test: UnicodeTest::test_codecs (CPython 3.12 oracle)."""


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

assert 'hello'.encode('ascii') == b'hello'

assert 'hello'.encode('utf-7') == b'hello'

assert 'hello'.encode('utf-8') == b'hello'

assert 'hello'.encode('utf-8') == b'hello'

assert 'hello'.encode('utf-16-le') == b'h\x00e\x00l\x00l\x00o\x00'

assert 'hello'.encode('utf-16-be') == b'\x00h\x00e\x00l\x00l\x00o'

assert 'hello'.encode('latin-1') == b'hello'

assert '☃'.encode() == b'\xe2\x98\x83'
for c in range(1024):
    u = chr(c)
    for encoding in ('utf-7', 'utf-8', 'utf-16', 'utf-16-le', 'utf-16-be', 'raw_unicode_escape', 'unicode_escape'):

        assert str(u.encode(encoding), encoding) == u
for c in range(256):
    u = chr(c)
    for encoding in ('latin-1',):

        assert str(u.encode(encoding), encoding) == u
for c in range(128):
    u = chr(c)
    for encoding in ('ascii',):

        assert str(u.encode(encoding), encoding) == u
with warnings.catch_warnings():
    u = '𐀁𠀂𰀃\U00040004\U00050005'
    for encoding in ('utf-8', 'utf-16', 'utf-16-le', 'utf-16-be', 'raw_unicode_escape', 'unicode_escape'):

        assert str(u.encode(encoding), encoding) == u
u = ''.join(map(chr, list(range(0, 55296)) + list(range(57344, 1114112))))
for encoding in ('utf-8',):

    assert str(u.encode(encoding), encoding) == u
print("UnicodeTest::test_codecs: ok")
