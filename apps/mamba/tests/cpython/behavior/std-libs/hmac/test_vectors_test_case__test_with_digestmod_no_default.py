# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "test_vectors_test_case__test_with_digestmod_no_default"
# subject = "cpython.test_hmac.TestVectorsTestCase.test_with_digestmod_no_default"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::TestVectorsTestCase::test_with_digestmod_no_default
"""Auto-ported test: TestVectorsTestCase::test_with_digestmod_no_default (CPython 3.12 oracle)."""


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
"""The digestmod parameter is required as of Python 3.8."""
try:
    key = b'\x0b' * 16
    data = b'Hi There'
    hmac.HMAC(key, data, digestmod=None)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('required.*digestmod', str(_aR_e))
try:
    hmac.new(key, data)
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('required.*digestmod', str(_aR_e))
try:
    hmac.HMAC(key, msg=data, digestmod='')
    raise AssertionError('expected TypeError')
except TypeError as _aR_e:
    import re as _re_aR
    assert _re_aR.search('required.*digestmod', str(_aR_e))
print("TestVectorsTestCase::test_with_digestmod_no_default: ok")
