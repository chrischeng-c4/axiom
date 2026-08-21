# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_utf8_decode_invalid_sequences"
# subject = "cpython.test_unicode.UnicodeTest.test_utf8_decode_invalid_sequences"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_utf8_decode_invalid_sequences
"""Auto-ported test: UnicodeTest::test_utf8_decode_invalid_sequences (CPython 3.12 oracle)."""


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
continuation_bytes = [bytes([x]) for x in range(128, 192)]
invalid_2B_seq_start_bytes = [bytes([x]) for x in range(192, 194)]
invalid_4B_seq_start_bytes = [bytes([x]) for x in range(245, 248)]
invalid_start_bytes = continuation_bytes + invalid_2B_seq_start_bytes + invalid_4B_seq_start_bytes + [bytes([x]) for x in range(247, 256)]
for byte in invalid_start_bytes:

    try:
        byte.decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
for sb in invalid_2B_seq_start_bytes:
    for cb in continuation_bytes:

        try:
            (sb + cb).decode('utf-8')
            raise AssertionError('expected UnicodeDecodeError')
        except UnicodeDecodeError:
            pass
for sb in invalid_4B_seq_start_bytes:
    for cb1 in continuation_bytes[:3]:
        for cb3 in continuation_bytes[:3]:

            try:
                (sb + cb1 + b'\x80' + cb3).decode('utf-8')
                raise AssertionError('expected UnicodeDecodeError')
            except UnicodeDecodeError:
                pass
for cb in [bytes([x]) for x in range(128, 160)]:

    try:
        (b'\xe0' + cb + b'\x80').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass

    try:
        (b'\xe0' + cb + b'\xbf').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
for cb in [bytes([x]) for x in range(160, 192)]:

    try:
        (b'\xed' + cb + b'\x80').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass

    try:
        (b'\xed' + cb + b'\xbf').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
for cb in [bytes([x]) for x in range(128, 144)]:

    try:
        (b'\xf0' + cb + b'\x80\x80').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass

    try:
        (b'\xf0' + cb + b'\xbf\xbf').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
for cb in [bytes([x]) for x in range(144, 192)]:

    try:
        (b'\xf4' + cb + b'\x80\x80').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass

    try:
        (b'\xf4' + cb + b'\xbf\xbf').decode('utf-8')
        raise AssertionError('expected UnicodeDecodeError')
    except UnicodeDecodeError:
        pass
print("UnicodeTest::test_utf8_decode_invalid_sequences: ok")
