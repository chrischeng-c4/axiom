# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "hash"
# dimension = "behavior"
# case = "memoryview_hash_randomization_tests__test_empty_string_ucba5902"
# subject = "cpython.test_hash.MemoryviewHashRandomizationTests.test_empty_string"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_hash.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_hash
_suite = unittest.defaultTestLoader.loadTestsFromName("MemoryviewHashRandomizationTests.test_empty_string", test_hash)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython MemoryviewHashRandomizationTests.test_empty_string did not pass"
print("MemoryviewHashRandomizationTests::test_empty_string: ok")
