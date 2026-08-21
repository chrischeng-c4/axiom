# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "find"
# dimension = "behavior"
# case = "test__open_g_l_libs__test_shell_injection_uc672f6d"
# subject = "cpython.test_find.Test_OpenGL_libs.test_shell_injection"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ctypes/test_find.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_ctypes import test_find
_suite = unittest.defaultTestLoader.loadTestsFromName("Test_OpenGL_libs.test_shell_injection", test_find)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython Test_OpenGL_libs.test_shell_injection did not pass"
print("Test_OpenGL_libs::test_shell_injection: ok")
