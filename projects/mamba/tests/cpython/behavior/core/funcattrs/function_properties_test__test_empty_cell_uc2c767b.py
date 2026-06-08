# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test_empty_cell_uc2c767b"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test_empty_cell"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_funcattrs
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionPropertiesTest.test_empty_cell", test_funcattrs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionPropertiesTest.test_empty_cell did not pass"
print("FunctionPropertiesTest::test_empty_cell: ok")
