# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_codecs_utf7"
# subject = "cpython.test_unicode.UnicodeTest.test_codecs_utf7"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_codecs_utf7
"""Auto-ported test: UnicodeTest::test_codecs_utf7 (CPython 3.12 oracle)."""


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
utfTests = [('A≢Α.', b'A+ImIDkQ.'), ('Hi Mom -☺-!', b'Hi Mom -+Jjo--!'), ('日本語', b'+ZeVnLIqe-'), ('Item 3 is £1.', b'Item 3 is +AKM-1.'), ('+', b'+-'), ('+-', b'+--'), ('+?', b'+-?'), ('\\?', b'+AFw?'), ('+?', b'+-?'), ('\\\\?', b'+AFwAXA?'), ('\\\\\\?', b'+AFwAXABc?'), ('++--', b'+-+---'), ('\U000abcde', b'+2m/c3g-'), ('/', b'/')]
for x, y in utfTests:

    assert x.encode('utf-7') == y

assert '\ud801'.encode('utf-7') == b'+2AE-'

assert '\ud801x'.encode('utf-7') == b'+2AE-x'

assert '\udc01'.encode('utf-7') == b'+3AE-'

assert '\udc01x'.encode('utf-7') == b'+3AE-x'

assert b'+2AE-'.decode('utf-7') == '\ud801'

assert b'+2AE-x'.decode('utf-7') == '\ud801x'

assert b'+3AE-'.decode('utf-7') == '\udc01'

assert b'+3AE-x'.decode('utf-7') == '\udc01x'

assert '\ud801\U000abcde'.encode('utf-7') == b'+2AHab9ze-'

assert b'+2AHab9ze-'.decode('utf-7') == '\ud801\U000abcde'

assert b'+\xc1'.decode('utf-7', 'ignore') == ''
set_d = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'(),-./:?"
set_o = '!"#$%&*;<=>@[]^_`{|}'
for c in set_d:

    assert c.encode('utf7') == c.encode('ascii')

    assert c.encode('ascii').decode('utf7') == c
for c in set_o:

    assert c.encode('ascii').decode('utf7') == c
try:
    b'+@'.decode('utf-7')
    raise AssertionError('expected UnicodeDecodeError')
except UnicodeDecodeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('ill-formed sequence', str(_aR_e))
print("UnicodeTest::test_codecs_utf7: ok")
