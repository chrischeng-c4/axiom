# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo_property"
# dimension = "behavior"
# case = "zone_info_test__test_key_uc848af1"
# subject = "cpython.test_zoneinfo_property.ZoneInfoTest.test_key"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo_property
_suite = unittest.defaultTestLoader.loadTestsFromName("ZoneInfoTest.test_key", test_zoneinfo_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZoneInfoTest.test_key did not pass"
print("ZoneInfoTest::test_key: ok")
