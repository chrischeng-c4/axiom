# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "discovery"
# dimension = "behavior"
# case = "test_discovery__test_find_tests_customize_via_package_pattern_ucb86c8a"
# subject = "cpython.test_discovery.TestDiscovery.test_find_tests_customize_via_package_pattern"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_discovery.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_discovery
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDiscovery.test_find_tests_customize_via_package_pattern", test_discovery)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDiscovery.test_find_tests_customize_via_package_pattern did not pass"
print("TestDiscovery::test_find_tests_customize_via_package_pattern: ok")
