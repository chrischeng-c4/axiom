# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo_property"
# dimension = "behavior"
# case = "zone_info_cache_test__test_cache_uc077bbd"
# subject = "cpython.test_zoneinfo_property.ZoneInfoCacheTest.test_cache"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo_property
_suite = unittest.defaultTestLoader.loadTestsFromName("ZoneInfoCacheTest.test_cache", test_zoneinfo_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZoneInfoCacheTest.test_cache did not pass"
print("ZoneInfoCacheTest::test_cache: ok")
