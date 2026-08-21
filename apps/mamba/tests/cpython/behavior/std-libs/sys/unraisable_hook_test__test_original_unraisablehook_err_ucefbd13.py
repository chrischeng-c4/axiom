# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "unraisable_hook_test__test_original_unraisablehook_err_ucefbd13"
# subject = "cpython.test_sys.UnraisableHookTest.test_original_unraisablehook_err"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys
_suite = unittest.defaultTestLoader.loadTestsFromName("UnraisableHookTest.test_original_unraisablehook_err", test_sys)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython UnraisableHookTest.test_original_unraisablehook_err did not pass"
print("UnraisableHookTest::test_original_unraisablehook_err: ok")
