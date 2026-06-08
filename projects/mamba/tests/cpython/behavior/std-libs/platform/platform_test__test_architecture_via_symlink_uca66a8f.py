# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "platform"
# dimension = "behavior"
# case = "platform_test__test_architecture_via_symlink_uca66a8f"
# subject = "cpython.test_platform.PlatformTest.test_architecture_via_symlink"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_platform.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_platform
_suite = unittest.defaultTestLoader.loadTestsFromName("PlatformTest.test_architecture_via_symlink", test_platform)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PlatformTest.test_architecture_via_symlink did not pass"
print("PlatformTest::test_architecture_via_symlink: ok")
