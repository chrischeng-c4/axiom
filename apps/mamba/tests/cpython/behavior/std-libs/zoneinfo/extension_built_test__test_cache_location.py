# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "extension_built_test__test_cache_location"
# subject = "cpython.test_zoneinfo.ExtensionBuiltTest.test_cache_location"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo
_suite = unittest.defaultTestLoader.loadTestsFromName("ExtensionBuiltTest.test_cache_location", test_zoneinfo)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ExtensionBuiltTest.test_cache_location did not pass"
print("ExtensionBuiltTest::test_cache_location: ok")
