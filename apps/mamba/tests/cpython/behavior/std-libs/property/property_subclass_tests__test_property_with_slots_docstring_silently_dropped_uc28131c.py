# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "property"
# dimension = "behavior"
# case = "property_subclass_tests__test_property_with_slots_docstring_silently_dropped_uc28131c"
# subject = "cpython.test_property.PropertySubclassTests.test_property_with_slots_docstring_silently_dropped"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_property.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_property
_suite = unittest.defaultTestLoader.loadTestsFromName("PropertySubclassTests.test_property_with_slots_docstring_silently_dropped", test_property)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PropertySubclassTests.test_property_with_slots_docstring_silently_dropped did not pass"
print("PropertySubclassTests::test_property_with_slots_docstring_silently_dropped: ok")
