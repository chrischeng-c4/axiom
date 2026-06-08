# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "behavior"
# case = "tz_path_test__test_env_variable_relative_paths_warning_location"
# subject = "cpython.test_zoneinfo.TzPathTest.test_env_variable_relative_paths_warning_location"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_zoneinfo/test_zoneinfo.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_zoneinfo import test_zoneinfo
_suite = unittest.defaultTestLoader.loadTestsFromName("TzPathTest.test_env_variable_relative_paths_warning_location", test_zoneinfo)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TzPathTest.test_env_variable_relative_paths_warning_location did not pass"
print("TzPathTest::test_env_variable_relative_paths_warning_location: ok")
