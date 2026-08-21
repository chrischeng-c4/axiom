# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "main"
# dimension = "behavior"
# case = "directory_test__test_egg_info"
# subject = "cpython.test_main.DirectoryTest.test_egg_info"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_importlib/test_main.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test.test_importlib import test_main
_suite = unittest.defaultTestLoader.loadTestsFromName("DirectoryTest.test_egg_info", test_main)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython DirectoryTest.test_egg_info did not pass"
print("DirectoryTest::test_egg_info: ok")
