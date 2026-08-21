# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "compare_digest_test_case__test_openssl_compare_digest_uc496705"
# subject = "cpython.test_hmac.CompareDigestTestCase.test_openssl_compare_digest"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hmac
_suite = unittest.defaultTestLoader.loadTestsFromName("CompareDigestTestCase.test_openssl_compare_digest", test_hmac)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CompareDigestTestCase.test_openssl_compare_digest did not pass"
print("CompareDigestTestCase::test_openssl_compare_digest: ok")
