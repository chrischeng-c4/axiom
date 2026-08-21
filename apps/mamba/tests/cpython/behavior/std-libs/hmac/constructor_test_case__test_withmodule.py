# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_withmodule"
# subject = "cpython.test_hmac.ConstructorTestCase.test_withmodule"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_hmac.py::ConstructorTestCase::test_withmodule
"""Auto-ported test: ConstructorTestCase::test_withmodule (CPython 3.12 oracle)."""


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
expected = '6c845b47f52b3b47f6590c502db7825aad757bf4fadc8fa972f7cd2e76a5bdeb'
try:
    h = hmac.HMAC(b'key', b'', hashlib.sha256)
except Exception:

    raise AssertionError('Constructor call with hashlib.sha256 raised exception.')
print("ConstructorTestCase::test_withmodule: ok")
