# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "copy_test_case__test_realcopy_old"
# subject = "cpython.test_hmac.CopyTestCase.test_realcopy_old"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_hmac.py::CopyTestCase::test_realcopy_old
"""Auto-ported test: CopyTestCase::test_realcopy_old (CPython 3.12 oracle)."""


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
h1 = hmac.HMAC.__new__(hmac.HMAC)
h1._init_old(b'key', b'msg', digestmod='sha256')
h2 = h1.copy()

assert id(h1) != id(h2)

assert id(h1._inner) != id(h2._inner)

assert id(h1._outer) != id(h2._outer)

assert h1._hmac is None
print("CopyTestCase::test_realcopy_old: ok")
