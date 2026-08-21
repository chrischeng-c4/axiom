# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_buggy_cases__test_nested_class_definition_inside_async_function"
# subject = "cpython.test_inspect.TestBuggyCases.test_nested_class_definition_inside_async_function"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestBuggyCases.test_nested_class_definition_inside_async_function", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestBuggyCases.test_nested_class_definition_inside_async_function did not pass"
print("TestBuggyCases::test_nested_class_definition_inside_async_function: ok")
