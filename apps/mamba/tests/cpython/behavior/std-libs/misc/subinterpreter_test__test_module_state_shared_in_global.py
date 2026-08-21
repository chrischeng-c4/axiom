# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "subinterpreter_test__test_module_state_shared_in_global"
# subject = "cpython.test_misc.SubinterpreterTest.test_module_state_shared_in_global"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("SubinterpreterTest.test_module_state_shared_in_global", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SubinterpreterTest.test_module_state_shared_in_global did not pass"
print("SubinterpreterTest::test_module_state_shared_in_global: ok")
