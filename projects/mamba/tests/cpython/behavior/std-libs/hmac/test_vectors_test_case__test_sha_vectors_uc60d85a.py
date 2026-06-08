# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "test_vectors_test_case__test_sha_vectors_uc60d85a"
# subject = "cpython.test_hmac.TestVectorsTestCase.test_sha_vectors"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hmac
_suite = unittest.defaultTestLoader.loadTestsFromName("TestVectorsTestCase.test_sha_vectors", test_hmac)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestVectorsTestCase.test_sha_vectors did not pass"
print("TestVectorsTestCase::test_sha_vectors: ok")
