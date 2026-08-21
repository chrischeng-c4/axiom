# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "custom"
# dimension = "behavior"
# case = "custom_traversable_resources_tests__test_custom_loader_uc2c807b"
# subject = "cpython.test_custom.CustomTraversableResourcesTests.test_custom_loader"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_custom.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_custom
_suite = unittest.defaultTestLoader.loadTestsFromName("CustomTraversableResourcesTests.test_custom_loader", test_custom)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython CustomTraversableResourcesTests.test_custom_loader did not pass"
print("CustomTraversableResourcesTests::test_custom_loader: ok")
