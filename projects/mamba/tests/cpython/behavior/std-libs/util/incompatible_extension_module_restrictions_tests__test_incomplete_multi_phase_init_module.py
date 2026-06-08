# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "util"
# dimension = "behavior"
# case = "incompatible_extension_module_restrictions_tests__test_incomplete_multi_phase_init_module"
# subject = "cpython.test_util.IncompatibleExtensionModuleRestrictionsTests.test_incomplete_multi_phase_init_module"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_util.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_util
_suite = unittest.defaultTestLoader.loadTestsFromName("IncompatibleExtensionModuleRestrictionsTests.test_incomplete_multi_phase_init_module", test_util)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython IncompatibleExtensionModuleRestrictionsTests.test_incomplete_multi_phase_init_module did not pass"
print("IncompatibleExtensionModuleRestrictionsTests::test_incomplete_multi_phase_init_module: ok")
