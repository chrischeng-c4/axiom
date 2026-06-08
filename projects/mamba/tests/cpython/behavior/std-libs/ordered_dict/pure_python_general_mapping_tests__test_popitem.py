# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ordered_dict"
# dimension = "behavior"
# case = "pure_python_general_mapping_tests__test_popitem"
# subject = "cpython.test_ordered_dict.PurePythonGeneralMappingTests.test_popitem"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ordered_dict.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ordered_dict
_suite = unittest.defaultTestLoader.loadTestsFromName("PurePythonGeneralMappingTests.test_popitem", test_ordered_dict)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PurePythonGeneralMappingTests.test_popitem did not pass"
print("PurePythonGeneralMappingTests::test_popitem: ok")
