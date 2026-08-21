# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test_set_cell_uc4f0cfa"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test_set_cell"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_funcattrs
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionPropertiesTest.test_set_cell", test_funcattrs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionPropertiesTest.test_set_cell did not pass"
print("FunctionPropertiesTest::test_set_cell: ok")
