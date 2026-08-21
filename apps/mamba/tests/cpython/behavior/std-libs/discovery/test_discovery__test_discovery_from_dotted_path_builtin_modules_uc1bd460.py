# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "discovery"
# dimension = "behavior"
# case = "test_discovery__test_discovery_from_dotted_path_builtin_modules_uc1bd460"
# subject = "cpython.test_discovery.TestDiscovery.test_discovery_from_dotted_path_builtin_modules"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_unittest/test_discovery.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_unittest import test_discovery
_suite = unittest.defaultTestLoader.loadTestsFromName("TestDiscovery.test_discovery_from_dotted_path_builtin_modules", test_discovery)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestDiscovery.test_discovery_from_dotted_path_builtin_modules did not pass"
print("TestDiscovery::test_discovery_from_dotted_path_builtin_modules: ok")
