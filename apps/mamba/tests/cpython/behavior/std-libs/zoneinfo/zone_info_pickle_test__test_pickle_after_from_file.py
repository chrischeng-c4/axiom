# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "zone_info_pickle_test__test_pickle_after_from_file"
# subject = "cpython.test_zoneinfo.ZoneInfoPickleTest.test_pickle_after_from_file"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo
_suite = unittest.defaultTestLoader.loadTestsFromName("ZoneInfoPickleTest.test_pickle_after_from_file", test_zoneinfo)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZoneInfoPickleTest.test_pickle_after_from_file did not pass"
print("ZoneInfoPickleTest::test_pickle_after_from_file: ok")
