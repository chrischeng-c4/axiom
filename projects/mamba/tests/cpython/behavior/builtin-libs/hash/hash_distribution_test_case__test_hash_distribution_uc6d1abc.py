# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "hash"
# dimension = "behavior"
# case = "hash_distribution_test_case__test_hash_distribution_uc6d1abc"
# subject = "cpython.test_hash.HashDistributionTestCase.test_hash_distribution"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hash.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hash
_suite = unittest.defaultTestLoader.loadTestsFromName("HashDistributionTestCase.test_hash_distribution", test_hash)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashDistributionTestCase.test_hash_distribution did not pass"
print("HashDistributionTestCase::test_hash_distribution: ok")
