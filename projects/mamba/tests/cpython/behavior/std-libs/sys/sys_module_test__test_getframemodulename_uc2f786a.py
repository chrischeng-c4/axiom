# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_getframemodulename_uc2f786a"
# subject = "cpython.test_sys.SysModuleTest.test_getframemodulename"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys
_suite = unittest.defaultTestLoader.loadTestsFromName("SysModuleTest.test_getframemodulename", test_sys)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SysModuleTest.test_getframemodulename did not pass"
print("SysModuleTest::test_getframemodulename: ok")
