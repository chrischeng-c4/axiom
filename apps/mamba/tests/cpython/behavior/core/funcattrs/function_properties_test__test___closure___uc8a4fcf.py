# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "funcattrs"
# dimension = "behavior"
# case = "function_properties_test__test___closure___uc8a4fcf"
# subject = "cpython.test_funcattrs.FunctionPropertiesTest.test___closure__"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_funcattrs.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_funcattrs
_suite = unittest.defaultTestLoader.loadTestsFromName("FunctionPropertiesTest.test___closure__", test_funcattrs)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython FunctionPropertiesTest.test___closure__ did not pass"
print("FunctionPropertiesTest::test___closure__: ok")
