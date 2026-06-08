# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "discovery"
# dimension = "behavior"
# case = "test_discovery__test_discover_with_init_module_that_raises_skiptest_on_import_ucd423ac"
# subject = "cpython.test_discovery.TestDiscovery.test_discover_with_init_module_that_raises_SkipTest_on_import"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_discovery.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_discovery
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDiscovery.test_discover_with_init_module_that_raises_SkipTest_on_import", test_discovery)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDiscovery.test_discover_with_init_module_that_raises_SkipTest_on_import did not pass"
print("TestDiscovery::test_discover_with_init_module_that_raises_SkipTest_on_import: ok")
