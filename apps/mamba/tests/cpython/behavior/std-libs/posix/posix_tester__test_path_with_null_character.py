# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posix"
# dimension = "behavior"
# case = "posix_tester__test_path_with_null_character"
# subject = "cpython.test_posix.PosixTester.test_path_with_null_character"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posix.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posix
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixTester.test_path_with_null_character", test_posix)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixTester.test_path_with_null_character did not pass"
print("PosixTester::test_path_with_null_character: ok")
