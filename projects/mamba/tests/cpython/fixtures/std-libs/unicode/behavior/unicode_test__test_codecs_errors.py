# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_codecs_errors"
# subject = "cpython.test_unicode.UnicodeTest.test_codecs_errors"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_codecs_errors
"""Auto-ported test: UnicodeTest::test_codecs_errors (CPython 3.12 oracle)."""


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

try:
    'Andr\x82 x'.encode('ascii')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    'Andr\x82 x'.encode('ascii', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

assert 'Andr\x82 x'.encode('ascii', 'ignore') == b'Andr x'

assert 'Andr\x82 x'.encode('ascii', 'replace') == b'Andr? x'

assert 'Andr\x82 x'.encode('ascii', 'replace') == 'Andr\x82 x'.encode('ascii', errors='replace')

assert 'Andr\x82 x'.encode('ascii', 'ignore') == 'Andr\x82 x'.encode(encoding='ascii', errors='ignore')

try:
    str(b'Andr\x82 x', 'ascii')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    str(b'Andr\x82 x', 'ascii', 'strict')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

assert str(b'Andr\x82 x', 'ascii', 'ignore') == 'Andr x'

assert str(b'Andr\x82 x', 'ascii', 'replace') == 'Andr� x'

assert str(b'\x82 x', 'ascii', 'replace') == '� x'

assert b'\\N{foo}xx'.decode('unicode-escape', 'ignore') == 'xx'

try:
    b'\\'.decode('unicode-escape')
    raise AssertionError('expected UnicodeError')
except UnicodeError:
    pass

try:
    b'hello'.decode('test.unicode1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    str(b'hello', 'test.unicode2')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    'hello'.encode('test.unicode1')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    'hello'.encode('test.unicode2')
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    'hello'.encode(42, 42, 42)
    raise AssertionError('expected TypeError')
except TypeError:
    pass

try:
    int('\ud800')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    int('\udf00')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('\ud800')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    float('\udf00')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    complex('\ud800')
    raise AssertionError('expected ValueError')
except ValueError:
    pass

try:
    complex('\udf00')
    raise AssertionError('expected ValueError')
except ValueError:
    pass
print("UnicodeTest::test_codecs_errors: ok")
