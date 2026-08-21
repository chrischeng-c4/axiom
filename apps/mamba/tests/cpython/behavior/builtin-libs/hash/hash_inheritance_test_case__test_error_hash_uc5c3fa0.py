# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "hash"
# dimension = "behavior"
# case = "hash_inheritance_test_case__test_error_hash_uc5c3fa0"
# subject = "cpython.test_hash.HashInheritanceTestCase.test_error_hash"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hash.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hash
_suite = unittest.defaultTestLoader.loadTestsFromName("HashInheritanceTestCase.test_error_hash", test_hash)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashInheritanceTestCase.test_error_hash did not pass"
print("HashInheritanceTestCase::test_error_hash: ok")
