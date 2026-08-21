# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "test_vectors_test_case__test_legacy_block_size_warnings"
# subject = "cpython.test_hmac.TestVectorsTestCase.test_legacy_block_size_warnings"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hmac.py::TestVectorsTestCase::test_legacy_block_size_warnings
"""Auto-ported test: TestVectorsTestCase::test_legacy_block_size_warnings (CPython 3.12 oracle)."""


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
class MockCrazyHash(object):
    """Ain't no block_size attribute here."""

    def __init__(self, *args):
        self._x = hashlib.sha256(*args)
        self.digest_size = self._x.digest_size

    def update(self, v):
        self._x.update(v)

    def digest(self):
        return self._x.digest()
with warnings.catch_warnings():
    warnings.simplefilter('error', RuntimeWarning)
    try:
        hmac.HMAC(b'a', b'b', digestmod=MockCrazyHash)

        raise AssertionError('Expected warning about missing block_size')
        raise AssertionError('expected RuntimeWarning')
    except RuntimeWarning:
        pass
    MockCrazyHash.block_size = 1
    try:
        hmac.HMAC(b'a', b'b', digestmod=MockCrazyHash)

        raise AssertionError('Expected warning about small block_size')
        raise AssertionError('expected RuntimeWarning')
    except RuntimeWarning:
        pass
print("TestVectorsTestCase::test_legacy_block_size_warnings: ok")
