# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "hash"
# dimension = "behavior"
# case = "hash_equality_test_case__test_unaligned_buffers_ucd9c19d"
# subject = "cpython.test_hash.HashEqualityTestCase.test_unaligned_buffers"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hash.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hash
_suite = unittest.defaultTestLoader.loadTestsFromName("HashEqualityTestCase.test_unaligned_buffers", test_hash)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython HashEqualityTestCase.test_unaligned_buffers did not pass"
print("HashEqualityTestCase::test_unaligned_buffers: ok")
