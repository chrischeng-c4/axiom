# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "resource"
# dimension = "behavior"
# case = "resource_corner_case_tests__test_package_has_no_reader_fallback_uca86bc1"
# subject = "cpython.test_resource.ResourceCornerCaseTests.test_package_has_no_reader_fallback"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/resources/test_resource.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib.resources import test_resource
_suite = unittest.defaultTestLoader.loadTestsFromName("ResourceCornerCaseTests.test_package_has_no_reader_fallback", test_resource)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ResourceCornerCaseTests.test_package_has_no_reader_fallback did not pass"
print("ResourceCornerCaseTests::test_package_has_no_reader_fallback: ok")
