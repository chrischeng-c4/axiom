# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_tests__test_long_asdouble_uc28e95f"
# subject = "cpython.test_long.LongTests.test_long_asdouble"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_long
_suite = unittest.defaultTestLoader.loadTestsFromName("LongTests.test_long_asdouble", test_long)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LongTests.test_long_asdouble did not pass"
print("LongTests::test_long_asdouble: ok")
