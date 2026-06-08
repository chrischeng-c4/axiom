# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "misc"
# dimension = "behavior"
# case = "test__module_state_access__test_get_module_bad_def"
# subject = "cpython.test_misc.Test_ModuleStateAccess.test_get_module_bad_def"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_capi/test_misc.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_capi import test_misc
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_ModuleStateAccess.test_get_module_bad_def", test_misc)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_ModuleStateAccess.test_get_module_bad_def did not pass"
print("Test_ModuleStateAccess::test_get_module_bad_def: ok")
