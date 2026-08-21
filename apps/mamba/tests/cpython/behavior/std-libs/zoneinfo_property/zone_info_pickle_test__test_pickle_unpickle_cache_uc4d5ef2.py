# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo_property"
# dimension = "behavior"
# case = "zone_info_pickle_test__test_pickle_unpickle_cache_uc4d5ef2"
# subject = "cpython.test_zoneinfo_property.ZoneInfoPickleTest.test_pickle_unpickle_cache"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo_property
_suite = unittest.defaultTestLoader.loadTestsFromName("ZoneInfoPickleTest.test_pickle_unpickle_cache", test_zoneinfo_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ZoneInfoPickleTest.test_pickle_unpickle_cache did not pass"
print("ZoneInfoPickleTest::test_pickle_unpickle_cache: ok")
