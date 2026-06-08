# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hashlib"
# dimension = "behavior"
# case = "k_d_f_tests__test_pbkdf2_hmac_c"
# subject = "cpython.test_hashlib.KDFTests.test_pbkdf2_hmac_c"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hashlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hashlib
_suite = unittest.defaultTestLoader.loadTestsFromName("KDFTests.test_pbkdf2_hmac_c", test_hashlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython KDFTests.test_pbkdf2_hmac_c did not pass"
print("KDFTests::test_pbkdf2_hmac_c: ok")
