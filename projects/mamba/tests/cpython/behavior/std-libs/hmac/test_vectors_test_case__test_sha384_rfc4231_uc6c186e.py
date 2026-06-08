# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "test_vectors_test_case__test_sha384_rfc4231_uc6c186e"
# subject = "cpython.test_hmac.TestVectorsTestCase.test_sha384_rfc4231"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hmac
_suite = unittest.defaultTestLoader.loadTestsFromName("TestVectorsTestCase.test_sha384_rfc4231", test_hmac)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestVectorsTestCase.test_sha384_rfc4231 did not pass"
print("TestVectorsTestCase::test_sha384_rfc4231: ok")
