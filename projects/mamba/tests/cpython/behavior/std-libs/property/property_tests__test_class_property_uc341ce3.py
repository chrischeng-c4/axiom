# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "property"
# dimension = "behavior"
# case = "property_tests__test_class_property_uc341ce3"
# subject = "cpython.test_property.PropertyTests.test_class_property"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_property
_suite = unittest.defaultTestLoader.loadTestsFromName("PropertyTests.test_class_property", test_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PropertyTests.test_class_property did not pass"
print("PropertyTests::test_class_property: ok")
