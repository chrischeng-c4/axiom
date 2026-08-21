# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "calling_convention_test__test_clear_cache"
# subject = "cpython.test_zoneinfo.CallingConventionTest.test_clear_cache"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo
_suite = unittest.defaultTestLoader.loadTestsFromName("CallingConventionTest.test_clear_cache", test_zoneinfo)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CallingConventionTest.test_clear_cache did not pass"
print("CallingConventionTest::test_clear_cache: ok")
