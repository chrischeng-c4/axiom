# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys_setprofile"
# dimension = "behavior"
# case = "profile_simulator_test_case__test_unbound_method_invalid_keyword_args_ucff9337"
# subject = "cpython.test_sys_setprofile.ProfileSimulatorTestCase.test_unbound_method_invalid_keyword_args"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys_setprofile.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys_setprofile
_suite = unittest.defaultTestLoader.loadTestsFromName("ProfileSimulatorTestCase.test_unbound_method_invalid_keyword_args", test_sys_setprofile)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython ProfileSimulatorTestCase.test_unbound_method_invalid_keyword_args did not pass"
print("ProfileSimulatorTestCase::test_unbound_method_invalid_keyword_args: ok")
