# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "long"
# dimension = "behavior"
# case = "long_test__test_conversion_uc5052ea"
# subject = "cpython.test_long.LongTest.test_conversion"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_long.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_long
_suite = unittest.defaultTestLoader.loadTestsFromName("LongTest.test_conversion", test_long)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython LongTest.test_conversion did not pass"
print("LongTest::test_conversion: ok")
