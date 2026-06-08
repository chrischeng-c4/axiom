# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "test_vectors_test_case__test_with_fallback"
# subject = "cpython.test_hmac.TestVectorsTestCase.test_with_fallback"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hmac.py::TestVectorsTestCase::test_with_fallback
"""Auto-ported test: TestVectorsTestCase::test_with_fallback (CPython 3.12 oracle)."""


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
cache = getattr(hashlib, '__builtin_constructor_cache')
try:
    cache['foo'] = hashlib.sha256
    hexdigest = hmac.digest(b'key', b'message', 'foo').hex()
    expected = '6e9ef29b75fffc5b7abae527d58fdadb2fe42e7219011976917343065f58ed4a'

    assert hexdigest == expected
finally:
    cache.pop('foo')
print("TestVectorsTestCase::test_with_fallback: ok")
