# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "compare_digest_test_case__test_hmac_compare_digest"
# subject = "cpython.test_hmac.CompareDigestTestCase.test_hmac_compare_digest"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hmac.py::CompareDigestTestCase::test_hmac_compare_digest
"""Auto-ported test: CompareDigestTestCase::test_hmac_compare_digest (CPython 3.12 oracle)."""


import binascii
import functools
import hmac
import hashlib
import unittest
import unittest.mock
import warnings
from test.support import hashlib_helper, check_disallow_instantiation
from _operator import _compare_digest as operator_compare_digest


try:
    import _hashlib as _hashopenssl
    from _hashlib import HMAC as C_HMAC
    from _hashlib import hmac_new as c_hmac_new
    from _hashlib import compare_digest as openssl_compare_digest
except ImportError:
    _hashopenssl = None
    C_HMAC = None
    c_hmac_new = None
    openssl_compare_digest = None

try:
    import _sha256 as sha256_module
except ImportError:
    sha256_module = None

def ignore_warning(func):

    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        with warnings.catch_warnings():
            warnings.filterwarnings('ignore', category=DeprecationWarning)
            return func(*args, **kwargs)
    return wrapper


# --- test body ---
def _test_compare_digest(compare_digest):
    a, b = (100, 200)

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (100, b'foobar')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (b'foobar', 200)

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = ('foobar', b'foobar')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (b'foobar', 'foobar')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (b'foobar', b'foo')

    assert not compare_digest(a, b)
    a, b = (b'\xde\xad\xbe\xef', b'\xde\xad')

    assert not compare_digest(a, b)
    a, b = (b'foobar', b'foobaz')

    assert not compare_digest(a, b)
    a, b = (b'\xde\xad\xbe\xef', b'\xab\xad\x1d\xea')

    assert not compare_digest(a, b)
    a, b = (b'foobar', b'foobar')

    assert compare_digest(a, b)
    a, b = (b'\xde\xad\xbe\xef', b'\xde\xad\xbe\xef')

    assert compare_digest(a, b)
    a, b = (bytearray(b'foobar'), bytearray(b'foobar'))

    assert compare_digest(a, b)
    a, b = (bytearray(b'foobar'), bytearray(b'foo'))

    assert not compare_digest(a, b)
    a, b = (bytearray(b'foobar'), bytearray(b'foobaz'))

    assert not compare_digest(a, b)
    a, b = (bytearray(b'foobar'), b'foobar')

    assert compare_digest(a, b)

    assert compare_digest(b, a)
    a, b = (bytearray(b'foobar'), b'foo')

    assert not compare_digest(a, b)

    assert not compare_digest(b, a)
    a, b = (bytearray(b'foobar'), b'foobaz')

    assert not compare_digest(a, b)

    assert not compare_digest(b, a)
    a, b = ('foobar', 'foobar')

    assert compare_digest(a, b)
    a, b = ('foo', 'foobar')

    assert not compare_digest(a, b)
    a, b = ('foobar', 'foobaz')

    assert not compare_digest(a, b)
    a, b = ('foobar', b'foobar')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (b'foobar', 'foobar')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (b'foobar', 1)

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = (100, 200)

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass
    a, b = ('fooä', 'fooä')

    try:
        compare_digest(a, b)
        raise AssertionError('expected TypeError')
    except TypeError:
        pass

    class mystr(str):

        def __eq__(self, other):
            return False
    a, b = (mystr('foobar'), mystr('foobar'))

    assert compare_digest(a, b)
    a, b = (mystr('foobar'), 'foobar')

    assert compare_digest(a, b)
    a, b = (mystr('foobar'), mystr('foobaz'))

    assert not compare_digest(a, b)

    class mybytes(bytes):

        def __eq__(self, other):
            return False
    a, b = (mybytes(b'foobar'), mybytes(b'foobar'))

    assert compare_digest(a, b)
    a, b = (mybytes(b'foobar'), b'foobar')

    assert compare_digest(a, b)
    a, b = (mybytes(b'foobar'), mybytes(b'foobaz'))

    assert not compare_digest(a, b)
_test_compare_digest(hmac.compare_digest)
if openssl_compare_digest is not None:

    assert hmac.compare_digest is openssl_compare_digest
else:

    assert hmac.compare_digest is operator_compare_digest
print("CompareDigestTestCase::test_hmac_compare_digest: ok")
