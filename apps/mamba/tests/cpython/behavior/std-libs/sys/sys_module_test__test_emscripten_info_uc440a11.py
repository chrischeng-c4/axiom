# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sys"
# dimension = "behavior"
# case = "sys_module_test__test_emscripten_info_uc440a11"
# subject = "cpython.test_sys.SysModuleTest.test_emscripten_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_sys.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_sys
_suite = unittest.defaultTestLoader.loadTestsFromName("SysModuleTest.test_emscripten_info", test_sys)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython SysModuleTest.test_emscripten_info did not pass"
print("SysModuleTest::test_emscripten_info: ok")
