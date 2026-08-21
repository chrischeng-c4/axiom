# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "crypt"
# dimension = "behavior"
# case = "crypt_test_case__test_sha2_rounds_uccd7288"
# subject = "cpython.test_crypt.CryptTestCase.test_sha2_rounds"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_crypt.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_crypt
_suite = unittest.defaultTestLoader.loadTestsFromName("CryptTestCase.test_sha2_rounds", test_crypt)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CryptTestCase.test_sha2_rounds did not pass"
print("CryptTestCase::test_sha2_rounds: ok")
