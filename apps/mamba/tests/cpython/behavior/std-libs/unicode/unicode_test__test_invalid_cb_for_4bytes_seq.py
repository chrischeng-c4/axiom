# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "unicode"
# dimension = "behavior"
# case = "unicode_test__test_invalid_cb_for_4bytes_seq"
# subject = "cpython.test_unicode.UnicodeTest.test_invalid_cb_for_4bytes_seq"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unicode.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_unicode.py::UnicodeTest::test_invalid_cb_for_4bytes_seq
"""Auto-ported test: UnicodeTest::test_invalid_cb_for_4bytes_seq (CPython 3.12 oracle)."""


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
"\n        Test that an 'invalid continuation byte' error is raised when the\n        continuation byte(s) of a 4-bytes sequence are invalid.  When\n        errors='replace',the start byte and all the following valid\n        continuation bytes are replaced with a single U+FFFD, and all the bytes\n        starting from the first invalid continuation bytes (included) are\n        handled separately.\n        E.g. in the sequence <E1 80 41>, E1 is the start byte of a 3-bytes\n        sequence, 80 is a valid continuation byte, but 41 is not a valid cb\n        because it's the ASCII letter 'A'.\n        Note: when the start byte is E0 or ED, the valid ranges for the first\n        continuation byte are limited to A0..BF and 80..9F respectively.\n        However, when the start byte is ED, Python 2 considers all the bytes\n        in range 80..BF valid.  This is fixed in Python 3.\n        "
FFFD = '�'
FFFDx2 = FFFD * 2
sequences = [('F0 00', FFFD + '\x00'), ('F0 7F', FFFD + '\x7f'), ('F0 80', FFFDx2), ('F0 8F', FFFDx2), ('F0 C0', FFFDx2), ('F0 FF', FFFDx2), ('F0 90 00', FFFD + '\x00'), ('F0 90 7F', FFFD + '\x7f'), ('F0 90 C0', FFFDx2), ('F0 90 FF', FFFDx2), ('F0 BF 00', FFFD + '\x00'), ('F0 BF 7F', FFFD + '\x7f'), ('F0 BF C0', FFFDx2), ('F0 BF FF', FFFDx2), ('F0 90 80 00', FFFD + '\x00'), ('F0 90 80 7F', FFFD + '\x7f'), ('F0 90 80 C0', FFFDx2), ('F0 90 80 FF', FFFDx2), ('F0 90 BF 00', FFFD + '\x00'), ('F0 90 BF 7F', FFFD + '\x7f'), ('F0 90 BF C0', FFFDx2), ('F0 90 BF FF', FFFDx2), ('F0 BF 80 00', FFFD + '\x00'), ('F0 BF 80 7F', FFFD + '\x7f'), ('F0 BF 80 C0', FFFDx2), ('F0 BF 80 FF', FFFDx2), ('F0 BF BF 00', FFFD + '\x00'), ('F0 BF BF 7F', FFFD + '\x7f'), ('F0 BF BF C0', FFFDx2), ('F0 BF BF FF', FFFDx2), ('F1 00', FFFD + '\x00'), ('F1 7F', FFFD + '\x7f'), ('F1 C0', FFFDx2), ('F1 FF', FFFDx2), ('F1 80 00', FFFD + '\x00'), ('F1 80 7F', FFFD + '\x7f'), ('F1 80 C0', FFFDx2), ('F1 80 FF', FFFDx2), ('F1 BF 00', FFFD + '\x00'), ('F1 BF 7F', FFFD + '\x7f'), ('F1 BF C0', FFFDx2), ('F1 BF FF', FFFDx2), ('F1 80 80 00', FFFD + '\x00'), ('F1 80 80 7F', FFFD + '\x7f'), ('F1 80 80 C0', FFFDx2), ('F1 80 80 FF', FFFDx2), ('F1 80 BF 00', FFFD + '\x00'), ('F1 80 BF 7F', FFFD + '\x7f'), ('F1 80 BF C0', FFFDx2), ('F1 80 BF FF', FFFDx2), ('F1 BF 80 00', FFFD + '\x00'), ('F1 BF 80 7F', FFFD + '\x7f'), ('F1 BF 80 C0', FFFDx2), ('F1 BF 80 FF', FFFDx2), ('F1 BF BF 00', FFFD + '\x00'), ('F1 BF BF 7F', FFFD + '\x7f'), ('F1 BF BF C0', FFFDx2), ('F1 BF BF FF', FFFDx2), ('F3 00', FFFD + '\x00'), ('F3 7F', FFFD + '\x7f'), ('F3 C0', FFFDx2), ('F3 FF', FFFDx2), ('F3 80 00', FFFD + '\x00'), ('F3 80 7F', FFFD + '\x7f'), ('F3 80 C0', FFFDx2), ('F3 80 FF', FFFDx2), ('F3 BF 00', FFFD + '\x00'), ('F3 BF 7F', FFFD + '\x7f'), ('F3 BF C0', FFFDx2), ('F3 BF FF', FFFDx2), ('F3 80 80 00', FFFD + '\x00'), ('F3 80 80 7F', FFFD + '\x7f'), ('F3 80 80 C0', FFFDx2), ('F3 80 80 FF', FFFDx2), ('F3 80 BF 00', FFFD + '\x00'), ('F3 80 BF 7F', FFFD + '\x7f'), ('F3 80 BF C0', FFFDx2), ('F3 80 BF FF', FFFDx2), ('F3 BF 80 00', FFFD + '\x00'), ('F3 BF 80 7F', FFFD + '\x7f'), ('F3 BF 80 C0', FFFDx2), ('F3 BF 80 FF', FFFDx2), ('F3 BF BF 00', FFFD + '\x00'), ('F3 BF BF 7F', FFFD + '\x7f'), ('F3 BF BF C0', FFFDx2), ('F3 BF BF FF', FFFDx2), ('F4 00', FFFD + '\x00'), ('F4 7F', FFFD + '\x7f'), ('F4 90', FFFDx2), ('F4 BF', FFFDx2), ('F4 C0', FFFDx2), ('F4 FF', FFFDx2), ('F4 80 00', FFFD + '\x00'), ('F4 80 7F', FFFD + '\x7f'), ('F4 80 C0', FFFDx2), ('F4 80 FF', FFFDx2), ('F4 8F 00', FFFD + '\x00'), ('F4 8F 7F', FFFD + '\x7f'), ('F4 8F C0', FFFDx2), ('F4 8F FF', FFFDx2), ('F4 80 80 00', FFFD + '\x00'), ('F4 80 80 7F', FFFD + '\x7f'), ('F4 80 80 C0', FFFDx2), ('F4 80 80 FF', FFFDx2), ('F4 80 BF 00', FFFD + '\x00'), ('F4 80 BF 7F', FFFD + '\x7f'), ('F4 80 BF C0', FFFDx2), ('F4 80 BF FF', FFFDx2), ('F4 8F 80 00', FFFD + '\x00'), ('F4 8F 80 7F', FFFD + '\x7f'), ('F4 8F 80 C0', FFFDx2), ('F4 8F 80 FF', FFFDx2), ('F4 8F BF 00', FFFD + '\x00'), ('F4 8F BF 7F', FFFD + '\x7f'), ('F4 8F BF C0', FFFDx2), ('F4 8F BF FF', FFFDx2)]
for seq, res in sequences:
    assertCorrectUTF8Decoding(bytes.fromhex(seq), res, 'invalid continuation byte')
print("UnicodeTest::test_invalid_cb_for_4bytes_seq: ok")
