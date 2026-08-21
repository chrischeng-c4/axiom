# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "binary_converter_tests__test_binary_input_for_converter"
# subject = "cpython.test_types.BinaryConverterTests.test_binary_input_for_converter"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_sqlite3 import test_types
_suite = unittest.defaultTestLoader.loadTestsFromName("BinaryConverterTests.test_binary_input_for_converter", test_types)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython BinaryConverterTests.test_binary_input_for_converter did not pass"
print("BinaryConverterTests::test_binary_input_for_converter: ok")
