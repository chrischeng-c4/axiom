# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pathlib"
# dimension = "behavior"
# case = "posix_path_test__test_absolute"
# subject = "cpython.test_pathlib.PosixPathTest.test_absolute"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_pathlib.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_pathlib
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixPathTest.test_absolute", test_pathlib)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixPathTest.test_absolute did not pass"
print("PosixPathTest::test_absolute: ok")
