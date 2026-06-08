# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "hmac"
# dimension = "behavior"
# case = "constructor_test_case__test_with_sha256_module_ucdc5687"
# subject = "cpython.test_hmac.ConstructorTestCase.test_with_sha256_module"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hmac.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hmac
_suite = unittest.defaultTestLoader.loadTestsFromName("ConstructorTestCase.test_with_sha256_module", test_hmac)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ConstructorTestCase.test_with_sha256_module did not pass"
print("ConstructorTestCase::test_with_sha256_module: ok")
