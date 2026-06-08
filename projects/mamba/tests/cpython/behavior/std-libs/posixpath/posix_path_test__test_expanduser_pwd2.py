# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "posixpath"
# dimension = "behavior"
# case = "posix_path_test__test_expanduser_pwd2"
# subject = "cpython.test_posixpath.PosixPathTest.test_expanduser_pwd2"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_posixpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_posixpath
_suite = unittest.defaultTestLoader.loadTestsFromName("PosixPathTest.test_expanduser_pwd2", test_posixpath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython PosixPathTest.test_expanduser_pwd2 did not pass"
print("PosixPathTest::test_expanduser_pwd2: ok")
