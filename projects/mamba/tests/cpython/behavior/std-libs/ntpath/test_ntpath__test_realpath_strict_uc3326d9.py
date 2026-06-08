# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ntpath"
# dimension = "behavior"
# case = "test_ntpath__test_realpath_strict_uc3326d9"
# subject = "cpython.test_ntpath.TestNtpath.test_realpath_strict"
# kind = "semantic"
# xfail = "auto-extracted CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_ntpath.py"
# status = "filled"
# ///
# mamba-xfail: auto-extracted CPython test; mamba promotion pending
import unittest, io
from test import test_ntpath
_suite = unittest.defaultTestLoader.loadTestsFromName("TestNtpath.test_realpath_strict", test_ntpath)
_result = unittest.TextTestRunner(stream=io.StringIO(), verbosity=0).run(_suite)
assert _result.wasSuccessful(), "CPython TestNtpath.test_realpath_strict did not pass"
print("TestNtpath::test_realpath_strict: ok")
