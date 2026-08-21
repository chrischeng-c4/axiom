# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_tests__test_long_checkexact_uc45b9be"
# subject = "cpython.test_long.LongTests.test_long_checkexact"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_long
_suite = unittest.defaultTestLoader.loadTestsFromName("LongTests.test_long_checkexact", test_long)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LongTests.test_long_checkexact did not pass"
print("LongTests::test_long_checkexact: ok")
