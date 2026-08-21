# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "discovery_tests__test_invalid_usage"
# subject = "cpython.test_main.DiscoveryTests.test_invalid_usage"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("DiscoveryTests.test_invalid_usage", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DiscoveryTests.test_invalid_usage did not pass"
print("DiscoveryTests::test_invalid_usage: ok")
