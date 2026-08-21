# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "zone_info_test__test_key_attribute"
# subject = "cpython.test_zoneinfo.ZoneInfoTest.test_key_attribute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo
_suite = unittest.defaultTestLoader.loadTestsFromName("ZoneInfoTest.test_key_attribute", test_zoneinfo)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZoneInfoTest.test_key_attribute did not pass"
print("ZoneInfoTest::test_key_attribute: ok")
