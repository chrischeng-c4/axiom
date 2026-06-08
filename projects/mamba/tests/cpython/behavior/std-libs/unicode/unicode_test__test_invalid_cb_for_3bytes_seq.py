# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_invalid_cb_for_3bytes_seq"
# subject = "cpython.test_unicode.UnicodeTest.test_invalid_cb_for_3bytes_seq"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_invalid_cb_for_3bytes_seq
"""Auto-ported test: UnicodeTest::test_invalid_cb_for_3bytes_seq (CPython 3.12 oracle)."""


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
"\n        Test that an 'invalid continuation byte' error is raised when the\n        continuation byte(s) of a 3-bytes sequence are invalid.  When\n        errors='replace', if the first continuation byte is valid, the first\n        two bytes (start byte + 1st cb) are replaced by a single U+FFFD and the\n        third byte is handled separately, otherwise only the start byte is\n        replaced with a U+FFFD and the other continuation bytes are handled\n        separately.\n        E.g. in the sequence <E1 80 41>, E1 is the start byte of a 3-bytes\n        sequence, 80 is a valid continuation byte, but 41 is not a valid cb\n        because it's the ASCII letter 'A'.\n        Note: when the start byte is E0 or ED, the valid ranges for the first\n        continuation byte are limited to A0..BF and 80..9F respectively.\n        Python 2 used to consider all the bytes in range 80..BF valid when the\n        start byte was ED.  This is fixed in Python 3.\n        "
FFFD = '�'
FFFDx2 = FFFD * 2
sequences = [('E0 00', FFFD + '\x00'), ('E0 7F', FFFD + '\x7f'), ('E0 80', FFFDx2), ('E0 9F', FFFDx2), ('E0 C0', FFFDx2), ('E0 FF', FFFDx2), ('E0 A0 00', FFFD + '\x00'), ('E0 A0 7F', FFFD + '\x7f'), ('E0 A0 C0', FFFDx2), ('E0 A0 FF', FFFDx2), ('E0 BF 00', FFFD + '\x00'), ('E0 BF 7F', FFFD + '\x7f'), ('E0 BF C0', FFFDx2), ('E0 BF FF', FFFDx2), ('E1 00', FFFD + '\x00'), ('E1 7F', FFFD + '\x7f'), ('E1 C0', FFFDx2), ('E1 FF', FFFDx2), ('E1 80 00', FFFD + '\x00'), ('E1 80 7F', FFFD + '\x7f'), ('E1 80 C0', FFFDx2), ('E1 80 FF', FFFDx2), ('E1 BF 00', FFFD + '\x00'), ('E1 BF 7F', FFFD + '\x7f'), ('E1 BF C0', FFFDx2), ('E1 BF FF', FFFDx2), ('EC 00', FFFD + '\x00'), ('EC 7F', FFFD + '\x7f'), ('EC C0', FFFDx2), ('EC FF', FFFDx2), ('EC 80 00', FFFD + '\x00'), ('EC 80 7F', FFFD + '\x7f'), ('EC 80 C0', FFFDx2), ('EC 80 FF', FFFDx2), ('EC BF 00', FFFD + '\x00'), ('EC BF 7F', FFFD + '\x7f'), ('EC BF C0', FFFDx2), ('EC BF FF', FFFDx2), ('ED 00', FFFD + '\x00'), ('ED 7F', FFFD + '\x7f'), ('ED A0', FFFDx2), ('ED BF', FFFDx2), ('ED C0', FFFDx2), ('ED FF', FFFDx2), ('ED 80 00', FFFD + '\x00'), ('ED 80 7F', FFFD + '\x7f'), ('ED 80 C0', FFFDx2), ('ED 80 FF', FFFDx2), ('ED 9F 00', FFFD + '\x00'), ('ED 9F 7F', FFFD + '\x7f'), ('ED 9F C0', FFFDx2), ('ED 9F FF', FFFDx2), ('EE 00', FFFD + '\x00'), ('EE 7F', FFFD + '\x7f'), ('EE C0', FFFDx2), ('EE FF', FFFDx2), ('EE 80 00', FFFD + '\x00'), ('EE 80 7F', FFFD + '\x7f'), ('EE 80 C0', FFFDx2), ('EE 80 FF', FFFDx2), ('EE BF 00', FFFD + '\x00'), ('EE BF 7F', FFFD + '\x7f'), ('EE BF C0', FFFDx2), ('EE BF FF', FFFDx2), ('EF 00', FFFD + '\x00'), ('EF 7F', FFFD + '\x7f'), ('EF C0', FFFDx2), ('EF FF', FFFDx2), ('EF 80 00', FFFD + '\x00'), ('EF 80 7F', FFFD + '\x7f'), ('EF 80 C0', FFFDx2), ('EF 80 FF', FFFDx2), ('EF BF 00', FFFD + '\x00'), ('EF BF 7F', FFFD + '\x7f'), ('EF BF C0', FFFDx2), ('EF BF FF', FFFDx2)]
for seq, res in sequences:
    assertCorrectUTF8Decoding(bytes.fromhex(seq), res, 'invalid continuation byte')
print("UnicodeTest::test_invalid_cb_for_3bytes_seq: ok")
