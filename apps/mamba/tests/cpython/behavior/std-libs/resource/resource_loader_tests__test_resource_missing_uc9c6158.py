# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_loader_tests__test_resource_missing_uc9c6158"
# subject = "cpython.test_resource.ResourceLoaderTests.test_resource_missing"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceLoaderTests.test_resource_missing", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceLoaderTests.test_resource_missing did not pass"
print("ResourceLoaderTests::test_resource_missing: ok")
