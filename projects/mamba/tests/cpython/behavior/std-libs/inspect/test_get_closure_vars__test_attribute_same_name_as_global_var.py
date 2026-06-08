# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "test_get_closure_vars__test_attribute_same_name_as_global_var"
# subject = "cpython.test_inspect.TestGetClosureVars.test_attribute_same_name_as_global_var"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_inspect/test_inspect.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_inspect import test_inspect
_suite = unittest.defaultTestLoader.loadTestsFromName("TestGetClosureVars.test_attribute_same_name_as_global_var", test_inspect)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestGetClosureVars.test_attribute_same_name_as_global_var did not pass"
print("TestGetClosureVars::test_attribute_same_name_as_global_var: ok")
