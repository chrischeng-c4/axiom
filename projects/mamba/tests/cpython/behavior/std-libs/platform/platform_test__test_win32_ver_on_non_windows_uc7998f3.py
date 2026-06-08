# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_win32_ver_on_non_windows_uc7998f3"
# subject = "cpython.test_platform.PlatformTest.test_win32_ver_on_non_windows"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_platform
_suite = unittest.defaultTestLoader.loadTestsFromName("PlatformTest.test_win32_ver_on_non_windows", test_platform)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PlatformTest.test_win32_ver_on_non_windows did not pass"
print("PlatformTest::test_win32_ver_on_non_windows: ok")
